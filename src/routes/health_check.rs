use actix_web::HttpResponse;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthCheckResponse {
    is_healthy: bool,
}

pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(HealthCheckResponse { is_healthy: true })
}
