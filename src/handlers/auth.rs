use actix_web::{
    HttpRequest, HttpResponse, Responder, ResponseError,
    cookie::{Cookie, SameSite, time::Duration},
    web::{Data, Json},
};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::errors::errors::{e401, e500};
use crate::models::auth_models::{Login, Register};
use crate::services::{
    auth_services::AuthService, jwt_services::JwtService, redis_services::RedisService,
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

            return e.error_response();
        }
    };

    let refresh_token_jti = Uuid::new_v4().to_string();
    let sub = user.id;

    match redis
        .set(
            format!("user:{}:refresh:{refresh_token_jti}", sub),
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

            return e.error_response();
        }
    };

    let refresh_token_jti = Uuid::new_v4().to_string();
    let sub = user.id;

    match redis
        .set(
            format!("user:{}:refresh:{refresh_token_jti}", sub),
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
        Err(e) => {
            error!(error = ?e);

            return e.error_response();
        }
    };

    let jti = uuid::Uuid::new_v4().to_string();
    let sub = user.id;

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
        .set(
            format!("user:{}:refresh:{jti}", sub),
            &jti,
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
        .max_age(Duration::seconds(15))
        // .max_age(Duration::minutes(15))
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
