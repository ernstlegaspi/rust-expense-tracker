use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::expense_errors::ExpenseError,
    models::expense_model::{
        AddExpenseRequest, EditExpenseRequest, ExpenseCached, ExpensePath, ExpenseResponse,
        ExpensesTotal, ExpensesTotalCached, PageParams,
    },
    services::redis_services::RedisService,
    utils::utils::{all_expenses_version_key, single_expense_key, total_expense_key},
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
        user_id: Uuid,
    ) -> Result<ExpenseResponse, ExpenseError> {
        expense.validate()?;

        let mut tx = self.pool.begin().await.map_err(ExpenseError::internal)?;

        let expense = query_as::<_, ExpenseResponse>(
            r#"
                INSERT INTO expense (amount, description, user_id, category_id, date, is_recurring, tags)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, amount, description, user_id, category_id, date, payment_method, is_recurring, tags;
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
                match db_err.code().as_deref() {
                    Some("23502") => return ExpenseError::RequiredFieldMissing,
                    Some("23503") => return ExpenseError::ForeignKeyNotFound,
                    _ => return ExpenseError::internal(e),
                }
            }

            ExpenseError::internal(e)
        })?;

        redis
            .pipeline::<()>(|pipe| {
                pipe.incr(all_expenses_version_key(user_id), 1)
                    .del(total_expense_key(user_id));
            })
            .await
            .map_err(ExpenseError::internal)?;

        tx.commit().await.map_err(ExpenseError::internal)?;

        Ok(expense)
    }

    pub async fn get_user_expenses(
        &self,
        params: PageParams,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<ExpensesTotalCached, ExpenseError> {
        let limit: i64 = 10;
        let page = params.page.max(1);
        let offset = (page - 1) * limit;

        let v_key = all_expenses_version_key(user_id);

        let (_, v): (i64, String) = redis
            .pipeline(|pipe| {
                pipe.set_nx(&v_key, "1").get(&v_key);
            })
            .await
            .map_err(ExpenseError::internal)?;

        let key = format!("user:{}:p:{}:v:{}:expenses", user_id, page, v);

        if let Some(cached) = redis.get(&key).await.ok().flatten() {
            let cached_json = serde_json::from_str(&cached).map_err(ExpenseError::internal)?;

            return Ok(ExpensesTotalCached {
                expenses_total: cached_json,
                cached: true,
            });
        }

        let expenses = query_as::<_, ExpenseResponse>(
            r#"

                SELECT id, amount, description, user_id,
                    category_id, date, payment_method,
                    is_recurring, tags FROM expense
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

        let total: Decimal = query_scalar(
            r#"
                SELECT COALESCE(SUM(amount), 0) FROM expense
                WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(ExpenseError::internal)?;

        let result = ExpensesTotal { expenses, total };

        let json = serde_json::to_string(&result).map_err(ExpenseError::internal)?;

        redis
            .set(key, json, 300)
            .await
            .map_err(ExpenseError::internal)?;

        Ok(ExpensesTotalCached {
            expenses_total: result,
            cached: false,
        })
    }

    pub async fn get_single_expense_per_user(
        &self,
        path: ExpensePath,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<ExpenseCached, ExpenseError> {
        let key = single_expense_key(user_id, path.expense_id);

        if let Some(cached) = redis.get(&key).await.ok().flatten() {
            let expense = serde_json::from_str(&cached).map_err(ExpenseError::internal)?;

            return Ok(ExpenseCached {
                cached: true,
                expense,
            });
        }

        let expense = query_as::<_, ExpenseResponse>(
            r#"
                SELECT id, amount, description, user_id,
                    category_id, date, payment_method,
                    is_recurring, tags FROM expense
                WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(path.expense_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await;

        let expense = expense.map_err(|e| {
            if let sqlx::Error::RowNotFound = &e {
                return ExpenseError::ExpenseNotFound;
            }

            ExpenseError::internal(e)
        })?;

        let json = serde_json::to_string(&expense).map_err(ExpenseError::internal)?;

        redis
            .set(key, json, 300)
            .await
            .map_err(ExpenseError::internal)?;

        Ok(ExpenseCached {
            cached: false,
            expense,
        })
    }

    pub async fn edit_expense_per_user(
        &self,
        body: EditExpenseRequest,
        path: ExpensePath,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<ExpenseResponse, ExpenseError> {
        let mut tx = self.pool.begin().await.map_err(ExpenseError::internal)?;

        let expense = query_as::<_, ExpenseResponse>(
            r#"
                UPDATE expense
                SET amount = $3, description = $4,
                    category_id = $5, date = $6,
                    updated_at = NOW(), payment_method = $7,
                    is_recurring = $8, tags = $9
                WHERE id = $1 AND user_id = $2
                RETURNING id, amount, description, user_id,
                    category_id, date, payment_method,
                    is_recurring, tags
            "#,
        )
        .bind(path.expense_id)
        .bind(user_id)
        .bind(body.amount)
        .bind(body.description)
        .bind(body.category_id)
        .bind(body.date)
        .bind(body.payment_method)
        .bind(body.is_recurring)
        .bind(body.tags)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ExpenseError::internal)?
        .ok_or(ExpenseError::ExpenseNotFound)?;

        redis
            .pipeline::<()>(|pipe| {
                pipe.del(single_expense_key(path.expense_id, user_id))
                    .incr(all_expenses_version_key(user_id), 1)
                    .del(total_expense_key(user_id));
            })
            .await
            .map_err(ExpenseError::internal)?;

        tx.commit().await.map_err(ExpenseError::internal)?;

        Ok(expense)
    }

    pub async fn delete_expense_per_use(
        &self,
        path: ExpensePath,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<String, ExpenseError> {
        let mut tx = self.pool.begin().await.map_err(ExpenseError::internal)?;

        let id: Uuid = query_scalar(
            r#"
                DELETE FROM expense
                WHERE id = $1 AND user_id = $2
                RETURNING id
            "#,
        )
        .bind(path.expense_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ExpenseError::internal)?
        .ok_or(ExpenseError::ExpenseNotFound)?;

        redis
            .pipeline::<()>(|pipe| {
                pipe.del(single_expense_key(path.expense_id, user_id))
                    .incr(all_expenses_version_key(user_id), 1)
                    .del(total_expense_key(user_id));
            })
            .await
            .map_err(ExpenseError::internal)?;

        tx.commit().await.map_err(ExpenseError::internal)?;

        Ok(id.to_string())
    }

    pub async fn get_total_of_all_expenses(
        &self,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<String, ExpenseError> {
        let key = total_expense_key(user_id);

        if let Some(cached) = redis.get(&key).await.ok().flatten() {
            return Ok(cached);
        }

        let total: Decimal = query_scalar(
            r#"
                SELECT COALESCE(SUM(amount), 0) FROM expense
                WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(ExpenseError::internal)?;

        let total = total.to_string();

        redis
            .set(key, &total, 300)
            .await
            .map_err(ExpenseError::internal)?;

        Ok(total)
    }
}
