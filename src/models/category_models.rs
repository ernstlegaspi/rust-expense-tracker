use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::errors::category_errors::CategoryError;

#[derive(serde::Deserialize)]
pub struct Category {
    pub name: String,
}

impl Category {
    pub fn validate(&self) -> Result<(), CategoryError> {
        if self.name.is_empty() {
            return Err(CategoryError::NameRequired);
        }

        if self.name.len() < 3 {
            return Err(CategoryError::NameTooShort);
        }

        if self.name.len() > 30 {
            return Err(CategoryError::NameTooLong);
        }

        Ok(())
    }
}

#[derive(FromRow, serde::Serialize)]
pub struct AddCategoryResponse {
    id: Uuid,
    name: String,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    description: Option<String>,
}
