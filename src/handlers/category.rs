use actix_web::{
    HttpResponse, Responder, ResponseError,
    web::{Data, Json, Query},
};

use crate::{
    middleware::auth::AuthMiddleware,
    models::category_models::{Category, CategoryPagination},
    services::{category_services::CategoryService, redis_services::RedisService},
};

pub async fn add_category(
    auth: AuthMiddleware,
    body: Json<Category>,
    redis: Data<RedisService>,
    service: Data<CategoryService>,
) -> impl Responder {
    match service
        .add_category(body.into_inner(), &redis, auth.user_id)
        .await
    {
        Ok(category) => HttpResponse::Created().json(category),
        Err(e) => e.error_response(),
    }
}

pub async fn get_user_categories(
    auth: AuthMiddleware,
    params: Query<CategoryPagination>,
    redis: Data<RedisService>,
    service: Data<CategoryService>,
) -> impl Responder {
    match service
        .get_user_categories(params.into_inner(), &redis, auth.user_id)
        .await
    {
        Ok(category) => HttpResponse::Ok().json(category),
        Err(e) => e.error_response(),
    }
}
