use std::time::Duration;

use anyhow::anyhow;
use database::entity::inspiration_image::ActiveModel as InspirationImageActiveModel;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{event, instrument, Level};

#[derive(serde::Deserialize, Debug)]
struct UnsplashImage {
    id: String,
    urls: ImageUrls,
    description: Option<String>,
    alt_description: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct ImageUrls {
    regular: String,
}

#[instrument]
async fn fetch_images() -> anyhow::Result<Vec<UnsplashImage>> {
    let client = reqwest::Client::new();
    let access_key = std::env::var("UNSPLASH_ACCESS_KEY").expect("Unsplash access key must be set");
    let url = format!(
        "https://api.unsplash.com/photos?order_by=popular&client_id={}",
        access_key
    );

    let response = client.get(url).send().await?;
    if response.status().is_success() {
        let collection: Vec<UnsplashImage> = response.json::<Vec<UnsplashImage>>().await?;
        event!(Level::INFO, "found new images");
        return Ok(collection);
    }
    Err(anyhow!("woops bad result"))
}

#[instrument]
async fn insert_image(db: &DatabaseConnection, image: UnsplashImage) -> anyhow::Result<()> {
    if image.description.is_none() && image.alt_description.is_none() {
        return Err(anyhow!("Image cannot be saved without a description"));
    }
    let description = match image.description {
        Some(d) => d,
        None => image.alt_description.expect("checked above for some"),
    };
    let inspiration_image = InspirationImageActiveModel {
        source_id: Set(image.id),
        source_url: Set(image.urls.regular),
        description: Set(Some(description)),
        ..Default::default()
    };

    inspiration_image.insert(db).await?;
    println!("Saved image");
    Ok(())
}

#[instrument]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@127.0.0.1:5433/rust-software-arch".to_string());
    let db = std::sync::Arc::new(database::get_connection(&db_url).await?);

    println!("Up and atom");
    let sched = JobScheduler::new().await?;
    let job = Job::new_async("0 8 * * * *", move |_uuid, mut _l| {
        let db_clone = db.clone();
        Box::pin(async move {
            let images = fetch_images().await;

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

#[cfg(test)]
mod tests {
    use database::get_connection;
    use testcontainers::{clients, images};

    #[tokio::test]
    async fn test_insert_image() {
        let docker = clients::Cli::default();
        let database = images::postgres::Postgres::default();
        let node = docker.run(database);
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        let database_connection = get_connection(connection_string).await.unwrap();
    }
}
