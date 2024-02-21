use server::startup::Application;

struct TestApp {
    address: String,
}

impl TestApp {
    pub fn address(self) -> String {
        self.address
    }

    pub async fn build_test_app() -> TestApp {
        let app = Application::build(3232)
            .await
            .expect("Failed to build test app");
        let _ = tokio::spawn(app.run());

        Self {
            address: format!("http://{}:{}", Application::address(), 3232),
        }
    }
}

#[tokio::test]
async fn test_echo_user_input() {
    let app = TestApp::build_test_app().await;
    let client = reqwest::Client::new();
    let address = format!("{}/echo_user_input", app.address());

    let response = client
        .post(address)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("input=hello")
        .send()
        .await
        .unwrap();

    let text = response.text().await.unwrap();

    assert_eq!(text, "You entered: hello")
}
