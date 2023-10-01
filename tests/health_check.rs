use zero2prod::run;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app();
    // Needs the `reqwest` crate to run HTTP requests against the server
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8000/health-check")
        .send()
        .await
        .expect("Failed to run request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = zero2prod::run().expect("Failed to bind address");

    let _ = tokio::spawn(server);
}
