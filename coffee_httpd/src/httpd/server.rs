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

use coffee_core::coffee::CoffeeManager;
use coffee_lib::plugin_manager::PluginManager;

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
            .service(coffee_list)
            .service(coffee_remote_add)
            .service(coffee_remote_rm)
            .service(coffee_remote_list)
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
#[get("/list")]
async fn coffee_list(data: web::Data<AppState>) -> Result<Json<Value>, Error> {
    let mut coffee = data.coffee.lock().await;
    let result = coffee.list().await;
    match result {
        Ok(coffee_list) => {
            let val = serde_json::to_value(coffee_list).map_err(|err| {
                actix_web::error::ErrorInternalServerError(format!("coffee list error: {err}"))
            })?;
            Ok(Json(val))
        }
        Err(err) => Err(actix_web::error::ErrorInternalServerError(format!(
            "coffee list error: {err}"
        ))),
    }
}

#[api_v2_operation]
#[post("/remote/add")]
async fn coffee_remote_add(
    data: web::Data<AppState>,
    body: Json<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let repository_name = body.get("repository_name").ok_or_else(|| {
        actix_web::error::ErrorBadRequest("Missing 'repository_name' field in the request body")
    })?;
    let repository_url = body.get("repository_url").ok_or_else(|| {
        actix_web::error::ErrorBadRequest("Missing 'repository_url' field in the request body")
    })?;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.add_remote(repository_name, repository_url).await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().body(format!(
            "Repository '{}' added successfully",
            repository_name
        ))),
        Err(err) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to add repository: {err}"
        ))),
    }
}

#[api_v2_operation]
#[post("/remote/rm")]
async fn coffee_remote_rm(
    data: web::Data<AppState>,
    body: Json<HashMap<String, String>>,
) -> Result<HttpResponse, Error> {
    let repository_name = body.get("repository_name").ok_or_else(|| {
        actix_web::error::ErrorBadRequest("Missing 'repository_name' field in the request body")
    })?;

    let mut coffee = data.coffee.lock().await;
    let result = coffee.rm_remote(repository_name).await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().body(format!(
            "Repository '{}' removed successfully",
            repository_name
        ))),
        Err(err) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to remove repository: {err}"
        ))),
    }
}

#[api_v2_operation]
#[get("/remote/list")]
async fn coffee_remote_list(data: web::Data<AppState>) -> Result<Json<Value>, Error> {
    let mut coffee = data.coffee.lock().await;
    let result = coffee.list_remotes().await;

    match result {
        Ok(coffee_remotes) => {
            let val = serde_json::to_value(coffee_remotes).map_err(|err| {
                actix_web::error::ErrorInternalServerError(format!(
                    "Failed to list remote repositories: {err}"
                ))
            })?;
            Ok(Json(val))
        }
        Err(err) => Err(actix_web::error::ErrorInternalServerError(format!(
            "Failed to list remote repositories: {err}"
        ))),
    }
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
