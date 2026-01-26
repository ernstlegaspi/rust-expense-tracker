use actix_web::{
    HttpRequest, HttpResponse, Responder, ResponseError,
    cookie::{Cookie, SameSite, time::Duration},
    web::{Data, Json},
};
use serde_json::json;

use crate::{
    models::auth_models::{LoginRequest, RegisterRequest},
    services::{
        auth_services::AuthService, jwt_services::JwtService, redis_services::RedisService,
    },
};

pub async fn register(
    new_user_body: Json<RegisterRequest>,
    auth: Data<AuthService>,
    jwt: Data<JwtService>,
    redis: Data<RedisService>,
) -> impl Responder {
    let response = match auth
        .register(new_user_body.into_inner(), &jwt, &redis)
        .await
    {
        Ok(user) => user,
        Err(e) => return e.error_response(),
    };

    HttpResponse::Created()
        .cookie(set_cookie_token(response.token))
        .cookie(set_cookie_refresh_token(response.refresh_token))
        .json(json!({
            "email": response.email
        }))
}

pub async fn login(
    user: Json<LoginRequest>,
    auth: Data<AuthService>,
    jwt: Data<JwtService>,
    redis: Data<RedisService>,
) -> impl Responder {
    let response = match auth.login(user.into_inner(), &jwt, &redis).await {
        Ok(u) => u,
        Err(e) => return e.error_response(),
    };

    HttpResponse::Ok()
        .cookie(set_cookie_token(response.token))
        .cookie(set_cookie_refresh_token(response.refresh_token))
        .json(json!({
            "email": response.email,
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
        None => {
            return HttpResponse::Unauthorized().json(json!({
                "message": "unauthorized"
            }));
        }
    };

    let response = match auth.refresh(cookie.value(), &jwt, &redis).await {
        Ok(u) => u,
        Err(e) => return e.error_response(),
    };

    HttpResponse::Ok()
        .cookie(set_cookie_token(response.token))
        .cookie(set_cookie_refresh_token(response.refresh_token))
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
