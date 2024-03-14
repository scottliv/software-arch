use anyhow::Ok;
use database::get_connection;
use server::startup::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@127.0.0.1:5433/rust-software-arch".to_string());
    let db = get_connection(&db_url).await?;
    let app = Application::build(8080, db).await?;
    app.run().await?;

    Ok(())
}
