use actix_web::{HttpResponse, http::StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("invalid email")]
    InvalidEmail,

    #[error("name required")]
    NameRequired,

    #[error("name too long")]
    NameTooLong,

    #[error("name too short")]
    NameTooShort,

    #[error("password required")]
    PasswordRequired,

    #[error("password too long")]
    PasswordTooLong,

    #[error("weak password")]
    WeakPassword,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("duplicate email")]
    DuplicateEmail,

    #[error("unauthorized")]
    Unauthorized,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl actix_web::ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            AuthError::DuplicateEmail => StatusCode::CONFLICT,
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
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

impl AuthError {
    pub fn internal(e: impl Into<anyhow::Error>) -> Self {
        AuthError::Internal(e.into())
    }
}
