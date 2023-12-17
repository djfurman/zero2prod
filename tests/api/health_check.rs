use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    // Pull back the application data used for the test server
    let app = spawn_app().await;
    // Needs the `reqwest` crate to run HTTP requests against the server
    // Define a client for HTTP requests in testing
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health-check", &app.api_address))
        .send()
        .await
        .expect("Failed to run request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(19), response.content_length());
}
