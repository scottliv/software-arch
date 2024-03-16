use actix_web::{error::InternalError, get, web};
use database::entity::generated_image::Entity as GeneratedImage;
use sea_orm::{DatabaseConnection, DbBackend, EntityTrait, Statement};

use crate::template::GeneratedImageTemplate;

#[get("/generated_images/{id}")]
pub async fn get_image_by_id(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> askama::Result<GeneratedImageTemplate, InternalError<String>> {
    let id = id.parse::<i32>();
    if id.is_err() {
        return Err(InternalError::new(
            "Invalid ID".to_string(),
            actix_web::http::StatusCode::from_u16(400).unwrap(),
        ));
    }

    let image = GeneratedImage::find_by_id(id.unwrap())
        .one(db.as_ref())
        .await;

    if image.is_err() {
        return Err(InternalError::new(
            "Error reading image from db".to_string(),
            actix_web::http::StatusCode::from_u16(500).unwrap(),
        ));
    }

    match image.unwrap() {
        Some(image) => Ok(GeneratedImageTemplate {
            generated_image: image,
        }),
        None => Err(InternalError::new(
            "Image not found".to_string(),
            actix_web::http::StatusCode::from_u16(404).unwrap(),
        )),
    }
}

#[get("/generated_images/{id}/next")]
pub async fn get_next_image(
    db: web::Data<DatabaseConnection>,
    id: web::Path<String>,
) -> askama::Result<GeneratedImageTemplate, InternalError<String>> {
    let id = id.parse::<i32>();
    if id.is_err() {
        return Err(InternalError::new(
            "Invalid ID".to_string(),
            actix_web::http::StatusCode::from_u16(400).unwrap(),
        ));
    }

    let image = GeneratedImage::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT * FROM generated_image WHERE id > $1 ORDER BY id LIMIT 1"#,
            [id.unwrap().into()],
        ))
        .one(db.as_ref())
        .await;

    if image.is_err() {
        return Err(InternalError::new(
            "Error reading image from db".to_string(),
            actix_web::http::StatusCode::from_u16(500).unwrap(),
        ));
    }

    match image.unwrap() {
        Some(image) => Ok(GeneratedImageTemplate {
            generated_image: image,
        }),
        None => Err(InternalError::new(
            "Image not found".to_string(),
            actix_web::http::StatusCode::from_u16(404).unwrap(),
        )),
    }
}

#[get("/generated_images/first")]
pub async fn get_first_image(
    db: web::Data<DatabaseConnection>,
) -> askama::Result<GeneratedImageTemplate, InternalError<String>> {
    let image = GeneratedImage::find().one(db.as_ref()).await;

    if image.is_err() {
        return Err(InternalError::new(
            "Error reading image from db".to_string(),
            actix_web::http::StatusCode::from_u16(500).unwrap(),
        ));
    }

    match image.unwrap() {
        Some(image) => Ok(GeneratedImageTemplate {
            generated_image: image,
        }),
        None => Err(InternalError::new(
            "Image not found".to_string(),
            actix_web::http::StatusCode::from_u16(404).unwrap(),
        )),
    }
}
