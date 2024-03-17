use askama_actix::Template;
use database::entity::generated_image::Model as GeneratedImageModel;
use database::entity::inspiration_image::Model as InspirationImageModel;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "images.html")]
pub struct GeneratedImageTemplate {
    pub generated_image: GeneratedImageModel,
    pub inspiration_image: InspirationImageModel,
}
