use anyhow::Context;
use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;
use std::result::Result;

use crate::{
    errors::errors::{AuthError, ValidationError},
    models::auth_models::{AuthResponse, Login, LoginResponse, RefreshResponse, Register},
    services::{jwt_services::JwtService, redis_services::RedisService},
};

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, body: Register) -> Result<AuthResponse, AuthError> {
        body.validate()?;

        let hashed_password =
            hash(body.password, DEFAULT_COST).context("failed to hash password")?;

        let new_user = sqlx::query_as::<_, AuthResponse>(
            r#"
                INSERT INTO users (email, name, password)
                VALUES ($1, $2, $3)
                RETURNING email, id, name
            "#,
        )
        .bind(body.email)
        .bind(body.name)
        .bind(hashed_password)
        .fetch_one(&self.pool)
        .await;

        let new_user = new_user.map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().as_deref() == Some("23505") {
                    return AuthError::DuplicateEmail;
                }
            }

            AuthError::Internal(anyhow::Error::from(e))
        })?;

        Ok(new_user)
    }

    pub async fn login(&self, body: Login) -> Result<AuthResponse, AuthError> {
        body.validate()?;

        let user = sqlx::query_as::<_, LoginResponse>(
            r#"
                SELECT email, id, name, password FROM users
                WHERE email = $1
            "#,
        )
        .bind(body.email)
        .fetch_one(&self.pool)
        .await;

        let user = user.map_err(|e| {
            if let sqlx::Error::RowNotFound = &e {
                return AuthError::UserNotFound;
            }

            AuthError::Internal(anyhow::Error::from(e))
        })?;

        if !bcrypt::verify(body.password, &user.password)
            .context("failed to verify password hash")?
        {
            return Err(ValidationError::WrongPassword)?;
        }

        Ok(AuthResponse {
            email: user.email,
            id: user.id,
            name: user.name,
        })
    }

    pub async fn refresh(
        &self,
        cookie: &str,
        jwt: &JwtService,
        redis: &RedisService,
    ) -> Result<RefreshResponse, AuthError> {
        let claims = jwt
            .validate_refresh_token(cookie)
            .map_err(|_| return AuthError::Unauthorized)?;

        let exists = redis
            .exists(format!("user:{}", claims.jti).as_str())
            .await
            .context("failed to check refresh token")?;

        if !exists {
            return Err(AuthError::Unauthorized);
        }

        let user = sqlx::query_as::<_, RefreshResponse>(
            r#"
                SELECT uuid FROM users
                WHERE uuid = $1
            "#,
        )
        .bind(claims.sub)
        .fetch_one(&self.pool)
        .await;

        let user = user.map_err(|e| {
            if let sqlx::Error::RowNotFound = &e {
                return AuthError::UserNotFound;
            }

            AuthError::Internal(anyhow::Error::from(e))
        })?;

        redis
            .revoke(format!("user:{}", claims.jti))
            .await
            .context("internal server error")?;

        Ok(user)
    }
}
