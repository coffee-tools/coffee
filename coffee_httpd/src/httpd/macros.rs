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
}

pub use handle_httpd_response;
