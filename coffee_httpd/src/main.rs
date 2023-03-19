use clap::Parser;
use coffee_core::coffee::CoffeeManager;

mod cmd;
pub mod httpd;

#[actix_web::main]
async fn main() {
    let cmd = cmd::HttpdArgs::parse();
    let coffee = CoffeeManager::new(&cmd).await;
    if let Err(err) = &coffee {
        println!("{err}");
    }
    let coffee = coffee.unwrap();
    if let Err(err) = httpd::run_httpd(coffee, ("127.0.0.1", 8080)).await {
        println!("{err}");
    }
}
