use clap::Parser;

use coffee_core::coffee::CoffeeManager;
use coffee_lib::errors::CoffeeError;
use coffee_lib::macros::error;
use coffee_lib::plugin_manager::PluginManager;

mod cmd;
pub mod httpd;

#[actix_web::main]
async fn main() -> Result<(), CoffeeError> {
    env_logger::init();
    let cmd = cmd::HttpdArgs::parse();
    let mut coffee = CoffeeManager::new(&cmd).await?;
    coffee.link(&cmd.cln_path).await?;

    let port = cmd.port.unwrap_or(8080) as u16;
    log::info!("Running on port 127.0.0.1:{port}");
    if let Err(err) = httpd::run_httpd(coffee, ("127.0.0.1", port)).await {
        return Err(error!("Error while running the httpd: {err}"));
    }

    Ok(())
}
