use actix_web::{HttpResponse, http::StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum ExpenseError {
    #[error("category id required")]
    CategoryIDRequired,

    #[error("description required")]
    DescriptionRequired,

    #[error("description too long")]
    DescriptionTooLong,

    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("invalid amount value")]
    InvalidAmountValue,

    #[error("required field missing")]
    RequiredFieldMissing,
}

#[derive(serde::Serialize)]
struct ErrorResponse {
    message: String,
}

impl actix_web::ResponseError for ExpenseError {
    fn status_code(&self) -> StatusCode {
        match self {
            ExpenseError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            message: self.to_string(),
        })
    }
}

impl ExpenseError {
    pub fn internal(e: impl Into<anyhow::Error>) -> Self {
        ExpenseError::Internal(e.into())
    }
}
