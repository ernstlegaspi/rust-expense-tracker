use actix_web::{HttpResponse, Responder, cookie, web};
use serde_json::json;
use tracing::error;
use uuid::Uuid;

use crate::errors::errors::{LoginError, RegisterError, e400, e404, e409, e500};
use crate::models::auth_model::{Login, Register};
use crate::services::{auth_services::AuthService, jwt_services::JwtService};

pub async fn register(
    new_user_body: web::Json<Register>,
    jwt: web::Data<JwtService>,
    service: web::Data<AuthService>,
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
                RegisterError::WeakPassword => return e400("Your password is too weak."),
            }
        }
    };

    let jti = Uuid::new_v4().to_string();
    let sub = user.uuid;

    let token = match jwt.create_token(15, jti, sub) {
        Ok(token) => token,
        Err(_) => return e500("Token generation failed"),
    };

    HttpResponse::Created()
        .cookie(
            cookie::Cookie::build("token", &token)
                .http_only(true)
                .secure(true)
                .same_site(cookie::SameSite::Strict)
                .path("/")
                .max_age(cookie::time::Duration::minutes(15))
                .finish(),
        )
        .json(json!({
            "email": user.email,
            "name": user.name,
        }))
}

pub async fn login(
    user: web::Json<Login>,
    jwt: web::Data<JwtService>,
    service: web::Data<AuthService>,
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

    let jti = Uuid::new_v4().to_string();
    let token = match jwt.create_token(15, jti, user.uuid) {
        Ok(t) => t,
        Err(_) => return e500("Token generation failed"),
    };

    HttpResponse::Ok()
        .cookie(
            cookie::Cookie::build("token", &token)
                .http_only(true)
                .secure(true)
                .same_site(cookie::SameSite::Strict)
                .path("/")
                .max_age(cookie::time::Duration::minutes(15))
                .finish(),
        )
        .json(json!({ "email": user.email }))
}
