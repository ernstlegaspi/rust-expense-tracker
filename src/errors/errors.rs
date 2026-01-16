use actix_web::HttpResponse;

#[derive(Debug)]
pub enum UserError {
    DuplicateEmail,
    Internal(String),
    InvalidEmail,
    InvalidNameLength,
    NameRequired,
    WeakPassword,
}

pub fn error_response(error: &str) -> HttpResponse {
    HttpResponse::Conflict().json(serde_json::json!({
        "error": error
    }))
}
