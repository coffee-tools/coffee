//! Github repository implementation

pub mod repository;
mod utils;

pub use utils::{git_upgrade_with_branch, git_upgrade_with_git_head};

#[cfg(test)]
mod tests {
    use std::{path::Path, sync::Once};

    use coffee_lib::repository::Repository;
    use coffee_lib::url::URL;
    use std::fs::remove_dir_all;

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
        let name = "cln_plugins";
        let url = URL::new(
            "/tmp",
            "https://github.com/lightningd/plugins",
            "lightningd_plugins",
        );
        let mut repo = Github::new(name, &url);
        let repo = repo.init().await;
        assert!(repo.is_ok());
        assert!(Path::new(&url.path_string).exists());
        remove_dir_all(&url.path_string).unwrap();
    }
}
