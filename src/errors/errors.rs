use actix_web::HttpResponse;
use serde_json::json;

pub fn e400(error: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({
        "error": error
    }))
}

pub fn e401(error: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({
        "error": error
    }))
}

pub fn e404(error: &str) -> HttpResponse {
    HttpResponse::NotFound().json(json!({
        "error": error
    }))
}

pub fn e409(error: &str) -> HttpResponse {
    HttpResponse::Conflict().json(json!({
        "error": error
    }))
}

pub fn e500(error: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "error": error
    }))
}
