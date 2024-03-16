use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_prom::PrometheusMetricsBuilder;
use sea_orm::DatabaseConnection;

use crate::{
    api::{get_first_image, get_image_by_id, get_next_image},
    template::IndexTemplate,
};

#[derive(serde::Deserialize)]
struct FormData {
    input: String,
}

async fn echo_user_input(form: web::Form<FormData>) -> impl Responder {
    let input = &form.input;
    format!("You entered: {input}")
}

pub async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn index() -> impl Responder {
    IndexTemplate {}
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        port: u16,
        db_connection: DatabaseConnection,
    ) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", Application::address(), port);
        let listener = TcpListener::bind(address)?;
        let server = Application::build_server(listener, db_connection)?;

        Ok(Self { port, server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn build_server(
        listener: TcpListener,
        db_connection: DatabaseConnection,
    ) -> Result<Server, std::io::Error> {
        let prometheus = PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap();

        let server = HttpServer::new(move || {
            App::new()
                .wrap(prometheus.clone())
                .route("/echo_user_input", web::post().to(echo_user_input))
                .route("/health_check", web::get().to(health_check))
                .route("/", web::get().to(index))
                .service(get_first_image)
                .service(get_image_by_id)
                .service(get_next_image)
                .app_data(web::Data::new(db_connection.clone()))
        })
        .listen(listener)?
        .run();
        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn address() -> String {
        match std::env::var("APP_ENVIRONMENT") {
            Ok(value) if value == "production" => "0.0.0.0".to_string(),
            _ => "127.0.0.1".to_string(),
        }
    }
}
