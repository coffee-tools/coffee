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

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(560000)]
pub async fn httpd_add_remove_plugins() {
    init();

    let mut cln = Node::tmp("regtest").await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    let manager = CoffeeHTTPDTesting::tmp(lightning_dir.to_string()).await;
    assert!(manager.is_ok(), "{:?}", manager);
    let manager = manager.unwrap();
    log::info!("lightning path: {lightning_dir}");
    let url = manager.url();
    log::info!("base url: {url}");
    let client = reqwest::Client::new();

    // Define the request body to be sent to the /remote/add endpoint
    let remote_add_request = RemoteAdd {
        repository_name: "lightningd".to_string(),
        repository_url: "https://github.com/lightningd/plugins.git".to_string(),
    };

    let response = client
        .post(format!("{}/remote/add", url))
        .json(&remote_add_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/remote/add response: {}", body);

    // Define the request body to be sent to the /install endpoint
    let install_request = Install {
        plugin: "summary".to_string(),
        try_dynamic: false,
    };

    let response = client
        .post(format!("{}/install", url))
        .json(&install_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    let body = response.text().await.unwrap();
    log::info!("/install response: {}", body);

    let body = reqwest::get(format!("{}/remote/list", url)).await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap().json::<serde_json::Value>().await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap();

    // Log the response body
    log::info!("/remote/list response: {}", body);

    // Assert that the "lightningd" remote repository exists in the response
    let remotes = body["remotes"].as_array();
    assert!(remotes.is_some(), "{:?}", remotes);
    let remotes = remotes.unwrap();
    assert!(
        remotes
            .iter()
            .any(|repo| repo["local_name"] == "lightningd"),
        "lightningd remote repository not found in the response"
    );

    let body = reqwest::get(format!("{}/list", url)).await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap().json::<serde_json::Value>().await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap();

    // Log the response body
    log::info!("/list response: {}", body);

    // Assert that the "summary" plugin exist in the response
    let plugins = body["plugins"].as_array();
    assert!(plugins.is_some(), "{:?}", plugins);
    let plugins = plugins.unwrap();
    assert!(
        plugins.iter().any(|plugin| plugin["name"] == "summary"),
        "summary plugin not found in the response"
    );

    // Define the request body to be sent
    let plugin_remove_request = Remove {
        plugin: "summary".to_string(),
    };

    let response = client
        .post(format!("{}/remove", url))
        .json(&plugin_remove_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("Response body: {}", body);

    // Define the request body to be sent
    let remote_rm_request = RemoteRm {
        repository_name: "lightningd".to_string(),
    };

    // This should also remove the helpme plugin
    let response = client
        .post(format!("{}/remote/rm", url))
        .json(&remote_rm_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/remote/rm response: {}", body);

    let body = reqwest::get(format!("{}/remote/list", url)).await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap().json::<serde_json::Value>().await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap();

    // Log the response body
    log::info!("/remote/list response: {}", body);

    // Assert that the "lightningd" remote repository doesn't exist in the response
    let remotes = body["remotes"].as_array();
    assert!(remotes.is_some(), "{:?}", remotes);
    let remotes = remotes.unwrap();
    assert!(
        !(remotes
            .iter()
            .any(|repo| repo["local_name"] == "lightningd")),
        "lightningd remote repository is found in the response while it should have been removed"
    );

    cln.stop().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(560000)]
pub async fn httpd_search_list_plugins() {
    init();

    let mut cln = Node::tmp("regtest").await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    let manager = CoffeeHTTPDTesting::tmp(lightning_dir.to_string()).await;
    assert!(manager.is_ok(), "{:?}", manager);
    let manager = manager.unwrap();
    log::info!("lightning path: {lightning_dir}");
    let url = manager.url();
    log::info!("base url: {url}");
    let client = reqwest::Client::new();

    // Define the request body to be sent to the /remote/add endpoint
    let remote_add_request = RemoteAdd {
        repository_name: "lightningd".to_string(),
        repository_url: "https://github.com/lightningd/plugins.git".to_string(),
    };

    let response = client
        .post(format!("{}/remote/add", url))
        .json(&remote_add_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/remote/add response: {}", body);

    // Define the request body to be sent to the /remote/list_plugins endpoint
    let remote_plugins_list_request = RemotePluginsList {
        repository_name: "lightningd".to_string(),
    };

    let response = client
        .get(format!("{}/remote/list_plugins", url))
        .json(&remote_plugins_list_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    let body = response.json::<serde_json::Value>().await;
    assert!(body.is_ok(), "{:?}", body);
    let body = body.unwrap();

    // Log the response body
    log::info!("/remote/list_plugins response: {}", body);

    // Assert the response plugin list is not empty
    let plugins = body["plugins"].as_array();
    assert!(plugins.is_some(), "{:?}", plugins);
    let plugins = plugins.unwrap();

    // Assert that the "helpme" plugin exists in the response
    assert!(
        plugins.iter().any(|plugin| plugin["name"] == "helpme"),
        "helpme plugin not found in the response"
    );

    // Define the request body to be sent to the /show endpoint
    let show_request = Show {
        plugin: "helpme".to_string(),
    };

    let response = client
        .get(format!("{}/show", url))
        .json(&show_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/show response: {}", body);

    // Parse the response body
    let response_json = serde_json::from_str(&body);
    assert!(response_json.is_ok(), "{:?}", response_json);
    let response_json: serde_json::Value = response_json.unwrap();

    // Extract the `readme` field from the response JSON
    let readme = response_json["readme"].as_str();
    assert!(readme.is_some(), "{:?}", readme);
    let readme = readme.unwrap();

    // Assert that the `readme` starts with the expected content
    assert!(readme.starts_with("# Helpme plugin"), "{:?}", readme);

    // Define the request body to be sent to the /search endpoint
    let search_request = Search {
        plugin: "summary".to_string(),
    };

    let response = client
        .get(format!("{}/search", url))
        .json(&search_request)
        .send()
        .await;
    assert!(response.is_ok(), "{:?}", response);
    let response = response.unwrap();

    // Check the response status code, log the body.
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    log::info!("/search response: {}", body);

    // Parse the response body
    let response_json = serde_json::from_str(&body);
    assert!(response_json.is_ok(), "{:?}", response_json);
    let response_json: serde_json::Value = response_json.unwrap();

    // Extract the `repository_url` field from the response JSON
    let repository_url = response_json["repository_url"].as_str();
    assert!(repository_url.is_some(), "{:?}", repository_url);
    let repository_url = repository_url.unwrap();

    // Assert that repository_url is the expected value
    assert_eq!(
        repository_url, "https://github.com/lightningd/plugins",
        "{:?}",
        repository_url
    );
    cln.stop().await.unwrap();
}
