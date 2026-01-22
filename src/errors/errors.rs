use actix_web::{HttpResponse, http::StatusCode};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("invalid email")]
    InvalidEmail,

    #[error("name required")]
    NameRequired,

    #[error("name too short")]
    NameTooShort,

    #[error("password required")]
    PasswordRequired,

    #[error("password too long")]
    PasswordTooLong,

    #[error("weak password")]
    WeakPassword,

    #[error("wrong password")]
    WrongPassword,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("duplicate email")]
    DuplicateEmail,

    #[error("unauthorized")]
    Unauthorized,

    #[error("user not found")]
    UserNotFound,

    #[error("user required")]
    UserRequired,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl actix_web::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::DuplicateEmail => StatusCode::CONFLICT,
            AuthError::UserNotFound => StatusCode::NOT_FOUND,
            AuthError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::Internal(e) => {
                tracing::error!(error = ?e);

                HttpResponse::InternalServerError().json(ErrorResponse {
                    message: "Internal Server Error".to_string(),
                })
            }
            _ => HttpResponse::build(self.status_code()).json(ErrorResponse {
                message: self.to_string(),
            }),
        }
    }
}

#[derive(Debug)]
pub enum CategoryError {}

pub fn e400(error: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({
        "error": error
    }))
}

pub fn e401(error: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({
        "error": error
    }))
}

pub fn e404(error: &str) -> HttpResponse {
    HttpResponse::NotFound().json(json!({
        "error": error
    }))
}

pub fn e409(error: &str) -> HttpResponse {
    HttpResponse::Conflict().json(json!({
        "error": error
    }))
}

pub fn e500(error: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "error": error
    }))
}
