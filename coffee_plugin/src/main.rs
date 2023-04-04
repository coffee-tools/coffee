use plugin::build_plugin;

mod plugin;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    build_plugin().await
}
