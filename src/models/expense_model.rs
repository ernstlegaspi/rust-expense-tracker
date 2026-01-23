use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::errors::expense_errors::ExpenseError;

#[derive(Deserialize)]
pub struct AddExpenseRequest {
    pub amount: Decimal,
    pub description: String,
    pub category_id: Uuid,
    pub date: NaiveDate,
    pub is_recurring: bool,
    pub tags: Option<Vec<String>>,
}

impl AddExpenseRequest {
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

#[derive(Deserialize, FromRow, Serialize)]
pub struct ExpenseResponse {
    id: Uuid,
    amount: Decimal,
    description: String,
    user_id: Uuid,
    category_id: Uuid,
    date: NaiveDate,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    payment_method: Option<String>,
    is_recurring: bool,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct QueryParams {
    #[serde(default = "default_page")]
    pub page: i64,
}

fn default_page() -> i64 {
    1
}
