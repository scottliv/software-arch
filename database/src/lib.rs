use pgmq::PGMQueue;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::log;

pub mod entity;

pub async fn get_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(10))
        .max_lifetime(Duration::from_secs(10))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);
    return Database::connect(opt).await;
}

#[derive(Serialize, Debug, Deserialize)]
pub struct GenerateImageMessage {
    pub inspiration_image_id: i32,
}

#[derive(Debug, Clone)]
pub struct GenerateImageQueue {
    pub queue: PGMQueue,
    pub queue_name: String,
}

pub async fn get_queue_connection(database_url: String, queue_name: String) -> GenerateImageQueue {
    let queue = PGMQueue::new(database_url)
        .await
        .expect("Failed to connect message queue to postgres");

    queue
        .create(&queue_name)
        .await
        .expect("Error creating generate image queue");

    GenerateImageQueue { queue, queue_name }
}
