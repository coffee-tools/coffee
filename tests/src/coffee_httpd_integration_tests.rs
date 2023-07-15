use serde_json::json;

use coffee_lib::types::request::*;
use coffee_testing::cln::Node;
use coffee_testing::CoffeeHTTPDTesting;

use crate::init;

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(560000)]
pub async fn httpd_init_add_remote() {
    init();

    let mut cln = Node::tmp("regtest").await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    let manager = CoffeeHTTPDTesting::tmp(lightning_dir.to_string())
        .await
        .unwrap();
    log::info!("lightning path: {lightning_dir}");
    let url = manager.url();
    log::info!("base url: {url}");
    let client = reqwest::Client::new();

    // Define the request body to be sent to the /remote/add endpoint
    let remote_add_request = RemoteAdd {
        repository_name: "lightningd".to_string(),
        repository_url: "https://github.com/lightningd/plugins.git".to_string(),
    };

    // Send the request to add a remote repository
    let response = client
        .post(format!("{}/remote/add", url))
        .json(&remote_add_request)
        .send()
        .await
        .unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/remote/add response: {}", body);

    // Define the request body to be sent to the /install endpoint
    let install_request = Install {
        plugin: "summary".to_string(),
        try_dynamic: true,
    };

    // Send the request to install a plugin
    let response = client
        .post(format!("{}/install", url))
        .json(&install_request)
        .send()
        .await
        .unwrap();

    // Check the response status code, log the body.
    // assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/install response: {}", body);

    // Make sure the "summary" plugin is installed
    cln.rpc()
        .call::<serde_json::Value, serde_json::Value>("summary", json!({}))
        .unwrap();

    cln.stop().await.unwrap();
}
