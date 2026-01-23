use actix_web::{HttpResponse, http::StatusCode};

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
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            message: self.to_string(),
        })
    }
}
