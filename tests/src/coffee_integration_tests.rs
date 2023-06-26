use std::collections::HashMap;
use std::sync::Once;

use coffee_lib::plugin_manager::PluginManager;
use coffee_testing::cln::Node;
use coffee_testing::CoffeeTesting;
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
pub async fn init_coffee_test() -> anyhow::Result<()> {
    init();

    let mut manager = CoffeeTesting::tmp().await?;
    let result = manager.coffee().list().await?;
    assert!(
        result.plugins.is_empty(),
        "list of plugin not empty: {:?}",
        result
    );
    Ok(())
}

#[tokio::test]
pub async fn init_coffee_test_with_cln() -> anyhow::Result<()> {
    init();
    let cln = Node::tmp().await?;

    let mut manager = CoffeeTesting::tmp().await?;
    let result = manager.coffee().list().await?;
    assert!(
        result.plugins.is_empty(),
        "list of plugin not empty: {:?}",
        result
    );
    let lightning_dir = cln.rpc().getinfo()?.ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");

    manager.coffee().setup(&lightning_dir).await?;

    Ok(())
}

#[tokio::test]
#[ntest::timeout(120000)]
pub async fn init_coffee_test_add_remote() {
    init();
    let mut cln = Node::tmp().await.unwrap();

    let mut manager = CoffeeTesting::tmp().await.unwrap();
    let result = manager.coffee().list().await.unwrap();
    assert!(
        result.plugins.is_empty(),
        "list of plugin not empty: {:?}",
        result
    );
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");

    manager.coffee().setup(&lightning_dir).await.unwrap();

    manager
        .coffee()
        .add_remote("lightningd", "https://github.com/lightningd/plugins.git")
        .await
        .unwrap();
    manager
        .coffee()
        .install("summary", true, true)
        .await
        .unwrap();

    cln.rpc()
        .call::<serde_json::Value, serde_json::Value>("summary", json!({}))
        .unwrap();

    cln.stop().await.unwrap();
}

#[tokio::test]
#[ntest::timeout(120000)]
pub async fn test_add_remove_plugins() {
    init();
    let mut cln = Node::tmp().await.unwrap();
    let mut manager = CoffeeTesting::tmp().await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");
    manager.coffee().setup(&lightning_dir).await.unwrap();

    // Add lightningd remote repository
    manager
        .coffee()
        .add_remote("lightningd", "https://github.com/lightningd/plugins.git")
        .await
        .unwrap();

    // Install summary plugin
    manager
        .coffee()
        .install("summary", true, true)
        .await
        .unwrap();

    // Install helpme plugin
    manager
        .coffee()
        .install("helpme", true, true)
        .await
        .unwrap();

    // Ensure that the list of remotes is correct
    let result = manager.coffee().list_remotes().await.unwrap();
    let remotes = result.remotes.expect("remotes field not found");
    assert_eq!(remotes.len(), 1, "Unexpected number of remote repositories");
    assert!(
        remotes
            .iter()
            .any(|remote| remote.local_name == "lightningd"),
        "Remote repository 'lightningd' not found"
    );

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await.unwrap();
    assert_eq!(result.plugins.len(), 2, "Unexpected number of plugins");
    assert!(
        result
            .plugins
            .iter()
            .any(|plugin| plugin.name() == "summary"),
        "Plugin 'summary' not found"
    );
    assert!(
        result
            .plugins
            .iter()
            .any(|plugin| plugin.name() == "helpme"),
        "Plugin 'helpme' not found"
    );

    // Remove summary plugin
    manager.coffee().remove("summary").await.unwrap();

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await.unwrap();
    assert_eq!(result.plugins.len(), 1, "Unexpected number of plugins");
    assert!(
        result
            .plugins
            .iter()
            .any(|plugin| plugin.name() == "helpme"),
        "Plugin 'helpme' not found"
    );

    // Remove lightningd remote repository
    // This should also remove the helpme plugin
    manager.coffee().rm_remote("lightningd").await.unwrap();

    // Ensure that the list of remotes is correct
    let result = manager.coffee().list_remotes().await.unwrap();
    let remotes = result.remotes.expect("remotes not found");
    assert_eq!(remotes.len(), 0, "Unexpected number of remote repositories");

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await.unwrap();
    assert_eq!(result.plugins.len(), 0, "Unexpected number of plugins");

    cln.stop().await.unwrap();
}
