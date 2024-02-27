use database::entity::inspiration_image::ActiveModel as InspirationImageActiveModel;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DatabaseConnection};

#[derive(serde::Deserialize, Debug)]
struct UnsplashImage {
    id: String,
    urls: ImageUrls,
}

#[derive(serde::Deserialize, Debug)]
struct ImageUrls {
    regular: String,
}

async fn fetch_images() -> anyhow::Result<Vec<UnsplashImage>> {
    let client = reqwest::Client::new();
    let access_key = std::env::var("UNSPLASH_ACCESS_KEY").expect("Unsplash access key must be set");
    let url = format!(
        "https://api.unsplash.com/photos?order_by=popular&client_id={}",
        access_key
    );

    let response = client.get(url).send().await.expect("Error fetching images");
    let collection = response.json::<Vec<UnsplashImage>>().await?;

    Ok(collection)
}

async fn insert_image(db: DatabaseConnection, image: UnsplashImage) -> anyhow::Result<()> {
    let inspiration_image = InspirationImageActiveModel {
        source_id: Set(image.id),
        source_url: Set(image.urls.regular),
        description: Set(None),
        ..Default::default()
    };

    inspiration_image.insert(&db).await?;
    println!("Saved image");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db =
        database::get_connection("postgres://postgres:postgres@localhost:5433/rust-software-arch")
            .await?;

    let images = fetch_images().await?;
    for image in images {
        insert_image(db.clone(), image).await?;
    }

    Ok(())
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
