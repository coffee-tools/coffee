//! Coffee Server Daemon implementation,
//!
//! you should start from here is you want understand
//! the code!
//!
//! Please node that this is just a wrapper for the
//! coffee core crate! The goal is to give an web
//! interface to interact with coffee.
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Mutex;

use super::macros::handle_httpd_response;
use coffee_core::coffee::CoffeeManager;
use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::types::request::*;

use actix_web::{App, HttpResponse};
use actix_web::{Error, HttpServer};
use paperclip::actix::HttpResponseWrapper;
use paperclip::actix::{
    api_v2_operation,
    // If you prefer the macro syntax for defining routes, import the paperclip macros
    // get, post, put, delete
    // use this instead of actix_web::web
    get,
    post,
    web::{self, Json},
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt,
};

// This struct represents state
struct AppState {
    #[allow(dead_code)]
    coffee: Arc<Mutex<CoffeeManager>>,
}

/// entry point of the httpd to allow
/// run the server
pub async fn run_httpd<T: ToSocketAddrs>(
    coffee: CoffeeManager,
    host: T,
) -> Result<(), std::io::Error> {
    let rc = Arc::new(Mutex::new(coffee));
    HttpServer::new(move || {
        let state = AppState { coffee: rc.clone() };
        App::new()
            .app_data(web::Data::new(state))
            .wrap_api()
            .service(swagger_api)
            .service(coffee_help)
            .service(coffee_install)
            .service(coffee_remove)
            .service(coffee_list)
            .service(coffee_remote_add)
            .service(coffee_remote_rm)
            .service(coffee_remote_list)
            .service(coffee_show)
            .service(coffee_search)
            .service(coffee_list_plugins_in_remote)
            .with_json_spec_at("/api/v1")
            .build()
    })
    .bind(host)?
    .run()
    .await?;
    Ok(())
}

#[api_v2_operation]
#[get("/help")]
async fn coffee_help(
    _: web::Data<AppState>,
    body: Json<HashMap<String, String>>,
) -> Result<Json<HashMap<String, String>>, Error> {
    // FIXME: the json need to be a struct
    Ok(body)
}

#[api_v2_operation]
#[post("/install")]
async fn coffee_install(
    data: web::Data<AppState>,
    body: Json<Install>,
) -> Result<HttpResponse, Error> {
    let plugin = &body.plugin;
    let try_dynamic = body.try_dynamic;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.install(plugin, false, try_dynamic).await;

    handle_httpd_response!(result, "Plugin '{plugin}' installed successfully")
}

#[api_v2_operation]
#[post("/remove")]
async fn coffee_remove(
    data: web::Data<AppState>,
    body: Json<Remove>,
) -> Result<HttpResponse, Error> {
    let plugin = &body.plugin;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.remove(plugin).await;

    handle_httpd_response!(result, "Plugin '{plugin}' removed successfully")
}

#[api_v2_operation]
#[get("/list")]
async fn coffee_list(data: web::Data<AppState>) -> Result<Json<Value>, Error> {
    let mut coffee = data.coffee.lock().await;
    let result = coffee.list().await;
    handle_httpd_response!(result)
}

#[api_v2_operation]
#[post("/remote/add")]
async fn coffee_remote_add(
    data: web::Data<AppState>,
    body: Json<RemoteAdd>,
) -> Result<HttpResponse, Error> {
    let repository_name = &body.repository_name;
    let repository_url = &body.repository_url;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.add_remote(repository_name, repository_url).await;

    handle_httpd_response!(result, "Repository '{repository_name}' added successfully")
}

#[api_v2_operation]
#[post("/remote/rm")]
async fn coffee_remote_rm(
    data: web::Data<AppState>,
    body: Json<RemoteRm>,
) -> Result<HttpResponse, Error> {
    let repository_name = &body.repository_name;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.rm_remote(repository_name).await;

    handle_httpd_response!(
        result,
        "Repository '{repository_name}' removed successfully"
    )
}

#[api_v2_operation]
#[get("/remote/list")]
async fn coffee_remote_list(data: web::Data<AppState>) -> Result<Json<Value>, Error> {
    let mut coffee = data.coffee.lock().await;
    let result = coffee.list_remotes().await;

    handle_httpd_response!(result)
}

#[api_v2_operation]
#[get("/remote/list_plugins")]
async fn coffee_list_plugins_in_remote(
    data: web::Data<AppState>,
    body: Json<RemotePluginsList>,
) -> Result<Json<Value>, Error> {
    let repository_name = &body.repository_name;

    let coffee = data.coffee.lock().await;
    let result = coffee.get_plugins_in_remote(repository_name).await;

    handle_httpd_response!(result)
}

#[api_v2_operation]
#[get("/show")]
async fn coffee_show(data: web::Data<AppState>, body: Json<Show>) -> Result<Json<Value>, Error> {
    let plugin = &body.plugin;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.show(plugin).await;

    handle_httpd_response!(result)
}

#[api_v2_operation]
#[get("/search")]
async fn coffee_search(
    data: web::Data<AppState>,
    body: Json<Search>,
) -> Result<Json<Value>, Error> {
    let plugin = &body.plugin;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.search(plugin).await;

    handle_httpd_response!(result)
}

// this is just a hack to support swagger UI with https://paperclip-rs.github.io/paperclip/
// and the raw html is taken from https://github.com/swagger-api/swagger-ui/blob/master/docs/usage/installation.md#unpkg
#[get("/")]
async fn swagger_api() -> HttpResponseWrapper {
    // FIXME: the url need to change here so we should support a better way
    let resp = HttpResponse::Ok().body(
        r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta
      name="description"
      content="SwaggerUI"
    />
    <title>SwaggerUI</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui.css" />
  </head>
  <body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui-bundle.js" crossorigin></script>
  <script src="https://unpkg.com/swagger-ui-dist@4.5.0/swagger-ui-standalone-preset.js" crossorigin></script>
  <script>
    window.onload = () => {
      window.ui = SwaggerUIBundle({
        url: 'http://localhost:8080/api/v1',
        dom_id: '#swagger-ui',
        presets: [
          SwaggerUIBundle.presets.apis,
          SwaggerUIStandalonePreset
        ],
        layout: "StandaloneLayout",
      });
    };
  </script>
  </body>
</html>
"#,
    );
    HttpResponseWrapper(resp)
}
