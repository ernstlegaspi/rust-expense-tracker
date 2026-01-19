use actix_web::{
    HttpResponse, Responder,
    cookie::{Cookie, SameSite, time::Duration},
    web,
};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::errors::errors::{LoginError, RegisterError, e400, e404, e409, e500};
use crate::models::auth_model::{Login, Register};
use crate::services::{
    auth_services::AuthService, jwt_services::JwtService, redis_services::RedisService,
};

pub async fn register(
    new_user_body: web::Json<Register>,
    jwt: web::Data<JwtService>,
    service: web::Data<AuthService>,
    redis: web::Data<RedisService>,
) -> impl Responder {
    let user = match service.register(new_user_body.into_inner()).await {
        Ok(user) => user,
        Err(e) => {
            error!(error = ?e);

            match e {
                RegisterError::DuplicateEmail => {
                    return e409("Email is already existing");
                }
                RegisterError::Internal(msg) => return e500(&msg),
                RegisterError::InvalidEmail => return e400("Please enter a valid email."),
                RegisterError::InvalidNameLength => {
                    return e400("Name must be at least 3 characters.");
                }
                RegisterError::NameRequired => return e400("Name field is required."),
                RegisterError::PasswordTooLong => return e400("Password too long."),
                RegisterError::WeakPassword => return e400("Your password is too weak."),
            }
        }
    };

    let refresh_token_jti = Uuid::new_v4().to_string();
    let sub = user.uuid;

    match redis
        .set(
            format!("user:{refresh_token_jti}"),
            &refresh_token_jti,
            60 * 60 * 24 * 7,
        )
        .await
    {
        Ok(()) => (),
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    }

    let token = match jwt.create_access_token(sub) {
        Ok(token) => token,
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    };

    let refresh_token = match jwt.create_refresh_token(refresh_token_jti, sub) {
        Ok(token) => token,
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    };

    HttpResponse::Created()
        .cookie(set_cookie_token(token))
        .cookie(set_cookie_refresh_token(refresh_token))
        .json(json!({
            "email": user.email,
            "name": user.name,
        }))
}

pub async fn login(
    user: web::Json<Login>,
    jwt: web::Data<JwtService>,
    service: web::Data<AuthService>,
    redis: web::Data<RedisService>,
) -> impl Responder {
    let user = match service.login(user.into_inner()).await {
        Ok(u) => u,
        Err(e) => {
            error!(error = ?e);

            match e {
                LoginError::Internal(msg) => return e500(&msg),
                LoginError::InvalidEmail => return e400("Please enter a valid email."),
                LoginError::PasswordRequired => return e400("Password is required."),
                LoginError::UserNotFound => return e404("User not found."),
                LoginError::WeakPassword => return e400("Password must be at least 8 characters."),
                LoginError::WrongPassword => return e400("Incorrect password."),
            }
        }
    };

    let refresh_token_jti = Uuid::new_v4().to_string();
    let sub = user.uuid;

    match redis
        .set(
            format!("user:{refresh_token_jti}"),
            &refresh_token_jti,
            60 * 60 * 24 * 7,
        )
        .await
    {
        Ok(()) => (),
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    }

    let token = match jwt.create_access_token(sub) {
        Ok(token) => token,
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    };

    let refresh_token = match jwt.create_refresh_token(refresh_token_jti, sub) {
        Ok(token) => token,
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    };

    HttpResponse::Ok()
        .cookie(set_cookie_token(token))
        .cookie(set_cookie_refresh_token(refresh_token))
        .json(json!({
            "email": user.email,
            "name": user.name,
        }))
}

fn set_cookie_token<'l>(token: String) -> Cookie<'l> {
    Cookie::build("token", token)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(Duration::minutes(15))
        .finish()
}

fn set_cookie_refresh_token<'l>(refresh_token: String) -> Cookie<'l> {
    Cookie::build("refresh_token", refresh_token)
        .http_only(true)
        .path("/api/refresh")
        .same_site(SameSite::Strict)
        .secure(true)
        .max_age(Duration::days(7))
        .finish()
}
