mod models;
mod response;
mod service;
mod utils;

use std::{env, io};

use actix_cors::Cors;
use actix_web::{
    get, http::header, middleware::Logger, web, App, HttpResponse,
    HttpServer, Responder,
};
use serde_json::json;

use models::AppState;

#[get("/api/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str =
        "How to Implement Two-Factor Authentication (2FA) in Rust (Actix)";

    HttpResponse::Ok().json(json!({
      "status": "success",
      "message": MESSAGE
    }))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();

    let db = AppState::init();
    let app_data = web::Data::new(db);

    println!("🚀 Server started successfully");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:3000/")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();

        App::new()
            .app_data(app_data.clone())
            .service(health_checker_handler)
            .configure(service::config)
            .wrap(cors)
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
