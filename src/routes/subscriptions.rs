use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{PgPool, Pool, Postgres};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, db_pool)
)]
pub async fn insert_subscriber(db_pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        Insert Into subscriptions(id, email, name, subscribed_at)
        Values ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early if the function failed, returning `sqlx::Error`
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Adding a subscriber",
    skip(form, db_pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<Pool<Postgres>>,
) -> HttpResponse {
    // Slime out an insert statement to the database, will add validations
    match insert_subscriber(&db_pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
