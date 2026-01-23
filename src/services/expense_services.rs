use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::expense_errors::ExpenseError,
    models::expense_model::{AddExpenseRequest, ExpenseResponse, ExpensesWithTotal, QueryParams},
    services::redis_services::RedisService,
};

use sqlx::{query_as, query_scalar};

#[derive(Debug, Clone)]
pub struct ExpenseServices {
    pool: PgPool,
}

impl ExpenseServices {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_expense(
        &self,
        expense: AddExpenseRequest,
        redis: &RedisService,
        user_id: uuid::Uuid,
    ) -> Result<ExpenseResponse, ExpenseError> {
        expense.validate()?;

        let mut tx = self.pool.begin().await.map_err(ExpenseError::internal)?;

        let expense = query_as::<_, ExpenseResponse>(
            r#"
                INSERT INTO expense (amount, description, user_id, category_id, date, is_recurring, tags)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING *;
            "#,
        )
        .bind(expense.amount)
        .bind(expense.description)
        .bind(user_id)
        .bind(expense.category_id)
        .bind(expense.date)
        .bind(expense.is_recurring)
        .bind(expense.tags)
        .fetch_one(&mut *tx)
        .await;

        let expense = expense.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23502") {
                    return ExpenseError::RequiredFieldMissing;
                }
            }

            ExpenseError::Internal(anyhow::Error::from(e))
        })?;

        redis
            .incr(format!("user:{}:expenses:version", user_id))
            .await
            .map_err(ExpenseError::internal)?;

        tx.commit().await.map_err(ExpenseError::internal)?;

        Ok(expense)
    }

    pub async fn get_user_expenses(
        &self,
        params: QueryParams,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<ExpensesWithTotal, ExpenseError> {
        let limit: i64 = 10;
        let page = params.page.max(1);
        let offset = (page - 1) * limit;

        let v: i64 = redis
            .get(format!("user:{}:expenses:version", user_id))
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let key = format!("user:{}:p:{}:v:{}:expenses", user_id, page, v);

        if let Some(cached) = redis.get(key.clone()).await.ok().flatten() {
            let cached_json = serde_json::from_str(&cached).map_err(ExpenseError::internal)?;

            return Ok(cached_json);
        }

        let expenses = query_as::<_, ExpenseResponse>(
            r#"
                SELECT * FROM expense
                WHERE user_id = $1
                ORDER BY updated_at DESC
                LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(ExpenseError::internal)?;

        let total: rust_decimal::Decimal = query_scalar(
            r#"
                SELECT COALESCE(SUM(amount), 0) FROM expense
                WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(ExpenseError::internal)?;

        let result = ExpensesWithTotal { expenses, total };

        let json = serde_json::to_string(&result).map_err(ExpenseError::internal)?;

        redis
            .set(key, json, 300)
            .await
            .map_err(ExpenseError::internal)?;

        Ok(result)
    }
}
