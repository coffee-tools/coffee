//! Coffee Server Deamon implementation,
//!
//! you should start from here is you want understand
//! the code!
//!
//! Please node that this is just a wrapper for the
//! coffee core crate! The goal is to give an web
//! interface to interact with coffee.
use std::net::ToSocketAddrs;

use actix_web::App;
use actix_web::{get, web, HttpResponse, HttpServer, Responder};
use coffee_core::coffee::CoffeeManager;

// This struct represents state
struct AppState {
    #[allow(dead_code)]
    coffee: CoffeeManager,
}

/// entry point of the httd to allow
/// run the server
pub async fn run_httpd<T: ToSocketAddrs>(
    coffee: CoffeeManager,
    host: T,
) -> Result<(), std::io::Error> {
    let state = AppState { coffee };
    App::new().app_data(web::Data::new(state));
    HttpServer::new(|| App::new().service(web::scope("/v1").service(coffee_help)))
        .bind(host)?
        .run()
        .await
}

#[get("/help")]
async fn coffee_help(_: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body("TODO: here will be printed the coffee help")
}
