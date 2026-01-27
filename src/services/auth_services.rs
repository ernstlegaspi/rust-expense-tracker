use anyhow::Context;
use bcrypt::{DEFAULT_COST, hash};
use sqlx::PgPool;
use std::result::Result;

use crate::{
    errors::auth_errors::AuthError,
    models::auth_models::{AuthResponse, LoginQuery, LoginRequest, RegisterRequest, UserQuery},
    services::{jwt_services::JwtService, redis_services::RedisService},
    utils::utils::create_uuid,
};

#[derive(Clone)]
pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(
        &self,
        body: RegisterRequest,
        jwt: &JwtService,
        redis: &RedisService,
    ) -> Result<AuthResponse, AuthError> {
        body.validate()?;

        let hashed_password =
            hash(body.password, DEFAULT_COST).context("failed to hash password")?;

        let new_user = sqlx::query_as::<_, UserQuery>(
            r#"
                INSERT INTO users (email, name, password)
                VALUES ($1, $2, $3)
                RETURNING email, id
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

            AuthError::internal(e)
        })?;

        let refresh_token_jti = create_uuid();
        let sub = new_user.id;

        redis
            .set(
                format!("user:{}:refresh:{refresh_token_jti}", sub),
                &refresh_token_jti,
                60 * 60 * 24 * 7,
            )
            .await
            .map_err(AuthError::internal)?;

        let token = jwt.create_access_token(sub).map_err(AuthError::internal)?;
        let refresh_token = jwt
            .create_refresh_token(&refresh_token_jti, sub)
            .map_err(AuthError::internal)?;

        Ok(AuthResponse {
            email: new_user.email,
            refresh_token,
            token,
        })
    }

    pub async fn login(
        &self,
        body: LoginRequest,
        jwt: &JwtService,
        redis: &RedisService,
    ) -> Result<AuthResponse, AuthError> {
        body.validate()?;

        let user = sqlx::query_as::<_, LoginQuery>(
            r#"
                SELECT email, id, password FROM users
                WHERE email = $1
            "#,
        )
        .bind(body.email)
        .fetch_one(&self.pool)
        .await;

        let user = user.map_err(|e| {
            if let sqlx::Error::RowNotFound = &e {
                return AuthError::InvalidCredentials;
            }

            AuthError::internal(e)
        })?;

        if !bcrypt::verify(body.password, &user.password)
            .context("failed to verify password hash")?
        {
            return Err(AuthError::InvalidCredentials)?;
        }

        let refresh_token_jti = create_uuid();
        let sub = user.id;

        redis
            .set(
                format!("user:{}:refresh:{refresh_token_jti}", sub),
                &refresh_token_jti,
                60 * 60 * 24 * 7,
            )
            .await
            .map_err(AuthError::internal)?;

        let token = jwt.create_access_token(sub).map_err(AuthError::internal)?;
        let refresh_token = jwt
            .create_refresh_token(&refresh_token_jti, sub)
            .map_err(AuthError::internal)?;

        Ok(AuthResponse {
            email: user.email,
            refresh_token,
            token,
        })
    }

    pub async fn refresh(
        &self,
        cookie: &str,
        jwt: &JwtService,
        redis: &RedisService,
    ) -> Result<AuthResponse, AuthError> {
        let claims = jwt
            .validate_refresh_token(cookie)
            .map_err(|_| return AuthError::Unauthorized)?;

        let exists = redis
            .exists(&format!("user:{}:refresh:{}", claims.sub, claims.jti))
            .await
            .context("failed to check refresh token")?;

        if !exists {
            return Err(AuthError::Unauthorized);
        }

        let user = sqlx::query_as::<_, UserQuery>(
            r#"
                SELECT email, id FROM users
                WHERE id = $1
            "#,
        )
        .bind(claims.sub)
        .fetch_one(&self.pool)
        .await;

        let user = user.map_err(|e| {
            if let sqlx::Error::RowNotFound = &e {
                return AuthError::InvalidCredentials;
            }

            AuthError::internal(e)
        })?;

        let jti = create_uuid();
        let sub = user.id;

        let token = jwt.create_access_token(sub).map_err(AuthError::internal)?;
        let refresh_token = jwt
            .create_refresh_token(&jti, sub)
            .map_err(AuthError::internal)?;

        redis
            .set(
                format!("user:{}:refresh:{jti}", sub),
                &jti,
                60 * 60 * 24 * 7,
            )
            .await
            .map_err(AuthError::internal)?;

        redis
            .revoke(&format!("user:{}:refresh:{}", claims.sub, claims.jti))
            .await
            .context("internal server error")?;

        Ok(AuthResponse {
            email: user.email,
            refresh_token,
            token,
        })
    }
}
