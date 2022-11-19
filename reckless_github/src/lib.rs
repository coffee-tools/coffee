//! Github repository implementation

pub mod repository;

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Once};

    use reckless_lib::repository::Repository;
    use reckless_lib::utils::get_dir_path_from_url;

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
        let url = "https://github.com/lightningd/plugins";
        let repo = Github::new(name, url);
        let repo = repo.init().await;
        assert!(repo.is_ok());
        assert_eq!(Path::new(&(get_dir_path_from_url(url))).exists(), true);
    }
}
