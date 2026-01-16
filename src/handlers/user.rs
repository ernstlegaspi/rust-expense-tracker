use actix_web::{HttpResponse, Responder, web};
use tracing::error;

use crate::errors::errors::{UserError, error_response};
use crate::models::user_model::CreateUser;
use crate::services::{jwt_services::JwtService, user_services::UserService};

pub async fn create_user(
    new_user_body: web::Json<CreateUser>,
    jwt: web::Data<JwtService>,
    service: web::Data<UserService>,
) -> impl Responder {
    let user = match service.create_user(new_user_body.into_inner()).await {
        Ok(user) => user,
        Err(e) => {
            error!(error = ?e);

            match e {
                UserError::DuplicateEmail => return error_response("Email is already existing"),
                UserError::Internal(msg) => return error_response(&msg),
                UserError::InvalidEmail => return error_response("Please enter a valid email."),
                UserError::InvalidNameLength => {
                    return error_response("Name must be at least 3 characters.");
                }
                UserError::NameRequired => return error_response("Name field is required."),
                UserError::WeakPassword => return error_response("Your password is too weak."),
            }
        }
    };

    let jti = uuid::Uuid::new_v4().to_string();
    let sub = user.uuid;

    let access_token = match jwt.create_token(15 * 60, jti, sub) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().body("Token generation failed"),
    };

    HttpResponse::Created().json(serde_json::json!({
        "email": user.email,
        "name": user.name,
        "token": access_token
    }))
}
