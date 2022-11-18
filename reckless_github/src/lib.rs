//! Github repository implementation

pub mod repository;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use reckless_lib::{repository::Repository, utils::get_dir_path_from_url};

    use crate::repository::Github;

    #[tokio::test]
    async fn repository_is_initialized_ok() {
        let name = "hello";
        let url = "https://github.com/lightningd/plugins";
        let repo = Github::new(name, url);
        let repo = repo.init().await;
        assert!(repo.is_ok());
        assert_eq!(Path::new(&(get_dir_path_from_url(url))).exists(), true);
    }
}
