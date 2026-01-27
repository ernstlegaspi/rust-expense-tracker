use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    errors::expense_errors::ExpenseError,
    models::expense_model::{
        CategoryIdPath, ExpenseCached, ExpensePath, ExpenseRequest, ExpenseResponse, ExpensesTotal,
        ExpensesTotalCached, PageParams,
    },
    services::redis_services::RedisService,
    utils::utils::{
        all_expenses_version_key, category_filter_expenses_version_key,
        category_filter_total_expense_key, single_expense_key, total_expense_key,
    },
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

    async fn invalidate_expense_cache(
        &self,
        redis: &RedisService,
        category_id: Uuid,
        user_id: Uuid,
        expense_id: Option<Uuid>,
    ) -> Result<(), ExpenseError> {
        redis
            .pipeline(|pipe| {
                if let Some(id) = expense_id {
                    pipe.del(single_expense_key(id, user_id));
                }

                pipe.incr(all_expenses_version_key(user_id), 1)
                    .del(total_expense_key(user_id))
                    .incr(
                        category_filter_expenses_version_key(category_id, user_id),
                        1,
                    )
                    .del(category_filter_total_expense_key(category_id, user_id));
            })
            .await
            .map_err(ExpenseError::internal)
    }

    pub async fn add_expense(
        &self,
        expense: ExpenseRequest,
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

        self.invalidate_expense_cache(redis, expense.category_id, user_id, None)
            .await?;

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

        let total: Decimal =
            if let Some(v) = redis.get(&total_expense_key(user_id)).await.ok().flatten() {
                Decimal::from_str(&v).map_err(ExpenseError::internal)?
            } else {
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

                redis
                    .set(total_expense_key(user_id), total.to_string(), 300)
                    .await
                    .map_err(ExpenseError::internal)?;

                total
            };

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
        let key = single_expense_key(path.expense_id, user_id);

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
        body: ExpenseRequest,
        path: ExpensePath,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<ExpenseResponse, ExpenseError> {
        body.validate()?;

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

        self.invalidate_expense_cache(redis, expense.category_id, user_id, Some(path.expense_id))
            .await?;

        tx.commit().await.map_err(ExpenseError::internal)?;

        Ok(expense)
    }

    pub async fn delete_expense_per_user(
        &self,
        path: ExpensePath,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<String, ExpenseError> {
        let mut tx = self.pool.begin().await.map_err(ExpenseError::internal)?;

        let (id, category_id): (Uuid, Uuid) = query_as(
            r#"
                DELETE FROM expense
                WHERE id = $1 AND user_id = $2
                RETURNING id, category_id
            "#,
        )
        .bind(path.expense_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(ExpenseError::internal)?
        .ok_or(ExpenseError::ExpenseNotFound)?;

        self.invalidate_expense_cache(redis, category_id, user_id, Some(path.expense_id))
            .await?;

        tx.commit().await.map_err(ExpenseError::internal)?;

        Ok(id.to_string())
    }

    pub async fn get_total_of_all_expenses(
        &self,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<Decimal, ExpenseError> {
        let key = total_expense_key(user_id);

        if let Some(cached) = redis.get(&key).await.ok().flatten() {
            let amount = Decimal::from_str(&cached).map_err(ExpenseError::internal)?;

            return Ok(amount);
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

        redis
            .set(key, &total.to_string(), 300)
            .await
            .map_err(ExpenseError::internal)?;

        Ok(total)
    }

    pub async fn filter_expense_by_category_per_user(
        &self,
        params: PageParams,
        path: CategoryIdPath,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<ExpensesTotalCached, ExpenseError> {
        let page = params.page.max(1);
        let limit = 10;
        let offset = (page - 1) * limit;
        let category_id = path.category_id;

        let key = category_filter_expenses_version_key(category_id, user_id);

        let (_, v): (i64, String) = redis
            .pipeline(|pipe| {
                pipe.set_nx(&key, "1").get(&key);
            })
            .await
            .map_err(ExpenseError::internal)?;

        let key = format!(
            "user:{}:filter:category:{}:v:{}:p:{}",
            user_id, category_id, v, page
        );

        let total_key = category_filter_total_expense_key(category_id, user_id);

        let total: Decimal = if let Some(cached) = redis.get(&total_key).await.ok().flatten() {
            let total = Decimal::from_str(&cached).map_err(ExpenseError::internal)?;

            total
        } else {
            let total = query_scalar::<_, Decimal>(
                r#"
                    SELECT COALESCE(SUM(amount), 0) FROM expense
                    WHERE user_id = $1 AND category_id = $2
                "#,
            )
            .bind(user_id)
            .bind(path.category_id)
            .fetch_one(&self.pool)
            .await
            .map_err(ExpenseError::internal)?;

            redis
                .set(total_key, total.to_string(), 300)
                .await
                .map_err(ExpenseError::internal)?;

            total
        };

        if let Some(cached) = redis.get(&key).await.ok().flatten() {
            let json: Vec<ExpenseResponse> =
                serde_json::from_str(&cached).map_err(ExpenseError::internal)?;

            let expenses_total = ExpensesTotal {
                expenses: json,
                total,
            };

            return Ok(ExpensesTotalCached {
                expenses_total,
                cached: true,
            });
        }

        let expenses = query_as::<_, ExpenseResponse>(
            r#"
                SELECT id, amount, description, user_id,
                    category_id, date, payment_method,
                    is_recurring, tags FROM expense
                WHERE user_id = $1 AND category_id = $2
                ORDER BY updated_at DESC
                LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_id)
        .bind(path.category_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(ExpenseError::internal)?;

        let json = serde_json::to_string(&expenses).map_err(ExpenseError::internal)?;

        redis
            .set(key, json, 300)
            .await
            .map_err(ExpenseError::internal)?;

        let expenses_total = ExpensesTotal { expenses, total };

        Ok(ExpensesTotalCached {
            expenses_total,
            cached: false,
        })
    }
}
