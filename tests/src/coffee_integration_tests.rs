use std::sync::Once;

use coffee_lib::plugin_manager::PluginManager;
use coffee_testing::cln::Node;
use coffee_testing::prelude::tempfile;
use coffee_testing::{CoffeeTesting, CoffeeTestingArgs};
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
pub async fn init_coffee_test_cmd() -> anyhow::Result<()> {
    init();

    let dir = tempfile::tempdir()?;
    let args = CoffeeTestingArgs {
        conf: None,
        data_dir: dir.path().clone().to_str().unwrap().to_owned(),
        network: "bitcoin".to_string(),
    };
    let mut manager = CoffeeTesting::tmp_with_args(&args, dir.clone()).await?;
    let root_path = manager.root_path().to_owned();
    manager
        .coffee()
        .add_remote("folgore", "https://github.com/coffee-tools/folgore.git")
        .await
        .unwrap();

    let new_args = CoffeeTestingArgs {
        conf: None,
        data_dir: dir.path().clone().to_string_lossy().to_string(),
        network: "testnet".to_string(),
    };
    let mut manager = CoffeeTesting::tmp_with_args(&new_args, dir.clone()).await?;
    let new_root_path = manager.root_path().to_owned();
    assert_eq!(root_path.path(), new_root_path.path());

    let actual_network = manager.coffee().storage_info().config.network;
    let expected_network = "testnet".to_string();

    assert_eq!(
        actual_network, expected_network,
        "Network is wrong. Actual: '{actual_network}', Expected: '{expected_network}'"
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
    let repo_name = "lightningd";
    let repo_url = "https://github.com/lightningd/plugins.git";
    manager
        .coffee()
        .add_remote(repo_name, repo_url)
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
    let remotes = result.remotes.unwrap();
    log::debug!("remotes: {:?}", remotes);
    assert_eq!(remotes.len(), 1, "Unexpected number of remote repositories");
    assert!(
        remotes.iter().any(|remote| remote.local_name == repo_name),
        "Remote repository 'lightningd' not found"
    );

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await.unwrap();
    log::debug!("plugins: {:?}", result.plugins);
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
    log::debug!("plugins: {:?}", result.plugins);
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
    manager.coffee().rm_remote(repo_name).await.unwrap();

    // Ensure that the list of remotes is correct
    let result = manager.coffee().list_remotes().await.unwrap();
    let remotes = result.remotes.unwrap();
    log::debug!("remotes: {:?}", remotes);
    assert_eq!(remotes.len(), 0, "Unexpected number of remote repositories");

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await.unwrap();
    log::debug!("plugins: {:?}", result.plugins);
    assert_eq!(result.plugins.len(), 0, "Unexpected number of plugins");

    cln.stop().await.unwrap();
}
