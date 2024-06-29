use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};
use tokio::{fs, process::Command};

const IMAGE_NAME: &str = "ngx-bearer-authentication";

#[tokio::test]
async fn test() {
    // Build the module
    assert!(Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--target")
        .arg("x86_64-unknown-linux-gnu")
        .env("NGX_VERSION", "1.27.0")
        .output()
        .await
        .expect("Failed to build module")
        .status
        .success(),);

    // Copy the module to the assets directory
    assert_ne!(
        fs::copy(
            "target/x86_64-unknown-linux-gnu/release/libbearerauth.so",
            "assets/libbearerauth.so",
        )
        .await
        .expect("Failed to copy module"),
        0
    );

    // Build the nginx image
    assert!(Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg(&format!("{}:latest", IMAGE_NAME))
        .arg("./assets")
        .output()
        .await
        .expect("Failed to build image")
        .status
        .success());

    // Start the container
    let container = GenericImage::new(IMAGE_NAME, "latest")
        .with_wait_for(WaitFor::healthcheck())
        .with_exposed_port(80.tcp())
        .with_env_var(
            "BEARER_TOKEN",
            "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b",
        )
        .start()
        .await
        .expect("Failed to start container");
    let port = container
        .get_host_port_ipv4(80)
        .await
        .expect("Failed to get port");

    // Test public endpoint
    let response = reqwest::get(format!("http://localhost:{}/public", port))
        .await
        .expect("Failed to send request")
        .text()
        .await
        .expect("Failed to get response");
    assert!(response.contains("index.html"));

    // Test private endpoint
    let response = reqwest::get(format!("http://localhost:{}/private", port))
        .await
        .expect("Failed to send request")
        .text()
        .await
        .expect("Failed to get response");
    assert!(response.contains("403.html"));

    // Test private endpoint with token
    let response = reqwest::Client::new()
        .get(format!("http://localhost:{}/private", port))
        .bearer_auth("secret")
        .send()
        .await
        .expect("Failed to send request")
        .text()
        .await
        .expect("Failed to get response");
    assert!(response.contains("index.html"));
}
