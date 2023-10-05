use actix_web::{web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// Slime this out by just returning a 200 OK response
pub async fn subscribe(_form: web::Form<FormData>) -> impl Responder {
    HttpResponse::Ok()
}
