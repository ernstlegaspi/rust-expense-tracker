use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::ValidateEmail;
use zxcvbn::{Score, zxcvbn};

use crate::errors::errors::UserError;

#[derive(Deserialize, Serialize)]
pub struct Register {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl Register {
    pub fn validate(&self) -> std::result::Result<(), UserError> {
        if self.name.is_empty() {
            return Err(UserError::NameRequired);
        }

        if self.name.len() < 3 {
            return Err(UserError::InvalidNameLength);
        }

        if !self.email.validate_email() {
            return Err(UserError::InvalidEmail);
        }

        let user_inputs = &[self.email.as_str(), self.name.as_str()];
        let estimate = zxcvbn(&self.password, user_inputs);

        if estimate.score() < Score::Three {
            return Err(UserError::WeakPassword);
        }

        Ok(())
    }
}

#[derive(FromRow, Serialize)]
pub struct UserResponse {
    pub email: String,
    pub name: String,
    pub uuid: Uuid,
}
