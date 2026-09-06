//! Authentication HTTP response helpers.

use actix_web::HttpResponse;

/// Unauthorized response for missing or invalid auth credentials.
pub(crate) fn unauthorized_response(message: &str) -> HttpResponse {
    HttpResponse::Unauthorized().body(message.to_string())
}
