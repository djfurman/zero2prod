use crate::helpers::{spawn_app, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act

    // A sketch of the newsletter paylos structure
    // Likely to change later
    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "html": "<p>Newsletter body as HTML</p>",
            "text": "Newsletter body as plain text",
        }
    });

    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.api_address))
        .json(&newsletter_body)
        .send()
        .await
        .expect("Failed to post a new newsletter.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies on Drop that we haven't sent the newsletter email
}

/// Use the public API of the application under test to create an unconfirmed subscriber.
async fn create_unconfirmed_subscriber(app: &TestApp) {
    let body = "name=Daniel%20Furman&email=djfurman%40users.noreply.github.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber.")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscription(body.into())
        .await
        .error_for_status()
        .unwrap();
}
