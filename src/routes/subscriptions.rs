use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<Pool<Postgres>>,
) -> HttpResponse {
    // Slime out an insert statement to the database, will add validations
    match sqlx::query!(
        r#"
        Insert Into subscriptions(id, email, name, subscribed_at)
        Values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
