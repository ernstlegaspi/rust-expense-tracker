use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn create_token(
        &self,
        exp: i64,
        jti: String,
        sub: Uuid,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let claims = Claims {
            sub,
            // to do
            // dont forget to Duration::hours() for refresh tokens
            exp: (Utc::now() + Duration::minutes(exp)).timestamp(),
            iat: Utc::now().timestamp(),
            jti,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret.as_bytes()),
        )
    }
}
