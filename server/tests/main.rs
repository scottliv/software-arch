use database::get_connection;
use server::startup::Application;
use testcontainers::{clients, images};

struct TestApp {
    address: String,
}

impl TestApp {
    pub fn address(self) -> String {
        self.address
    }

    pub async fn build_test_app() -> TestApp {
        let docker = clients::Cli::default();
        let database = images::postgres::Postgres::default();
        let node = docker.run(database);
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        let database_connection = get_connection(connection_string).await.unwrap();
        let app = Application::build(3232, database_connection)
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
