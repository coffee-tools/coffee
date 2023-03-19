//! Coffee Server Deamon implementation,
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

use coffee_core::coffee::CoffeeManager;

use actix_web::{App, HttpResponse};
use actix_web::{Error, HttpServer};
use paperclip::actix::HttpResponseWrapper;
use paperclip::actix::{
    api_v2_operation,
    // If you prefer the macro syntax for defining routes, import the paperclip macros
    // get, post, put, delete
    // use this instead of actix_web::web
    get,
    web::{self, Json},
    // extension trait for actix_web::App and proc-macro attributes
    OpenApiExt,
};

// This struct represents state
struct AppState {
    #[allow(dead_code)]
    coffee: Arc<CoffeeManager>,
}

/// entry point of the httd to allow
/// run the server
pub async fn run_httpd<T: ToSocketAddrs>(
    coffee: CoffeeManager,
    host: T,
) -> Result<(), std::io::Error> {
    let rc = Arc::new(coffee);
    HttpServer::new(move || {
        let state = AppState { coffee: rc.clone() };
        App::new()
            .app_data(web::Data::new(state))
            .wrap_api()
            .service(swagger_api)
            .service(coffee_help)
            .with_json_spec_at("/api/v1")
            .build()
    })
    .bind(host)?
    .run()
    .await
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

// this is just an hack to support swagger UI with https://paperclip-rs.github.io/paperclip/
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
