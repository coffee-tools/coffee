use std::sync::Once;

use coffee_lib::plugin_manager::PluginManager;
use coffee_testing::cln::Node;
use coffee_testing::CoffeeTesting;
use reqwest::Client;
use serde_json::json;

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
    let mut manager = CoffeeTesting::tmp().await.unwrap();

    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");
    manager.coffee().setup(&lightning_dir).await.unwrap();

    let client = Client::new();
    let url = CoffeeTesting::httpd(&mut manager).await.unwrap();
    let repository_name = "lightningd";
    let repository_url = "https://github.com/lightningd/plugins.git";

    let body = json!({
        "repository_name": repository_name,
        "repository_url": repository_url
    });

    let response = client
        .post(url)
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .unwrap();

    // Process the response as needed
    let status = response.status();
    let text = response.text().await.unwrap();

    println!("Status: {}", status);
    println!("Response body: {}", text);

    cln.stop().await.unwrap();
}
