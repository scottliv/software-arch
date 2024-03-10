use image_collector::{fetch_images, insert_image, ImageClient};
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{event, instrument, Level};

#[instrument]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@127.0.0.1:5433/rust-software-arch".to_string());
    let db = std::sync::Arc::new(database::get_connection(&db_url).await?);
    let access_key = std::env::var("UNSPLASH_ACCESS_KEY").expect("Unsplash access key must be set");
    let url = format!(
        "https://api.unsplash.com/photos?order_by=popular&client_id={}",
        access_key
    );

    let image_client = ImageClient::new(url);

    println!("Up and atom");
    let sched = JobScheduler::new().await?;
    let job = Job::new_async("0 8 * * * *", move |_uuid, mut _l| {
        let db_clone = db.clone();
        let client_clone = image_client.clone();
        Box::pin(async move {
            let images = fetch_images(client_clone.clone()).await;

            match images {
                Ok(images) => {
                    for image in images {
                        event!(Level::INFO, "Saving image to db");
                        let _ = insert_image(db_clone.as_ref(), image).await;
                    }
                }
                Err(e) => event!(Level::WARN, "Error loading images: {e}"),
            }
        })
    })?;
    sched.add(job).await?;

    sched.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
