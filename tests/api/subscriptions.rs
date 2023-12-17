use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=Daniel%20Furman&email=djfurman%40users.noreply.github.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.api_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    // Ensure we have the proper status code
    assert_eq!(200, response.status().as_u16());

    // Verify the database
    let saved_record = sqlx::query!("Select email, name From subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    // Ensure we have the correct database record
    assert_eq!(saved_record.email, "djfurman@users.noreply.github.com");
    assert_eq!(saved_record.name, "Daniel Furman");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=daniel%20furman", "missing email input"),
        (
            "email=djfurman%40users.noreply.github.com",
            "missing name input",
        ),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.api_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            "name=&email=email=djfurman%40users.noreply.github.com",
            "empty name",
        ),
        ("name=daniel%20furman&email=", "empty email"),
        (
            "name=daniel%20furman&email=not-anything-like-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.api_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        )
    }
}
