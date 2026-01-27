use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::errors::category_errors::CategoryError;

#[derive(Deserialize)]
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

#[derive(Deserialize, FromRow, Serialize)]
pub struct CategoryResponse {
    id: Uuid,
    description: Option<String>,
    name: String,
    user_id: Uuid,
}

// for dev mode only
#[derive(Serialize)]
pub struct CategoriesCached {
    pub cached: bool,
    pub categories: Vec<CategoryResponse>,
}

#[derive(Deserialize)]
pub struct CategoryPagination {
    #[serde(default = "default_page")]
    pub page: i64,
}

fn default_page() -> i64 {
    1
}
