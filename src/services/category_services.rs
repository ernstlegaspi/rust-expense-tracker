use sqlx::PgPool;

use crate::{
    errors::category_errors::CategoryError,
    models::category_models::{AddCategoryResponse, Category},
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
        user_id: uuid::Uuid,
    ) -> Result<AddCategoryResponse, CategoryError> {
        body.validate()?;

        let category = sqlx::query_as::<_, AddCategoryResponse>(
            r#"
                INSERT INTO category (name, user_id)
                VALUES ($1, $2)
                RETURNING *;
            "#,
        )
        .bind(body.name)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await;

        let category = category.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return CategoryError::NameExisting;
                }
            }

            CategoryError::Internal(anyhow::Error::from(e))
        })?;

        Ok(category)
    }
}
