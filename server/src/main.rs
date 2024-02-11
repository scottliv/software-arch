use actix_web::{http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/echo_user_input", web::post().to(echo_user_input))
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
