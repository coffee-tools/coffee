/// handle_httpd_response macro is the macro that handles HTTPD responses
#[macro_export]
macro_rules! handle_httpd_response {
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(_) => Ok(HttpResponse::Ok().body(format!($msg))),
            Err(err) => Err(actix_web::error::ErrorInternalServerError(format!(
                "Error: {}",
                err
            ))),
        }
    };
    ($result:expr) => {
        match $result {
            Ok(val) => {
                let val = serde_json::to_value(val).map_err(|err| {
                    actix_web::error::ErrorInternalServerError(format!("Failure: {err}"))
                })?;
                Ok(Json(val))
            }
            Err(err) => Err(actix_web::error::ErrorInternalServerError(format!(
                "Failure: {err}"
            ))),
        }
    };
}

pub use handle_httpd_response;
