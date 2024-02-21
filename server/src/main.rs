use anyhow::Ok;
use server::startup::Application;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Application::build(8080).await?;
    app.run().await?;

    Ok(())
}
