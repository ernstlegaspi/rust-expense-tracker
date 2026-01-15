use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;

use crate::models::user_model::{CreateUser, UserResponse};

#[derive(Clone)]
pub struct UserService {
    pool: PgPool,
}

impl UserService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(
        &self,
        new_user: CreateUser,
    ) -> std::result::Result<UserResponse, actix_web::Error> {
        new_user.validate().map_err(|e| ErrorBadRequest(e))?;

        let hashed_password =
            hash(new_user.password, DEFAULT_COST).map_err(|e| ErrorInternalServerError(e))?;

        let new_user = sqlx::query_as::<_, UserResponse>(
            r#"
                INSERT INTO users (email, name, password)
                VALUES ($1, $2, $3)
                RETURNING email, name, uuid
            "#,
        )
        .bind(new_user.email)
        .bind(new_user.name)
        .bind(hashed_password)
        .fetch_one(&self.pool)
        .await;

        new_user.map_err(ErrorInternalServerError)
    }
}
