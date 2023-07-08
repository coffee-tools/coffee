use coffee_testing::cln::Node;
use coffee_testing::CoffeeHTTPDTesting;

use crate::init;

#[tokio::test(flavor = "multi_thread")]
#[ntest::timeout(560000)]
pub async fn init_httpd_add_remote() {
    init();

    let mut cln = Node::tmp().await.unwrap();
    let manager = CoffeeHTTPDTesting::tmp().await.unwrap();

    let lightning_dir = cln.rpc().getinfo().unwrap().ligthning_dir;
    let lightning_dir = lightning_dir.strip_suffix("/regtest").unwrap();
    log::info!("lightning path: {lightning_dir}");

    let url = manager.url();
    log::info!("base url: {url}");

    let body = reqwest::get(format!("{url}/list"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    println!("Response body: {}", body);

    cln.stop().await.unwrap();
}
