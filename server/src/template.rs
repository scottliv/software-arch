use askama_actix::Template;
use database::entity::generated_image::Model as GeneratedImageModel;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(path = "generated-image.html")]
pub struct GeneratedImageTemplate {
    pub generated_image: GeneratedImageModel,
}
