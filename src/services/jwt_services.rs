use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode, errors::Error};
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

    fn create_token(&self, exp: Duration, jti: String, sub: Uuid) -> Result<String, Error> {
        let claims = Claims {
            sub,
            exp: (Utc::now() + exp).timestamp(),
            iat: Utc::now().timestamp(),
            jti,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&self.secret.as_bytes()),
        )
    }

    pub fn create_access_token(&self, jti: String, sub: Uuid) -> Result<String, Error> {
        self.create_token(Duration::minutes(15), jti, sub)
    }

    pub fn create_refresh_token(&self, jti: String, sub: Uuid) -> Result<String, Error> {
        self.create_token(Duration::days(7), jti, sub)
    }
}
