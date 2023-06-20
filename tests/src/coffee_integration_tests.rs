use std::collections::HashMap;
use std::sync::Once;

use coffee_lib::plugin_manager::PluginManager;
use coffee_testing::cln::Node;
use coffee_testing::CoffeeTesting;

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
#[ntest::timeout(60000)]
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
        .call::<_, HashMap<String, String>>("summary", HashMap::<String, String>::new())
        .unwrap();

    cln.stop().await.unwrap();
}
