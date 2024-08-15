use std::env;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test() {
    // wait for the containers to be ready
    while let Err(_) = reqwest::get("http://localhost:8080/public").await {
        eprintln!("Containers are not ready yet...");
        sleep(Duration::from_secs(1)).await;
    }

    // Test public endpoint
    let response = reqwest::get("http://localhost:8080/public")
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 200);
    assert!(response.text().await.unwrap().contains("index.html"));

    // Test private endpoint
    let response = reqwest::get("http://localhost:8080/private")
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 403);
    assert!(response.text().await.unwrap().contains("403.html"));

    // Test private endpoint with token
    let response = reqwest::Client::new()
        .get("http://localhost:8080/private")
        .header("cookie", &format!("token={}", env::var("TOKEN").unwrap()))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), 200);
    assert!(response.text().await.unwrap().contains("index.html"));
}
