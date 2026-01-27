use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    errors::category_errors::CategoryError,
    models::category_models::{CategoriesCached, Category, CategoryPagination, CategoryResponse},
    services::redis_services::RedisService,
    utils::utils::categories_version_key,
};

#[derive(Clone)]
pub struct CategoryService {
    pool: PgPool,
}

impl CategoryService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn add_category(
        &self,
        body: Category,
        redis: &RedisService,
        user_id: uuid::Uuid,
    ) -> Result<CategoryResponse, CategoryError> {
        body.validate()?;

        let mut tx = self.pool.begin().await.map_err(CategoryError::internal)?;

        let category = query_as::<_, CategoryResponse>(
            r#"
                INSERT INTO category (name, user_id)
                VALUES ($1, $2)
                RETURNING id, description, name, user_id
            "#,
        )
        .bind(body.name)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await;

        let category = category.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return CategoryError::NameExisting;
                }
            }

            CategoryError::internal(e)
        })?;

        redis
            .incr(&categories_version_key(user_id))
            .await
            .map_err(CategoryError::internal)?;

        tx.commit().await.map_err(CategoryError::internal)?;

        Ok(category)
    }

    pub async fn get_user_categories(
        &self,
        params: CategoryPagination,
        redis: &RedisService,
        user_id: Uuid,
    ) -> Result<CategoriesCached, CategoryError> {
        let limit = 10;
        let page = params.page.max(1);
        let offset = (page - 1) * limit;
        let key = categories_version_key(user_id);

        let (_, v): (i64, String) = redis
            .pipeline(|pipe| {
                pipe.set_nx(&key, "1").get(&key);
            })
            .await
            .map_err(CategoryError::internal)?;

        let key = format!("user:{}:v:{}:p:{}", user_id, v, page);

        if let Some(cached) = redis.get(&key).await.ok().flatten() {
            let json: Vec<CategoryResponse> =
                serde_json::from_str(&cached).map_err(CategoryError::internal)?;

            return Ok(CategoriesCached {
                cached: true,
                categories: json,
            });
        }

        let categories: Vec<CategoryResponse> = query_as(
            r#"
                SELECT id, description, name, user_id FROM category
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
        .map_err(CategoryError::internal)?;

        let json = serde_json::to_string(&categories).map_err(CategoryError::internal)?;

        redis
            .set(key, json, 300)
            .await
            .map_err(CategoryError::internal)?;

        Ok(CategoriesCached {
            cached: false,
            categories,
        })
    }
}
