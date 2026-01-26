use crate::{
    middleware::auth::AuthMiddleware,
    models::expense_model::{AddExpenseRequest, ExpensePath, PageParams},
    services::{expense_services::ExpenseServices, redis_services::RedisService},
};

use actix_web::{
    HttpResponse, Responder, ResponseError,
    web::{Data, Json, Path, Query},
};

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
        Err(e) => return e.error_response(),
    };

    HttpResponse::Created().json(service)
}

pub async fn get_user_expenses(
    auth: AuthMiddleware,
    params: Query<PageParams>,
    redis: Data<RedisService>,
    service: Data<ExpenseServices>,
) -> impl Responder {
    let expenses_with_total = match service
        .get_user_expenses(params.into_inner(), &redis, auth.user_id)
        .await
    {
        Ok(e) => e,
        Err(e) => return e.error_response(),
    };

    HttpResponse::Ok().json(expenses_with_total)
}

pub async fn get_single_expense_per_user(
    auth: AuthMiddleware,
    params: Path<ExpensePath>,
    service: Data<ExpenseServices>,
    redis: Data<RedisService>,
) -> impl Responder {
    match service
        .get_single_expense_per_user(params.into_inner(), &redis, auth.user_id)
        .await
    {
        Ok(expense) => HttpResponse::Ok().json(expense),
        Err(e) => e.error_response(),
    }
}
