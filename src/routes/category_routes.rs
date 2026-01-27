use actix_web::web::{ServiceConfig, get, post, scope};

use crate::handlers::category::{add_category, get_user_categories};

pub fn route(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("/api/category")
            .route("/", post().to(add_category))
            .route("/user", get().to(get_user_categories)),
    );
}
