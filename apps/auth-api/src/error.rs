use axum::response::{IntoResponse, Response};
use http::StatusCode;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{1}")]
    Server(StatusCode, String),
    #[error("{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("{0}")]
    TimeConversion(#[from] time::error::ConversionRange),
}

impl Error {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::Server(StatusCode::BAD_REQUEST, message.into())
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Server(StatusCode::UNAUTHORIZED, message.into())
    }
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
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
