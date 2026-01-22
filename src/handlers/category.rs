use actix_web::{
    HttpResponse, Responder, ResponseError,
    web::{Data, Json},
};
use tracing::error;

use crate::{
    middleware::auth::AuthMiddleware, models::category_models::Category,
    services::category_services::CategoryService,
};

pub async fn add_category(
    auth: AuthMiddleware,
    body: Json<Category>,
    service: Data<CategoryService>,
) -> impl Responder {
    let category = match service.add_category(body.into_inner(), auth.user_id).await {
        Ok(c) => c,
        Err(e) => {
            error!(error = ?e);

            return e.error_response();
        }
    };

    HttpResponse::Created().json(category)
}
