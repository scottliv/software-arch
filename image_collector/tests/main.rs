#[cfg(test)]
mod tests {
    use database::entity::inspiration_image::Entity as InspirationImage;
    use database::get_connection;
    use migration::sea_orm::Database;
    use migration::{Migrator, MigratorTrait};
    use sea_orm::EntityTrait;
    use serde_json::json;
    use testcontainers::{clients, images};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use image_collector::UnsplashImage;
    use image_collector::{fetch_images, insert_image};
    use image_collector::{ImageClient, ImageUrls};

    #[tokio::test]
    async fn test_insert_image() {
        let docker = clients::Cli::default();
        let db = images::postgres::Postgres::default();
        let node = docker.run(db);
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        let database_connection = get_connection(connection_string).await.unwrap();

        let migration_connection = Database::connect(connection_string).await.unwrap();
        Migrator::up(&migration_connection, None).await.unwrap();

        let image = UnsplashImage {
            id: "test".to_string(),
            urls: ImageUrls {
                regular: "https://example.com".to_string(),
            },
            description: Some("this is an image".to_string()),
            alt_description: None,
        };

        insert_image(&database_connection, image).await.unwrap();
        let inserted_images = InspirationImage::find()
            .all(&database_connection)
            .await
            .unwrap();

        assert_eq!(inserted_images.len(), 1);
    }

    #[tokio::test]
    async fn test_insert_image_requires_at_least_one_description() {
        let docker = clients::Cli::default();
        let db = images::postgres::Postgres::default();
        let node = docker.run(db);
        let connection_string = &format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            node.get_host_port_ipv4(5432)
        );
        let database_connection = get_connection(connection_string).await.unwrap();

        let migration_connection = Database::connect(connection_string).await.unwrap();
        Migrator::up(&migration_connection, None).await.unwrap();

        let image = UnsplashImage {
            id: "test".to_string(),
            urls: ImageUrls {
                regular: "https://example.com".to_string(),
            },
            description: None,
            alt_description: None,
        };
        let result = insert_image(&database_connection, image).await;
        assert!(result.is_err());
        let inserted_images = InspirationImage::find()
            .all(&database_connection)
            .await
            .unwrap();

        assert_eq!(inserted_images.len(), 0);
    }

    #[tokio::test]
    async fn test_fetch_images() {
        let mock_server = MockServer::start().await;

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_json(unsplash_response_stub()))
            .mount(&mock_server)
            .await;

        let image_client = ImageClient::new(mock_server.uri());

        let images = fetch_images(image_client).await.unwrap();
        assert_eq!(images.len(), 1);
    }

    fn unsplash_response_stub() -> serde_json::Value {
        json!([
        {
          "id": "LBI7cgq3pbM",
          "created_at": "2016-05-03T11:00:28-04:00",
          "updated_at": "2016-07-10T11:00:01-05:00",
          "width": 5245,
          "height": 3497,
          "color": "#60544D",
          "blur_hash": "LoC%a7IoIVxZ_NM|M{s:%hRjWAo0",
          "likes": 12,
          "liked_by_user": false,
          "description": "A man drinking a coffee.",
          "user": {
            "id": "pXhwzz1JtQU",
            "username": "poorkane",
            "name": "Gilbert Kane",
            "portfolio_url": "https://theylooklikeeggsorsomething.com/",
            "bio": "XO",
            "location": "Way out there",
            "total_likes": 5,
            "total_photos": 74,
            "total_collections": 52,
            "instagram_username": "instantgrammer",
            "twitter_username": "crew",
            "profile_image": {
              "small": "https://images.unsplash.com/face-springmorning.jpg?q=80&fm=jpg&crop=faces&fit=crop&h=32&w=32",
              "medium": "https://images.unsplash.com/face-springmorning.jpg?q=80&fm=jpg&crop=faces&fit=crop&h=64&w=64",
              "large": "https://images.unsplash.com/face-springmorning.jpg?q=80&fm=jpg&crop=faces&fit=crop&h=128&w=128"
            },
            "links": {
              "self": "https://api.unsplash.com/users/poorkane",
              "html": "https://unsplash.com/poorkane",
              "photos": "https://api.unsplash.com/users/poorkane/photos",
              "likes": "https://api.unsplash.com/users/poorkane/likes",
              "portfolio": "https://api.unsplash.com/users/poorkane/portfolio"
            }
          },
          "current_user_collections": [
            {
              "id": 206,
              "title": "Makers: Cat and Ben",
              "published_at": "2016-01-12T18:16:09-05:00",
              "last_collected_at": "2016-06-02T13:10:03-04:00",
              "updated_at": "2016-07-10T11:00:01-05:00",
              "cover_photo": null,
              "user": null
            },
          ],
          "urls": {
            "raw": "https://images.unsplash.com/face-springmorning.jpg",
            "full": "https://images.unsplash.com/face-springmorning.jpg?q=75&fm=jpg",
            "regular": "https://images.unsplash.com/face-springmorning.jpg?q=75&fm=jpg&w=1080&fit=max",
            "small": "https://images.unsplash.com/face-springmorning.jpg?q=75&fm=jpg&w=400&fit=max",
            "thumb": "https://images.unsplash.com/face-springmorning.jpg?q=75&fm=jpg&w=200&fit=max"
          },
          "links": {
            "self": "https://api.unsplash.com/photos/LBI7cgq3pbM",
            "html": "https://unsplash.com/photos/LBI7cgq3pbM",
            "download": "https://unsplash.com/photos/LBI7cgq3pbM/download",
            "download_location": "https://api.unsplash.com/photos/LBI7cgq3pbM/download"
          }
        }])
    }
}
