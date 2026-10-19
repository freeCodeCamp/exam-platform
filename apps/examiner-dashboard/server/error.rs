use axum::response::{IntoResponse, Response};
use http::StatusCode;

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
    TowerSessions(#[from] tower_sessions::session::Error),
    #[error("{0}")]
    TimeConversion(#[from] time::error::ConversionRange),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
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
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
