use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;

use crate::{
    errors::errors::UserError,
    models::user_model::{CreateUser, UserResponse},
};

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
    ) -> std::result::Result<UserResponse, UserError> {
        match new_user.validate() {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        let hashed_password = match hash(new_user.password, DEFAULT_COST) {
            Ok(hp) => hp,
            Err(e) => return Err(UserError::Internal(e.to_string())),
        };

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

        let new_user = match new_user {
            Ok(user) => user,
            Err(e) => match e {
                sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                    return Err(UserError::DuplicateEmail);
                }
                _ => return Err(UserError::Internal(e.to_string())),
            },
        };

        Ok(new_user)
    }
}
