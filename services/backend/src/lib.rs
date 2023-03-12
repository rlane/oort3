pub mod discord;
pub mod leaderboard;
pub mod shortcode;
pub mod telemetry;
pub mod tournament;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub fn project_id() -> String {
    std::env::var("PROJECT_ID").expect("missing PROJECT_ID environment variable")
}

pub fn error(status_code: StatusCode, msg: String) -> Error {
    Error {
        status_code,
        err: anyhow::anyhow!(msg),
    }
}

pub struct Error {
    status_code: StatusCode,
    err: anyhow::Error,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status_code, self.err.to_string()).into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            err: err.into(),
        }
    }
}
