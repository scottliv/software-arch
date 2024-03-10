use anyhow::anyhow;
use database::entity::inspiration_image::ActiveModel as InspirationImageActiveModel;
use reqwest::{Client, Response};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};
use tracing::{event, instrument, Level};

#[derive(serde::Deserialize, Debug)]
pub struct UnsplashImage {
    pub id: String,
    pub urls: ImageUrls,
    pub description: Option<String>,
    pub alt_description: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ImageUrls {
    pub regular: String,
}

#[derive(Debug, Clone)]
pub struct ImageClient {
    http_client: Client,
    base_url: String,
}

impl ImageClient {
    pub fn new(base_url: String) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
        }
    }

    pub async fn get_images(&self) -> anyhow::Result<Response> {
        let res = self.http_client.get(&self.base_url).send().await?;
        Ok(res)
    }
}

#[instrument]
pub async fn fetch_images(client: ImageClient) -> anyhow::Result<Vec<UnsplashImage>> {
    let response = client.get_images().await?;
    println!("{}", response.status());
    if response.status().is_success() {
        let collection: Vec<UnsplashImage> = response.json::<Vec<UnsplashImage>>().await?;
        event!(Level::INFO, "found new images");
        return Ok(collection);
    }
    Err(anyhow!("woops bad result"))
}

#[instrument]
pub async fn insert_image(db: &DatabaseConnection, image: UnsplashImage) -> anyhow::Result<()> {
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
