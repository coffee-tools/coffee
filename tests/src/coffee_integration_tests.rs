use std::path::Path;
use std::sync::Arc;
use tokio::fs;

use serde_json::json;

use coffee_lib::plugin_manager::PluginManager;
use coffee_lib::types::response::{Defect, NurseStatus};
use coffee_testing::cln::Node;
use coffee_testing::prelude::tempfile;
use coffee_testing::{CoffeeTesting, CoffeeTestingArgs};

use crate::init;

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

    let dir = Arc::new(tempfile::tempdir()?);
    let args = CoffeeTestingArgs {
        conf: None,
        data_dir: dir.path().to_str().unwrap().to_owned(),
        network: "bitcoin".to_string(),
    };
    let mut manager = CoffeeTesting::tmp_with_args(&args, dir.clone()).await?;
    let root_path = manager.root_path().to_owned();
    manager
        .coffee()
        .add_remote("folgore", "https://github.com/coffee-tools/folgore.git")
        .await
        .unwrap();

    // dropping the first coffee instance, but without delete the dir
    drop(manager);
    let new_args = CoffeeTestingArgs {
        conf: None,
        data_dir: dir.path().to_string_lossy().to_string(),
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
    let cln = Node::tmp("regtest").await?;

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
#[ntest::timeout(560000)]
pub async fn init_coffee_test_add_remote() {
    init();
    let mut cln = Node::tmp("regtest").await.unwrap();

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
        .install("summary", true, true, None)
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

    let mut cln = Node::tmp("regtest").await.unwrap();
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

    // Get the list of plugins available in the remote repository
    let result = manager.coffee().get_plugins_in_remote(repo_name).await;
    assert!(result.is_ok(), "{:?}", result);
    let result = result.unwrap();
    let plugins = result.plugins;
    // Assert the length of the list of plugins is greater than 0
    assert!(
        plugins.len() > 0,
        "The list of plugins is empty: {:?}",
        plugins
    );
    // Assert that the list of plugins contains the summary  and helpme plugin
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "summary"),
        "Plugin 'summary' not found"
    );
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "helpme"),
        "Plugin 'helpme' not found"
    );

    // Install summary plugin
    let result = manager.coffee().install("summary", true, false, None).await;
    assert!(result.is_ok(), "{:?}", result);

    // Install helpme plugin
    manager
        .coffee()
        .install("helpme", true, false, None)
        .await
        .unwrap();

    // Ensure that the list of remotes is correct
    let result = manager.coffee().list_remotes().await;
    assert!(result.is_ok(), "list_remotes failed. result: {:?}", result);
    let remotes = result.unwrap().remotes.unwrap();
    log::debug!("remotes: {:?}", remotes);
    assert_eq!(remotes.len(), 1, "Unexpected number of remote repositories");
    assert!(
        remotes.iter().any(|remote| remote.local_name == repo_name),
        "Remote repository 'lightningd' not found"
    );

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await;
    assert!(result.is_ok(), "{:?}", result);
    let plugins = result.unwrap().plugins;
    log::debug!("plugins: {:?}", plugins);
    assert_eq!(plugins.len(), 2, "{:?}", plugins);
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "summary"),
        "Plugin 'summary' not found"
    );
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "helpme"),
        "Plugin 'helpme' not found"
    );

    // Remove summary plugin
    let result = manager.coffee().remove("summary").await;
    assert!(result.is_ok(), "{:?}", result);

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await;
    assert!(result.is_ok(), "{:?}", result);

    let plugins = result.unwrap().plugins;
    assert_eq!(plugins.len(), 1, "{:?}", plugins);
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "helpme"),
        "Plugin 'helpme' not found"
    );

    // Remove lightningd remote repository
    // This should also remove the helpme plugin
    let result = manager.coffee().rm_remote(repo_name).await;
    assert!(result.is_ok(), "{:?}", result);

    // Ensure that the list of remotes is correct
    let result = manager.coffee().list_remotes().await;
    assert!(result.is_ok(), "{:?}", result);

    let remotes = result.unwrap().remotes.clone();
    assert!(remotes.is_some(), "{:?}", remotes);
    let remotes = remotes.unwrap();
    assert_eq!(remotes.len(), 0, "{:?}", remotes);

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await;
    assert!(result.is_ok(), "{:?}", result);
    let plugins = result.unwrap().plugins;
    log::debug!("plugins: {:?}", plugins);
    assert_eq!(plugins.len(), 0, "{:?}", plugins);

    cln.stop().await.unwrap();
}

#[tokio::test]
#[ntest::timeout(120000)]
pub async fn test_errors_and_show() {
    init();

    let mut cln = Node::tmp("regtest").await.unwrap();
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

    // Search for summary plugin
    let result = manager.coffee().search("summary").await;
    assert!(result.is_ok(), "{:?}", result);
    let result = result.unwrap();
    let repo_url = result.repository_url.as_str();
    assert_eq!(
        repo_url, "https://github.com/lightningd/plugins",
        "{:?}",
        repo_url
    );

    // Install summary plugin
    let result = manager.coffee().install("summary", true, false, None).await;
    assert!(result.is_ok(), "{:?}", result);

    // Get the README file for a plugin that is not installed
    let result = manager.coffee().show("helpme").await.unwrap();
    let val = result.readme.as_str();
    assert!(val.starts_with("# Helpme plugin"));

    // Install a plugin that is not in the repository
    let result = manager.coffee().install("x", true, false, None).await;
    assert!(result.is_err(), "{:?}", result);

    // Remove helpme plugin
    // This should fail because it is not installed
    let result = manager.coffee().remove("helpme").await;
    assert!(result.is_err(), "{:?}", result);

    // Remove folgore remote repository
    // This should also fail
    let result = manager.coffee().rm_remote("folgore").await;
    assert!(result.is_err(), "{:?}", result);

    // Ensure that the list of remotes is correct
    let result = manager.coffee().list_remotes().await;
    assert!(result.is_ok(), "list_remotes failed. result: {:?}", result);
    let remotes = result.unwrap().remotes.unwrap();
    log::debug!("remotes: {:?}", remotes);
    assert_eq!(remotes.len(), 1, "{:?}", remotes);
    assert!(
        remotes.iter().any(|remote| remote.local_name == repo_name),
        "{:?}",
        remotes
    );

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await;
    assert!(result.is_ok(), "{:?}", result);
    let plugins = result.unwrap().plugins;
    log::debug!("plugins: {:?}", plugins);
    assert_eq!(plugins.len(), 1, "{:?}", plugins);
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "summary"),
        "{:?}",
        plugins
    );

    cln.stop().await.unwrap();
}

#[tokio::test]
pub async fn install_plugin_in_two_networks() -> anyhow::Result<()> {
    init();
    // initialize a lightning node in regtest network
    let mut cln = Node::tmp("regtest").await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_regtest_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path for regtest network: {lightning_dir}");

    let dir = Arc::new(tempfile::tempdir()?);
    let args = CoffeeTestingArgs {
        conf: None,
        data_dir: dir.path().to_str().unwrap().to_owned(),
        network: "regtest".to_string(),
    };
    let mut manager = CoffeeTesting::tmp_with_args(&args, dir.clone()).await?;
    let result = manager.coffee().setup(&lightning_regtest_dir).await;
    assert!(result.is_ok(), "{:?}", result);
    // Add lightningd remote repository
    manager
        .coffee()
        .add_remote("lightningd", "https://github.com/lightningd/plugins.git")
        .await
        .unwrap();
    // Install summary plugin
    // This should install summary plugin for regtest network
    manager
        .coffee()
        .install("summary", true, true, None)
        .await
        .unwrap();
    // Ensure that summary is installed for regtest network
    cln.rpc()
        .call::<serde_json::Value, serde_json::Value>("summary", json!({}))
        .unwrap();
    cln.stop().await.unwrap();

    // dropping the first coffee instance, but without delete the dir
    drop(manager);
    log::info!("------ second run -------");
    // initialize a lightning node in testnet network
    let mut cln = Node::tmp("testnet").await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_testnet_dir = lightning_dir.strip_suffix("/testnet").unwrap();
    log::info!("lightning path for testnet network: {lightning_dir}");
    let new_args = CoffeeTestingArgs {
        conf: None,
        data_dir: dir.path().to_string_lossy().to_string(),
        network: "testnet".to_string(),
    };
    let mut manager = CoffeeTesting::tmp_with_args(&new_args, dir.clone()).await?;
    let new_root_path = manager.root_path().to_owned();
    assert_eq!(dir.path(), new_root_path.path());
    manager
        .coffee()
        .setup(&lightning_testnet_dir)
        .await
        .unwrap();

    let result = manager
        .coffee()
        .add_remote("lightningd", "https://github.com/lightningd/plugins.git")
        .await;
    assert!(result.is_err(), "{:?}", result);
    // Install summary plugin
    // This should install summary plugin for testnet network
    manager
        .coffee()
        .install("summary", true, true, None)
        .await
        .unwrap();
    // Ensure that summary is installed for testnet network
    cln.rpc()
        .call::<serde_json::Value, serde_json::Value>("summary", json!({}))
        .unwrap();

    cln.stop().await.unwrap();
    Ok(())
}

#[tokio::test]
#[ntest::timeout(560000)]
pub async fn test_double_slash() {
    init();

    let cln = Node::tmp("regtest").await.unwrap();
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
    let result = manager.coffee().install("summary", true, false, None).await;
    assert!(result.is_ok(), "{:?}", result);

    // Install helpme plugin
    manager
        .coffee()
        .install("helpme", true, false, None)
        .await
        .unwrap();

    // Ensure that the list of plugins is correct
    let result = manager.coffee().list().await;
    assert!(result.is_ok(), "{:?}", result);
    let plugins = result.unwrap().plugins;
    log::debug!("plugins: {:?}", plugins);
    assert_eq!(plugins.len(), 2, "{:?}", plugins);
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "summary"),
        "Plugin 'summary' not found"
    );
    assert!(
        plugins.iter().any(|plugin| plugin.name() == "helpme"),
        "Plugin 'helpme' not found"
    );

    // Assert that root_path and exec_path of all plugins don't have "//"
    for plugin in &plugins {
        assert!(
            !plugin.root_path.contains("//"),
            "Double slash found in root_path of plugin '{}': {}",
            plugin.name(),
            plugin.root_path
        );
        assert!(
            !plugin.exec_path.contains("//"),
            "Double slash found in exec_path of plugin '{}': {}",
            plugin.name(),
            plugin.exec_path
        );
    }
}

#[tokio::test]
#[ntest::timeout(560000)]
pub async fn test_plugin_installation_path() {
    init();
    let mut cln = Node::tmp("regtest").await.unwrap();

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

    // Install summary plugin for regtest network
    manager
        .coffee()
        .install("summary", true, false, None)
        .await
        .unwrap();

    let root_path = manager
        .root_path()
        .to_owned()
        .path()
        .to_str()
        .map(String::from);
    assert!(root_path.is_some(), "{:?}", root_path);
    let root_path = root_path.unwrap();

    // Construct the path of the regtest config file
    let regtest_config_file = format!("{}/.coffee/regtest/coffee.conf", root_path);
    let regtest_config_file = Path::new(&regtest_config_file);

    // Check if the regtest config file exists
    assert!(
        regtest_config_file.exists(),
        "The file {:?} does not exist",
        regtest_config_file
    );

    let regtest_config_file_content = fs::read_to_string(&regtest_config_file).await;
    assert!(
        regtest_config_file_content.is_ok(),
        "{:?}",
        regtest_config_file_content
    );
    let regtest_config_file_content = regtest_config_file_content.unwrap();

    log::info!(
        "regtest_config_file_content: {}",
        regtest_config_file_content
    );

    // Extract the executable path of the summary plugin
    let exec_path = regtest_config_file_content
        .lines()
        .find(|line| line.starts_with("plugin="))
        .and_then(|line| line.split('=').nth(1))
        .map(String::from);
    assert!(exec_path.is_some(), "{:?}", exec_path);
    let exec_path = exec_path.unwrap();
    log::debug!("summary plugin installation execution path: {}", exec_path);
    let exec_path = Path::new(&exec_path);

    // Check if the executable path of the summary plugin exists
    assert!(
        exec_path.exists(),
        "The file {:?} does not exist",
        exec_path
    );

    // Remove summary plugin
    let result = manager.coffee().remove("summary").await;
    assert!(result.is_ok(), "{:?}", result);

    // Check if the executable path of the summary plugin exists
    // after the removal of the plugin
    assert!(
        !exec_path.exists(),
        "The file {:?} exists where it should have been removed",
        exec_path
    );

    // Construct the path of the summary plugin cloned version
    let summary_exec_path = format!(
        "{}/.coffee/repositories/lightningd/summary/summary.py",
        root_path
    );
    let summary_exec_path = Path::new(&summary_exec_path);
    // Check if the executable path of the summary plugin exists
    // This is the cloned version of the plugin (not specific to regtest network)
    // This should not be removed
    assert!(
        summary_exec_path.exists(),
        "The file {:?} does not exist",
        summary_exec_path
    );

    cln.stop().await.unwrap();
}

#[tokio::test]
#[ntest::timeout(560000)]
pub async fn test_nurse_repository_missing_on_disk() {
    init();
    let mut cln = Node::tmp("regtest").await.unwrap();

    let mut manager = CoffeeTesting::tmp().await.unwrap();
    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");

    manager.coffee().setup(&lightning_dir).await.unwrap();

    // Construct the root path
    let root_path = manager
        .root_path()
        .to_owned()
        .path()
        .to_str()
        .map(String::from);
    assert!(root_path.is_some(), "{:?}", root_path);
    let root_path = root_path.unwrap();

    // Add folgore remote repository
    manager
        .coffee()
        .add_remote("folgore", "https://github.com/coffee-tools/folgore.git")
        .await
        .unwrap();

    // Construct the path of the folgore repository
    let folgore_path = format!("{}/.coffee/repositories/folgore", root_path);
    let folgore_path = Path::new(&folgore_path);
    // Check if the folgore repository exists
    assert!(
        folgore_path.exists(),
        "The folder {:?} does not exist",
        folgore_path
    );

    // Make sure that the folgore repository has README.md file
    // This is to ensure that the repository is cloned correctly later
    // (not just an empty folder)
    let folgore_readme_path = format!("{}/.coffee/repositories/folgore/README.md", root_path);
    let folgore_readme_path = Path::new(&folgore_readme_path);
    // Check if the folgore repository has README.md file
    assert!(
        folgore_readme_path.exists(),
        "The file {:?} does not exist",
        folgore_readme_path
    );

    // Assert that nurse returns that coffee is Sane
    let result = manager.coffee().nurse().await;
    assert!(result.is_ok(), "{:?}", result);
    let result = result.unwrap();
    // Assert that the value is Sane
    assert!(result.is_sane());

    // Remove folgore repository (we militate that the repository is missing on disk)
    let result = fs::remove_dir_all(&folgore_path).await;
    assert!(result.is_ok(), "{:?}", result);

    // Assert that folgore repository is missing on disk
    assert!(
        !folgore_path.exists(),
        "The folder {:?} exists",
        folgore_path
    );

    // Assert that nurse --verify returns that coffee is corrupt
    let result = manager.coffee().nurse_verify().await;
    assert!(result.is_ok(), "{:?}", result);
    let result = result.unwrap();
    // Assert that the value is Corrupt
    let defects = result.defects;
    assert_eq!(defects.len(), 1, "{:?}", defects);
    assert_eq!(
        defects[0],
        Defect::RepositoryLocallyAbsent(vec!["folgore".to_string()]),
        "{:?}",
        defects
    );

    // Run nurse again
    // Assert that nurse returns that coffee isn't Sane
    let result = manager.coffee().nurse().await;
    assert!(result.is_ok(), "{:?}", result);
    let result = result.unwrap();
    // Assert result has only 1 value
    assert_eq!(result.status.len(), 1, "{:?}", result);
    // Assert that the value is RepositoryLocallyRestored
    assert_eq!(
        result.status[0],
        NurseStatus::RepositoryLocallyRestored(vec!["folgore".to_string()]),
        "{:?}",
        result
    );

    // Assert that the folgore repository is cloned again
    assert!(
        folgore_path.exists(),
        "The folder {:?} does not exist",
        folgore_path
    );

    // Assert that the folgore repository has README.md file
    assert!(
        folgore_readme_path.exists(),
        "The file {:?} does not exist",
        folgore_readme_path
    );

    // Assert that nurse --verify returns that coffee is Sane
    let result = manager.coffee().nurse_verify().await;
    assert!(result.is_ok(), "{:?}", result);
    let result = result.unwrap();
    // Assert that the value is Sane
    assert!(result.is_sane());

    cln.stop().await.unwrap();
}
