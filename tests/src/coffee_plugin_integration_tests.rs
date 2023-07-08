//! Coffee Plugin integration testing
use coffee_testing::cln::Node;

use crate::init;

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(560000)]
pub async fn init_cln_with_coffee_plugin_test() {
    init();

    let mut cln = Node::tmp().await.unwrap();

    let cargo_target = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let path = std::path::Path::new(cargo_target).to_str().unwrap();
    let plugin_path = format!("{path}/target/debug/coffee_plugin");
    log::info!("plugin path {plugin_path}");
    let result: serde_json::Value = cln
        .rpc()
        .call(
            "plugin",
            serde_json::json!({
                "subcommand": "start",
                "plugin": plugin_path,
            }),
        )
        .unwrap();
    log::info!("cln response {result}");
    cln.stop().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(560000)]
pub async fn init_cln_with_coffee_add_remore_test() {
    init();

    let mut cln = Node::tmp().await.unwrap();

    let cargo_target = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let path = std::path::Path::new(cargo_target).to_str().unwrap();
    let plugin_path = format!("{path}/target/debug/coffee_plugin");
    log::info!("plugin path {plugin_path}");

    let _: serde_json::Value = cln
        .rpc()
        .call(
            "plugin",
            serde_json::json!({
                "subcommand": "start",
                "plugin": plugin_path,
            }),
        )
        .unwrap();

    let result: serde_json::Value = cln
        .rpc()
        .call(
            "coffee_remote",
            serde_json::json!({
                "cmd": "add",
                "name": "folgore",
                "url": "https://github.com/coffee-tools/folgore.git",
            }),
        )
        .unwrap();
    log::info!("cln response {result}");
    cln.stop().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(1000000)]
pub async fn init_cln_with_coffee_install_plugin_test() {
    init();

    let mut cln = Node::tmp().await.unwrap();

    let cargo_target = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let path = std::path::Path::new(cargo_target).to_str().unwrap();
    let plugin_path = format!("{path}/target/debug/coffee_plugin");
    log::info!("plugin path {plugin_path}");

    let _: serde_json::Value = cln
        .rpc()
        .call(
            "plugin",
            serde_json::json!({
                "subcommand": "start",
                "plugin": plugin_path,
            }),
        )
        .unwrap();

    let result: serde_json::Value = cln
        .rpc()
        .call(
            "coffee_remote",
            serde_json::json!({
                "cmd": "add",
                "name": "official",
                "url": "https://github.com/lightningd/plugins.git",
            }),
        )
        .unwrap();
    log::info!("cln response {result}");

    let result: serde_json::Value = cln
        .rpc()
        .call(
            "coffee_install",
            serde_json::json!({
                "name": "helpme",
            }),
        )
        .unwrap();
    log::info!("cln response {result}");

    cln.stop().await.unwrap();
}
