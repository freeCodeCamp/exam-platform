use axum::response::{IntoResponse, Response};
use http::StatusCode;

use object_storage::StorageError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{1}")]
    Server(StatusCode, String),
    #[error("{1}")]
    InvalidConfig(StatusCode, String),
    // Froms
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    Cdb(#[from] cdb::Error),
    /// Details stay in logs: they name the bucket and provider.
    #[error("object storage request failed")]
    Storage(#[from] StorageError),
    #[error("{0}")]
    TimeConversion(#[from] time::error::ConversionRange),
    #[error("{0}")]
    TimeFormat(#[from] time::error::Format),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if let Self::Storage(error) = &self {
            tracing::error!(%error, "object storage request failed");
        }
        let msg = self.to_string();
        let status: StatusCode = self.into();

        (status, msg).into_response()
    }
}

impl From<Error> for StatusCode {
    fn from(error: Error) -> Self {
        match error {
            Error::Server(c, _) => c,
            Error::InvalidConfig(c, _) => c,
            Error::Storage(_) => StatusCode::BAD_GATEWAY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
