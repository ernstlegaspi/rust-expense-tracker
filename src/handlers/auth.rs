use actix_web::{
    HttpRequest, HttpResponse, Responder,
    cookie::{Cookie, SameSite, time::Duration},
    web::{Data, Json},
};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::errors::errors::{LoginError, RegisterError, e400, e401, e404, e409, e500};
use crate::services::{
    auth_services::AuthService, jwt_services::JwtService, redis_services::RedisService,
};
use crate::{
    errors::errors::RefreshEndpointError,
    models::auth_model::{Login, Register},
};

pub async fn register(
    new_user_body: Json<Register>,
    jwt: Data<JwtService>,
    service: Data<AuthService>,
    redis: Data<RedisService>,
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

    let refresh_token = match jwt.create_refresh_token(&refresh_token_jti, sub) {
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
    user: Json<Login>,
    jwt: Data<JwtService>,
    service: Data<AuthService>,
    redis: Data<RedisService>,
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

    let refresh_token = match jwt.create_refresh_token(&refresh_token_jti, sub) {
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

pub async fn refresh(
    req: HttpRequest,
    auth: Data<AuthService>,
    jwt: Data<JwtService>,
    redis: Data<RedisService>,
) -> impl Responder {
    let cookie = match req.cookie("refresh_token") {
        Some(c) => c,
        None => return e401("Unauthorized"),
    };

    let user = match auth
        .refresh(cookie.value(), jwt.get_ref(), redis.get_ref())
        .await
    {
        Ok(u) => u,
        Err(e) => match e {
            RefreshEndpointError::BadRequest => return e400("Bad request"),
            RefreshEndpointError::Internal(msg) => return e500(&msg),
            RefreshEndpointError::NotFound => return e404("Not found"),
            RefreshEndpointError::Unauthorized => return e401("Unauthorized."),
        },
    };

    let jti = uuid::Uuid::new_v4().to_string();
    let sub = user.uuid;

    let token = match jwt.create_access_token(sub) {
        Ok(token) => token,
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    };

    let refresh_token = match jwt.create_refresh_token(&jti, sub) {
        Ok(token) => token,
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    };

    match redis
        .set(format!("user:{jti}"), &jti, 60 * 60 * 24 * 7)
        .await
    {
        Ok(()) => (),
        Err(e) => {
            error!(error = ?e);

            return e500("Internal Server Error.");
        }
    }

    HttpResponse::Ok()
        .cookie(set_cookie_token(token))
        .cookie(set_cookie_refresh_token(refresh_token))
        .json(json!({
            "message": "Token refreshed."
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
        .path("/api/user/refresh")
        .same_site(SameSite::Strict)
        .secure(true)
        .max_age(Duration::days(7))
        .finish()
}
