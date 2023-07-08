use coffee_testing::cln::Node;
use coffee_testing::CoffeeHTTPDTesting;
use std::sync::Once;
// use reqwest::Client;
// use serde_json::json;

#[cfg(test)]
static INIT: Once = Once::new();

#[cfg(test)]
fn init() {
    use crate::logger;
    // ignore error
    INIT.call_once(|| {
        logger::init(log::Level::Debug).expect("initializing logger for the first time");
    });
}

#[tokio::test]
pub async fn init_httpd_add_remote() {
    init();

    let mut cln = Node::tmp().await.unwrap();
    let manager = CoffeeHTTPDTesting::tmp().await.unwrap();

    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");

    let url = manager.url().await.unwrap();
    log::info!("base url: {url}");

    let body = reqwest::get(format!("http://{url}/list"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    println!("Response body: {}", body);

    cln.stop().await.unwrap();
}
