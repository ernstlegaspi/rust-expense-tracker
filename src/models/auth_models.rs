use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::result::Result;
use uuid::Uuid;
use validator::ValidateEmail;
use zxcvbn::{Score, zxcvbn};

use crate::errors::auth_errors::ValidationError;

#[derive(FromRow)]
pub struct UserQuery {
    pub email: String,
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub email: String,
    pub refresh_token: String,
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl RegisterRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::NameRequired);
        }

        if self.name.len() < 3 {
            return Err(ValidationError::NameTooShort);
        }

        if self.name.len() > 100 {
            return Err(ValidationError::NameTooLong);
        }

        if !self.email.validate_email() || self.email.len() > 254 {
            return Err(ValidationError::InvalidEmail);
        }

        if self.password.len() > 72 {
            return Err(ValidationError::PasswordTooLong);
        }

        let user_inputs = &[self.email.as_str(), self.name.as_str()];
        let estimate = zxcvbn(&self.password, user_inputs);

        if estimate.score() < Score::Three {
            return Err(ValidationError::WeakPassword);
        }

        Ok(())
    }
}

#[derive(FromRow)]
pub struct LoginQuery {
    pub email: String,
    pub id: Uuid,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl LoginRequest {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.email.validate_email() {
            return Err(ValidationError::InvalidEmail);
        }

        if self.password.is_empty() {
            return Err(ValidationError::PasswordRequired);
        }

        Ok(())
    }
}
