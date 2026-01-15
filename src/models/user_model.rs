use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::ValidateEmail;
use zxcvbn::{Score, zxcvbn};

#[derive(Deserialize, Serialize)]
pub struct CreateUser {
    pub email: String,
    pub name: String,
    pub password: String,
}

impl CreateUser {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.name.is_empty() {
            return Err("Name is required.".to_string());
        }

        if self.name.len() < 3 {
            return Err("Name should have at least 3 characters.".to_string());
        }

        if !self.email.validate_email() {
            return Err("Please enter a valid email.".to_string());
        }

        let user_inputs = &[self.email.as_str(), self.name.as_str()];
        let estimate = zxcvbn(&self.password, user_inputs);

        if estimate.score() < Score::Three {
            return Err("Password too weak.".to_string());
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
