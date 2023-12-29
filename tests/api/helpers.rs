use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // Cannot assign the output of `get_subscriber`` to a variable based on the value `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber``, therefore
    // they are not the same type. Could work around this, but this is the most straight-forward approach

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub api_address: String,
    pub api_port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.api_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute the request.")
    }

    pub async fn post_newsletter(&self, body: serde_json::Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/newsletters", &self.api_address))
            .json(&body)
            .send()
            .await
            .expect("Failed to execute request to create new newsletter.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_link = |s| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();

            // Ensure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.api_port)).unwrap();
            confirmation_link
        };
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }
}

pub async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    // Mock an email server to stand in for the Postmark API
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Use a different database for each test case
        c.database.name = Uuid::new_v4().to_string();
        // Use a random OS port to bind the testing server
        c.application.port = 0;
        // Use the mock server as email API
        c.email_client.base_url = email_server.uri();
        c
    };

    // Create and migrate the database
    configure_database(&configuration.database).await;

    // Build out the application as a background task
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();

    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        api_address: format!("http://127.0.0.1:{}", application_port),
        api_port: application_port,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create
    let mut db_conn = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    db_conn
        .execute(format!(r#"Create Database "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate
    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres new db.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database.");

    db_pool
}
