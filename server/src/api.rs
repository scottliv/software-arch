use actix_web::{error::InternalError, get, web};
use database::entity::generated_image::{Entity as GeneratedImage, Model as GeneratedImageModel};
use database::entity::inspiration_image::{
    Entity as InspirationImage, Model as InspirationImageModel,
};
use sea_orm::{DatabaseConnection, DbBackend, DbErr, EntityTrait, Statement};

use crate::template::GeneratedImageTemplate;

#[get("/images/{id}")]
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

    let (generated_image, inspiration_image) =
        get_generated_and_inspiration_image(image, db.as_ref()).await?;

    Ok(GeneratedImageTemplate {
        generated_image,
        inspiration_image,
    })
}

#[get("/images/{id}/next")]
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
    let (generated_image, inspiration_image) =
        get_generated_and_inspiration_image(image, db.as_ref()).await?;

    Ok(GeneratedImageTemplate {
        generated_image,
        inspiration_image,
    })
}

#[get("/images/{id}/previous")]
pub async fn get_previous_image(
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
            r#"SELECT * FROM generated_image WHERE id < $1 ORDER BY id DESC LIMIT 1"#,
            [id.unwrap().into()],
        ))
        .one(db.as_ref())
        .await;
    let (generated_image, inspiration_image) =
        get_generated_and_inspiration_image(image, db.as_ref()).await?;

    Ok(GeneratedImageTemplate {
        generated_image,
        inspiration_image,
    })
}

#[get("/images/first")]
pub async fn get_first_image(
    db: web::Data<DatabaseConnection>,
) -> askama::Result<GeneratedImageTemplate, InternalError<String>> {
    let image = GeneratedImage::find().one(db.as_ref()).await;
    let (generated_image, inspiration_image) =
        get_generated_and_inspiration_image(image, db.as_ref()).await?;

    Ok(GeneratedImageTemplate {
        generated_image,
        inspiration_image,
    })
}

async fn get_generated_and_inspiration_image(
    image: Result<Option<GeneratedImageModel>, DbErr>,
    db: &DatabaseConnection,
) -> Result<(GeneratedImageModel, InspirationImageModel), InternalError<String>> {
    let image = match image {
        Ok(image) => image,
        Err(_) => {
            return Err(InternalError::new(
                "Error reading image from db".to_string(),
                actix_web::http::StatusCode::from_u16(500).unwrap(),
            ));
        }
    };

    let image = match image {
        Some(image) => image,
        None => {
            return Err(InternalError::new(
                "Image not found".to_string(),
                actix_web::http::StatusCode::from_u16(404).unwrap(),
            ));
        }
    };

    let inspiration_image = InspirationImage::find_by_id(image.inspiration_image_id)
        .one(db)
        .await;

    let inspiration_image = match inspiration_image {
        Ok(inspiration_image) => inspiration_image,
        Err(_) => {
            return Err(InternalError::new(
                "Error reading image from db".to_string(),
                actix_web::http::StatusCode::from_u16(500).unwrap(),
            ));
        }
    };

    match inspiration_image {
        Some(inspo_image) => Ok((image, inspo_image)),
        None => Err(InternalError::new(
            "Image not found".to_string(),
            actix_web::http::StatusCode::from_u16(404).unwrap(),
        )),
    }
}
