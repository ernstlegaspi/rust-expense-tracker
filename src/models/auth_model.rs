use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::result::Result;
use uuid::Uuid;
use validator::ValidateEmail;
use zxcvbn::{Score, zxcvbn};

use crate::errors::errors::{LoginError, RegisterError};

#[derive(FromRow, Serialize)]
pub struct RegisterResponse {
    pub email: String,
    pub name: String,
    pub uuid: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct Register {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl Register {
    pub fn validate(&self) -> Result<(), RegisterError> {
        if self.name.is_empty() {
            return Err(RegisterError::NameRequired);
        }

        if self.name.len() < 3 {
            return Err(RegisterError::InvalidNameLength);
        }

        if !self.email.validate_email() {
            return Err(RegisterError::InvalidEmail);
        }

        let user_inputs = &[self.email.as_str(), self.name.as_str()];
        let estimate = zxcvbn(&self.password, user_inputs);

        if estimate.score() < Score::Three {
            return Err(RegisterError::WeakPassword);
        }

        Ok(())
    }
}

#[derive(FromRow, Serialize)]
pub struct LoginResponse {
    pub email: String,
    pub name: String,
    pub password: String,
    pub uuid: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

impl Login {
    pub fn validate(&self) -> Result<(), LoginError> {
        if !self.email.validate_email() {
            return Err(LoginError::InvalidEmail);
        }

        if self.password.is_empty() {
            return Err(LoginError::PasswordRequired);
        }

        if self.password.len() < 8 {
            return Err(LoginError::WeakPassword);
        }

        Ok(())
    }
}
