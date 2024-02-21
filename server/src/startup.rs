use std::net::TcpListener;

use actix_web::{
    dev::Server, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder,
};

#[derive(serde::Deserialize)]
struct FormData {
    input: String,
}

async fn echo_user_input(form: web::Form<FormData>) -> impl Responder {
    let input = &form.input;
    format!("You entered: {input}")
}

async fn index() -> impl Responder {
    let form = format!(
        "<form action=\"/echo_user_input\" method=\"POST\">
         <input name=\"input\">
         <input type=\"submit\" value=\"Submit!\">
     </form>"
    );
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(form)
}

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(port: u16) -> Result<Self, anyhow::Error> {
        let address = format!("{}:{}", Application::address(), port);
        let listener = TcpListener::bind(address)?;
        let server = Application::build_server(listener)?;

        Ok(Self { port, server })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }

    pub fn build_server(listener: TcpListener) -> Result<Server, std::io::Error> {
        let server = HttpServer::new(|| {
            App::new()
                .route("/echo_user_input", web::post().to(echo_user_input))
                .route("/", web::get().to(index))
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
