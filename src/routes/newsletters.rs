use actix_web::HttpResponse;

// Slime this out
pub async fn publish_newsletter() -> HttpResponse {
    HttpResponse::Ok().finish()
}
