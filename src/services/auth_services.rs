use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;
use std::result::Result;

use crate::{
    errors::errors::{LoginError, RefreshEndpointError, RegisterError},
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

    pub async fn register(&self, body: Register) -> Result<AuthResponse, RegisterError> {
        body.validate()?;

        let hashed_password = match hash(body.password, DEFAULT_COST) {
            Ok(hp) => hp,
            Err(_) => {
                return Err(RegisterError::Internal(
                    "Failed to hash password.".to_string(),
                ));
            }
        };

        let new_user = sqlx::query_as::<_, AuthResponse>(
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

    pub async fn login(&self, body: Login) -> Result<AuthResponse, LoginError> {
        body.validate()?;

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

        Ok(AuthResponse {
            email: user.email,
            name: user.name,
            uuid: user.uuid,
        })
    }

    pub async fn refresh(
        &self,
        cookie: &str,
        jwt: &JwtService,
        redis: &RedisService,
    ) -> Result<RefreshResponse, RefreshEndpointError> {
        let claims = match jwt.validate_refresh_token(cookie) {
            Ok(v) => v,
            Err(_) => return Err(RefreshEndpointError::Unauthorized),
        };

        match redis.exists(format!("user:{}", claims.jti).as_str()).await {
            Ok(v) => {
                if !v {
                    return Err(RefreshEndpointError::Unauthorized);
                }
            }
            Err(_) => return Err(RefreshEndpointError::Unauthorized),
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

        let user = match user {
            Ok(u) => u,
            Err(sqlx::Error::RowNotFound) => return Err(RefreshEndpointError::NotFound),
            Err(_) => {
                return Err(RefreshEndpointError::Internal(
                    "Internal Server Error.".to_string(),
                ));
            }
        };

        match redis.revoke(format!("user:{}", claims.jti)).await {
            Ok(()) => (),
            Err(_) => {
                return Err(RefreshEndpointError::Internal(
                    "Internal Server Error.".to_string(),
                ));
            }
        }

        Ok(user)
    }
}
