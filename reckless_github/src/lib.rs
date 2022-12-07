//! Github repository implementation

pub mod repository;

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Once};

    use reckless_lib::repository::Repository;
    use reckless_lib::url::URL;

    use crate::repository::Github;

    static INIT: Once = Once::new();

    fn init() {
        // ignore error
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    #[tokio::test]
    async fn repository_is_initialized_ok() {
        init();
        let name = "hello";
        let url = URL::new(
            "/tmp",
            "https://github.com/lightningd/plugins",
            "lightningd_plugins",
        );
        let mut repo = Github::new(name, &url);
        let repo = repo.init().await;
        assert!(repo.is_ok());
        assert_eq!(Path::new(&url.path_string).exists(), true);
    }
}
