use actix_web::web::{ServiceConfig, get, post, scope};

use crate::handlers::expense::{add_expense, get_user_expenses};

pub fn route(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/api/expense")
            .route("/", post().to(add_expense))
            .route("/user", get().to(get_user_expenses)),
    );
}
