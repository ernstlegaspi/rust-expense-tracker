use actix_web::{HttpResponse, http::StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum CategoryError {
    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("name already existing")]
    NameExisting,

    #[error("name required")]
    NameRequired,

    #[error("name too long")]
    NameTooLong,

    #[error("name too short")]
    NameTooShort,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl actix_web::ResponseError for CategoryError {
    fn status_code(&self) -> StatusCode {
        match self {
            CategoryError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CategoryError::NameExisting => StatusCode::CONFLICT,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        tracing::error!(error = ?self.to_string());

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            message: self.to_string(),
        })
    }
}

impl CategoryError {
    pub fn internal(e: impl Into<anyhow::Error>) -> Self {
        CategoryError::Internal(e.into())
    }
}
