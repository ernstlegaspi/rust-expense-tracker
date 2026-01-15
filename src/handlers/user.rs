use actix_web::{HttpResponse, Responder, web};

use crate::models::user_model::CreateUser;
use crate::services::{jwt_services::JwtService, user_services::UserService};

pub async fn create_user(
    body: web::Json<CreateUser>,
    jwt: web::Data<JwtService>,
    service: web::Data<UserService>,
) -> impl Responder {
    let user = match service.create_user(body.into_inner()).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::BadRequest().body("Bad request."),
    };

    let jti = uuid::Uuid::new_v4().to_string();
    let sub = user.uuid;

    let at = match jwt.create_token(15 * 60, jti, sub) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().body("Token generation failed"),
    };

    HttpResponse::Created().json(serde_json::json!({
        "email": user.email,
        "name": user.name,
        "token": at
    }))
}
