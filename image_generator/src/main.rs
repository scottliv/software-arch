use anyhow::anyhow;
use base64::decode;
use database::entity::generated_image::{
    ActiveModel as GeneratedImageActiveModel, Model as GeneratedImageModel,
};
use database::entity::inspiration_image::{self, Entity as InspirationImage, Model};
use database::{get_queue_connection, GenerateImageMessage};
use rusoto_core::{ByteStream, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde_json::json;
use tracing::{event, instrument, Level};

#[derive(serde::Deserialize)]
struct GeneratedImageResponse {
    created: u64,
    data: Vec<GeneratedImage>,
}

#[derive(serde::Deserialize)]
struct GeneratedImage {
    b64_json: String,
    revised_prompt: String,
}

#[instrument]
async fn generate_image(inspiration_image: Model) -> anyhow::Result<GeneratedImageResponse> {
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

    event!(Level::INFO, "Generating Image");
    let res = client
        .post(url)
        .header("Authorization", format!("Bearer {}", open_ai_access_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let json = res.json::<GeneratedImageResponse>().await?;
    if json.data.is_empty() {
        return Err(anyhow!("No image generated"));
    }

    event!(Level::INFO, "Image Generated");
    Ok(json)
}

async fn save_image(
    db: &DatabaseConnection,
    image_url: String,
    inspiration_image_id: i32,
) -> anyhow::Result<GeneratedImageModel> {
    let generated_image = GeneratedImageActiveModel {
        inspiration_image_id: Set(inspiration_image_id),
        source_url: Set(image_url),
        ..Default::default()
    };

    let img = generated_image.insert(db).await?;

    Ok(img)
}

async fn parse_image_response(response: GeneratedImageResponse) -> anyhow::Result<ByteStream> {
    match decode(
        &response
            .data
            .first()
            .expect("data is checked to not be empty")
            .b64_json,
    ) {
        Ok(image_data) => Ok(ByteStream::from(image_data)),

        Err(e) => Err(anyhow!(e)),
    }
}

#[instrument]
async fn handle_message(inspiration_image_id: i32, db: &DatabaseConnection) -> anyhow::Result<()> {
    let inspiration_image = InspirationImage::find()
        .filter(inspiration_image::Column::Description.is_not_null())
        .one(db)
        .await?;

    if inspiration_image.is_some() {
        let image_response = generate_image(inspiration_image.unwrap()).await?;
        let image_data = parse_image_response(image_response).await?;
        let image_url = upload_to_s3(image_data, inspiration_image_id).await?;

        save_image(db, image_url, inspiration_image_id).await?;
        event!(Level::INFO, "Saved new generated image");
    }

    Ok(())
}

#[instrument]
async fn upload_to_s3(image_data: ByteStream, image_id: i32) -> anyhow::Result<String> {
    let s3_access_key = std::env::var("S3_ACCESS_KEY").expect("S3 access key must be set");
    let s3_secret_key = std::env::var("S3_SECRET_KEY").expect("S3 secret key must be set");
    let bucket_name = "software-arch-images";
    let region = Region::UsEast1;

    event!(Level::INFO, "Authenticating with AWS");
    let credentials_provider = StaticProvider::new_minimal(s3_access_key, s3_secret_key);
    let dispatch_provider = rusoto_core::request::HttpClient::new()?;
    let s3_client = S3Client::new_with(dispatch_provider, credentials_provider, region);
    let id = uuid::Uuid::new_v4();
    let key = format!("{}:{}", image_id, id);

    let request = PutObjectRequest {
        bucket: bucket_name.to_owned(),
        key: key.to_owned(),
        content_encoding: Some("base64".to_string()),
        content_type: Some("image/png".to_string()),
        body: Some(image_data),
        ..Default::default()
    };

    event!(Level::INFO, "Uploading image to s3");
    s3_client.put_object(request).await?;

    let img_url = format!("https://{}.s3.amazonaws.com/{}", bucket_name, key);

    Ok(img_url)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or("postgres://postgres:postgres@127.0.0.1:5433/rust-software-arch".to_string());
    let db = std::sync::Arc::new(database::get_connection(&db_url).await?);
    let message_queue_url =
        std::env::var("MESSAGE_QUEUE_URL").expect("Message queue url must be set");
    println!("db up!");

    let image_queue = get_queue_connection(message_queue_url, "generate_image".to_string()).await;

    loop {
        let received_message = image_queue
            .queue
            .read::<GenerateImageMessage>(&image_queue.queue_name, Some(60))
            .await;

        match received_message {
            Ok(message) => match message {
                Some(message) => {
                    match handle_message(message.message.inspiration_image_id, &db).await {
                        Ok(_) => {
                            event!(Level::INFO, "Image successfully generated");
                            let _ = image_queue
                                .queue
                                .archive(&image_queue.queue_name, message.msg_id)
                                .await;
                        }
                        Err(e) => event!(Level::INFO, "{e}"),
                    }
                }
                None => {
                    event!(Level::INFO, "Queue is empty, sleeping for a few");
                    tokio::time::sleep(std::time::Duration::from_secs(100)).await;
                }
            },
            Err(e) => event!(Level::INFO, "Error reading from message queue: {e}"),
        }
    }
}
