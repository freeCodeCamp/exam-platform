//! S3-compatible object storage. Knows no provider: an endpoint, signing region, bucket and key
//! pair from env select it, so switching provider is a config change. Copy the bucket's objects
//! across, keeping their keys.
//!
//! One [`ObjectStore`] per bucket. Each workload (curriculum, evidence) has its own bucket and
//! credentials, under its own env prefix: [`S3Config::from_env`].
use std::{env, fmt, time::Duration};

use aws_sdk_s3::{
    Client, Config,
    config::{
        BehaviorVersion, Credentials, Region, RequestChecksumCalculation,
        ResponseChecksumValidation,
    },
    error::{DisplayErrorContext, SdkError},
    presigning::{PresigningConfig, PresigningConfigError},
    primitives::ByteStream,
};
use bytes::Bytes;
use url::Url;

/// One bucket on one S3-compatible provider.
#[derive(Clone, PartialEq, Eq)]
pub struct S3Config {
    /// Scheme and host, no path, e.g. `https://nyc3.digitaloceanspaces.com`.
    pub endpoint: String,
    /// SigV4 signing region. Providers without regions take any; DO Spaces signs as `us-east-1`,
    /// R2 as `auto`.
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    /// `true`: `<endpoint>/<bucket>/<key>` (MinIO, Garage). `false`: virtual-hosted,
    /// `<bucket>.<endpoint host>/<key>`.
    pub force_path_style: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} not set")]
    Missing(String),
    #[error("{0}: {1}")]
    Invalid(String, &'static str),
}

impl S3Config {
    /// Reads `<prefix>_S3_ENDPOINT`, `_REGION`, `_BUCKET`, `_ACCESS_KEY_ID`,
    /// `_SECRET_ACCESS_KEY`, and optionally `_FORCE_PATH_STYLE` (`true`/`false`, default `false`).
    /// Empty counts as unset.
    pub fn from_env(prefix: &str) -> Result<Self, ConfigError> {
        let get = |suffix: &str| {
            let name = format!("{prefix}_S3_{suffix}");
            match env::var(&name) {
                Ok(v) if !v.is_empty() => Ok(v),
                _ => Err(ConfigError::Missing(name)),
            }
        };
        let force_path_style = match get("FORCE_PATH_STYLE") {
            Err(_) => false,
            Ok(v) => v.parse().map_err(|_| {
                ConfigError::Invalid(format!("{prefix}_S3_FORCE_PATH_STYLE"), "not true or false")
            })?,
        };
        let config = Self {
            endpoint: get("ENDPOINT")?,
            region: get("REGION")?,
            bucket: get("BUCKET")?,
            access_key_id: get("ACCESS_KEY_ID")?,
            secret_access_key: get("SECRET_ACCESS_KEY")?,
            force_path_style,
        };
        config
            .validate()
            .map_err(|reason| ConfigError::Invalid(format!("{prefix}_S3_ENDPOINT"), reason))?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), &'static str> {
        let url = Url::parse(&self.endpoint).map_err(|_| "not a URL")?;
        if !matches!(url.scheme(), "https" | "http") {
            return Err("scheme must be https or http");
        }
        if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
            return Err("must be scheme and host only");
        }
        Ok(())
    }
}

/// Redacts the secret.
impl fmt::Debug for S3Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3Config")
            .field("endpoint", &self.endpoint)
            .field("region", &self.region)
            .field("bucket", &self.bucket)
            .field("access_key_id", &self.access_key_id)
            .field("secret_access_key", &"<redacted>")
            .field("force_path_style", &self.force_path_style)
            .finish()
    }
}

/// An S3 request failed. Holds the SDK's full error chain: for logs, never for clients.
#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct StorageError(String);

impl<E, R> From<SdkError<E, R>> for StorageError
where
    E: std::error::Error + 'static,
    R: fmt::Debug,
{
    fn from(error: SdkError<E, R>) -> Self {
        Self(DisplayErrorContext(&error).to_string())
    }
}

impl From<PresigningConfigError> for StorageError {
    fn from(error: PresigningConfigError) -> Self {
        Self(DisplayErrorContext(&error).to_string())
    }
}

/// One bucket. No `list`: keys live in a database, and listing is billed per call on some
/// providers. No `delete`: callers own retention.
#[derive(Debug, Clone)]
pub struct ObjectStore {
    client: Client,
    bucket: String,
}

impl ObjectStore {
    /// Builds the client. No network: connects on first request.
    pub fn new(config: &S3Config) -> Self {
        let sdk_config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(&config.endpoint)
            .region(Region::new(config.region.clone()))
            .credentials_provider(Credentials::new(
                &config.access_key_id,
                &config.secret_access_key,
                None,
                None,
                "object-storage-env",
            ))
            .force_path_style(config.force_path_style)
            // aws-sdk-s3 sends and checks CRC32 checksums on every request by default. Not every
            // S3-compatible provider supports them, so only when the operation requires one.
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired)
            .build();

        Self {
            client: Client::from_conf(sdk_config),
            bucket: config.bucket.clone(),
        }
    }

    /// Stores `body` at `key`, private (the bucket's default ACL). Single-part: callers bound the
    /// size.
    pub async fn put(
        &self,
        key: &str,
        body: Bytes,
        content_type: &str,
    ) -> Result<(), StorageError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .body(ByteStream::from(body))
            .send()
            .await?;
        Ok(())
    }

    /// A URL anyone holding it can `GET` `key` from, until `expires_in` passes. Signed locally: no
    /// request, and no check that `key` exists.
    pub async fn presigned_get(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, StorageError> {
        let request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;
        Ok(request.uri().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(endpoint: &str) -> S3Config {
        S3Config {
            endpoint: endpoint.to_string(),
            region: "us-east-1".to_string(),
            bucket: "bucket".to_string(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            force_path_style: false,
        }
    }

    #[test]
    fn endpoint_is_scheme_and_host() {
        assert!(
            config("https://nyc3.digitaloceanspaces.com")
                .validate()
                .is_ok()
        );
        assert!(config("http://127.0.0.1:9000").validate().is_ok());
        assert!(config("nyc3.digitaloceanspaces.com").validate().is_err());
        assert!(config("ftp://example.com").validate().is_err());
        assert!(config("https://example.com/bucket").validate().is_err());
        assert!(config("https://example.com?x=1").validate().is_err());
    }

    #[test]
    fn debug_hides_secret() {
        let debug = format!("{:?}", config("https://example.com"));

        assert!(!debug.contains("\"secret\""), "{debug}");
        assert!(debug.contains("<redacted>"));
    }

    /// Round trip against a real S3-compatible server, e.g. Garage or SeaweedFS, path style:
    /// `TEST_S3_ENDPOINT=http://127.0.0.1:3900 TEST_S3_REGION=… TEST_S3_ACCESS_KEY_ID=…
    /// TEST_S3_SECRET_ACCESS_KEY=… TEST_S3_BUCKET=… TEST_S3_FORCE_PATH_STYLE=true`. The bucket
    /// must exist. Unset: the test passes without running.
    #[tokio::test]
    async fn round_trip() -> Result<(), StorageError> {
        let config = match S3Config::from_env("TEST") {
            Ok(config) => config,
            Err(e) => {
                eprintln!("{e}: skipping S3 round trip");
                return Ok(());
            }
        };
        let store = ObjectStore::new(&config);
        let key = format!("test-{}/a.txt", uuid::Uuid::new_v4());

        store
            .put(&key, Bytes::from_static(b"hello"), "text/plain")
            .await?;

        let url = store.presigned_get(&key, Duration::from_secs(60)).await?;
        let response = reqwest::get(&url)
            .await
            .and_then(reqwest::Response::error_for_status)
            .expect("presigned GET succeeds");
        assert_eq!(
            response.headers()["content-type"].to_str().ok(),
            Some("text/plain")
        );
        assert_eq!(response.text().await.expect("text body"), "hello");
        Ok(())
    }
}
