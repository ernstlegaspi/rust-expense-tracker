use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode, errors::Error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct RefreshTokenClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenClaims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn create_access_token(&self, sub: Uuid) -> Result<String, Error> {
        let claims = TokenClaims {
            sub,
            exp: (Utc::now() + Duration::minutes(15)).timestamp(),
            iat: Utc::now().timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn create_refresh_token(&self, jti: String, sub: Uuid) -> Result<String, Error> {
        let claims = RefreshTokenClaims {
            sub,
            exp: (Utc::now() + Duration::days(7)).timestamp(),
            iat: Utc::now().timestamp(),
            jti,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }
}
