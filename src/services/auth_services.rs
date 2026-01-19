use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;
use std::result::Result;

use crate::{
    errors::errors::{LoginError, RegisterError},
    models::auth_model::{Login, LoginResponse, Register, RegisterResponse},
};

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, body: Register) -> Result<RegisterResponse, RegisterError> {
        body.validate()?;

        let hashed_password = match hash(body.password, DEFAULT_COST) {
            Ok(hp) => hp,
            Err(_) => {
                return Err(RegisterError::Internal(
                    "Failed to hash password.".to_string(),
                ));
            }
        };

        let new_user = sqlx::query_as::<_, RegisterResponse>(
            r#"
                INSERT INTO users (email, name, password)
                VALUES ($1, $2, $3)
                RETURNING email, name, uuid
            "#,
        )
        .bind(body.email)
        .bind(body.name)
        .bind(hashed_password)
        .fetch_one(&self.pool)
        .await;

        let new_user = match new_user {
            Ok(user) => user,
            Err(e) => match e {
                sqlx::Error::Database(db_err) if db_err.code().as_deref() == Some("23505") => {
                    return Err(RegisterError::DuplicateEmail);
                }
                _ => return Err(RegisterError::Internal(e.to_string())),
            },
        };

        Ok(new_user)
    }

    pub async fn login(&self, body: Login) -> Result<LoginResponse, LoginError> {
        match body.validate() {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        let user = sqlx::query_as::<_, LoginResponse>(
            r#"
                SELECT email, name, password, uuid FROM users
                WHERE email = $1
            "#,
        )
        .bind(body.email)
        .fetch_one(&self.pool)
        .await;

        let user = match user {
            Ok(u) => u,
            Err(sqlx::Error::RowNotFound) => return Err(LoginError::UserNotFound),
            Err(_) => return Err(LoginError::Internal("Internal Server Error.".to_string())),
        };

        match bcrypt::verify(body.password, &user.password) {
            Ok(true) => (),
            Ok(false) => return Err(LoginError::WrongPassword),
            Err(_) => return Err(LoginError::Internal("Invalid hash format".to_string())),
        };

        Ok(user)
    }
}
