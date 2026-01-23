use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::result::Result;
use uuid::Uuid;
use validator::ValidateEmail;
use zxcvbn::{Score, zxcvbn};

use crate::errors::auth_errors::ValidationError;

#[derive(FromRow, Serialize)]
pub struct AuthResponse {
    pub email: String,
    pub id: Uuid,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct Register {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl Register {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::NameRequired);
        }

        if self.name.len() < 3 || self.name.len() > 100 {
            return Err(ValidationError::NameTooShort);
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

#[derive(FromRow, Serialize)]
pub struct LoginResponse {
    pub email: String,
    pub id: Uuid,
    pub name: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

impl Login {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if !self.email.validate_email() {
            return Err(ValidationError::InvalidEmail);
        }

        if self.password.is_empty() {
            return Err(ValidationError::PasswordRequired);
        }

        if self.password.len() < 8 {
            return Err(ValidationError::WeakPassword);
        }

        Ok(())
    }
}

#[derive(FromRow, Deserialize)]
pub struct RefreshResponse {
    pub id: Uuid,
}
