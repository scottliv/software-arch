use anyhow::anyhow;
use base64::decode;
use database::entity::generated_image::{
    ActiveModel as GeneratedImageActiveModel, Model as GeneratedImageModel,
};
use database::entity::inspiration_image::{self, Entity as InspirationImage, Model};
use rusoto_core::{ByteStream, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{PutObjectOutput, PutObjectRequest, S3Client, S3};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::json;

#[derive(serde::Deserialize)]
struct GeneratedImageResponse {
    created: u64,
    data: Vec<GeneratedImage>,
}

#[derive(serde::Deserialize)]
struct GeneratedImage {
    base64_json: String,
    revised_prompt: String,
}

async fn generate_image(
    db: &DatabaseConnection,
    inspiration_image: Model,
) -> anyhow::Result<GeneratedImageModel> {
    let client = reqwest::Client::new();
    let open_ai_access_key =
        std::env::var("OPEN_AI_ACCESS_KEY").expect("Open AI access key must be set");

    let url = "https://api.openai.com/v1/images/generations";
    let body = json!({
      "model": "dall-e-3",
      "prompt": inspiration_image.description,
      "n": 1,
      "size": "1024x1024",
      "response_format" : "b64_json"
    });

    let res = client
        .post(url)
        .header("Authorization", format!("Bearer {}", open_ai_access_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let parsed_response = res.json::<GeneratedImageResponse>().await?;
    if parsed_response.data.is_empty() {
        return Err(anyhow!("No image generated"));
    }

    // Only care about 1 image for now
    let image_url = match decode(
        &parsed_response
            .data
            .first()
            .expect("data is checked to not be empty")
            .base64_json,
    ) {
        Ok(image_data) => upload_to_s3(ByteStream::from(image_data), inspiration_image.id)
            .await?
            .unwrap(),
        Err(e) => return Err(anyhow!(e)),
    };

    let generated_image = GeneratedImageActiveModel {
        inspiration_image_id: Set(inspiration_image.id),
        source_url: Set(image_url),
        ..Default::default()
    };

    let img = generated_image.insert(db).await?;

    Ok(img)
}

async fn upload_to_s3(image_data: ByteStream, image_id: i32) -> anyhow::Result<Option<String>> {
    let s3_access_key = std::env::var("S3_ACCESS_KEY").expect("S3 access key must be set");
    let s3_secret_key = std::env::var("S3_SECRET_KEY").expect("S3 secret key must be set");
    let region = Region::UsEast1;

    let credentials_provider = StaticProvider::new_minimal(s3_access_key, s3_secret_key);
    let dispatch_provider = rusoto_core::request::HttpClient::new()?;
    let s3_client = S3Client::new_with(dispatch_provider, credentials_provider, region);

    let request = PutObjectRequest {
        bucket: "software-arch-images".to_owned(),
        key: image_id.to_string(),
        content_encoding: Some("base64".to_string()),
        content_type: Some("image/png".to_string()),
        body: Some(image_data),
        ..Default::default()
    };

    let response: PutObjectOutput = s3_client.put_object(request).await?;

    Ok(response.e_tag)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@127.0.0.1:5433/rust-software-arch".to_string());
    let db = std::sync::Arc::new(database::get_connection(&db_url).await?);

    println!("db up!");
    let test_image = InspirationImage::find()
        .filter(inspiration_image::Column::Description.is_not_null())
        .one(db.as_ref())
        .await?;

    if test_image.is_some() {
        let img: GeneratedImageModel = generate_image(&db, test_image.unwrap()).await?;

        println!("{}", img.source_url);
    }

    Ok(())
}
