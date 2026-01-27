use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::errors::expense_errors::ExpenseError;

#[derive(Deserialize)]
pub struct ExpenseRequest {
    pub amount: Decimal,
    pub description: String,
    pub category_id: Uuid,
    pub date: NaiveDate,
    pub payment_method: Option<String>,
    pub is_recurring: bool,
    pub tags: Option<Vec<String>>,
}

impl ExpenseRequest {
    pub fn validate(&self) -> Result<(), ExpenseError> {
        if self.amount <= Decimal::ZERO {
            return Err(ExpenseError::InvalidAmountValue);
        }

        if self.description.is_empty() {
            return Err(ExpenseError::DescriptionRequired);
        }

        if self.description.len() > 255 {
            return Err(ExpenseError::DescriptionTooLong);
        }

        if self.category_id.is_nil() {
            return Err(ExpenseError::CategoryIDRequired);
        }

        Ok(())
    }
}

// properties are not reusable, sqlx only accepts flat structs
#[derive(Deserialize, FromRow, Serialize)]
pub struct ExpenseResponse {
    pub id: Uuid,
    pub amount: Decimal,
    pub description: String,
    pub user_id: Uuid,
    pub category_id: Uuid,
    pub date: NaiveDate,
    pub payment_method: Option<String>,
    pub is_recurring: bool,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize)]
pub struct ExpensesTotal {
    pub expenses: Vec<ExpenseResponse>,
    pub total: Decimal,
}

#[derive(Deserialize, Serialize)]
pub struct ExpensesTotalCached {
    pub expenses_total: ExpensesTotal,
    pub cached: bool,
}

#[derive(Deserialize)]
pub struct PageParams {
    #[serde(default = "default_page")]
    pub page: i64,
}

fn default_page() -> i64 {
    1
}

#[derive(Deserialize)]
pub struct ExpensePath {
    pub expense_id: Uuid,
}

#[derive(Deserialize)]
pub struct CategoryIdPath {
    pub category_id: Uuid,
}

#[derive(Deserialize, Serialize)]
pub struct ExpenseCached {
    pub cached: bool,
    pub expense: ExpenseResponse,
}
