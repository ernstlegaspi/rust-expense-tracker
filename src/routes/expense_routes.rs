use actix_web::web::{ServiceConfig, delete, get, post, put, scope};

use crate::handlers::expense::{
    add_expense, delete_expense_per_user, edit_expense_per_user, get_single_expense_per_user,
    get_user_expenses,
};

pub fn route(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/api/expense")
            .route("/", post().to(add_expense))
            .route("/user", get().to(get_user_expenses))
            .route("/user/{expense_id}", get().to(get_single_expense_per_user))
            .route("/user/{expense_id}", put().to(edit_expense_per_user))
            .route("/user/{expense_id}", delete().to(delete_expense_per_user)),
    );
}
