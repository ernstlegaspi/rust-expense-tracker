use crate::{
    middleware::auth::AuthMiddleware,
    models::expense_model::{AddExpenseRequest, QueryParams},
    services::{expense_services::ExpenseServices, redis_services::RedisService},
};

use actix_web::{
    HttpResponse, Responder, ResponseError,
    web::{Data, Json, Query},
};

use tracing::error;

pub async fn add_expense(
    auth: AuthMiddleware,
    body: Json<AddExpenseRequest>,
    redis: Data<RedisService>,
    service: Data<ExpenseServices>,
) -> impl Responder {
    let service = match service
        .add_expense(body.into_inner(), &redis, auth.user_id)
        .await
    {
        Ok(s) => s,
        Err(e) => {
            error!(error = ?e);

            return e.error_response();
        }
    };

    HttpResponse::Created().json(service)
}

pub async fn get_user_expenses(
    auth: AuthMiddleware,
    params: Query<QueryParams>,
    redis: Data<RedisService>,
    service: Data<ExpenseServices>,
) -> impl Responder {
    let expenses = match service
        .get_user_expenses(params.into_inner(), &redis, auth.user_id)
        .await
    {
        Ok(e) => e,
        Err(e) => {
            error!(error = ?e);

            return e.error_response();
        }
    };

    HttpResponse::Ok().json(expenses)
}
