//! Github repository implementation

pub mod repository;

#[cfg(test)]
mod tests {
    use reckless_lib::repository::Repository;

    use crate::repository::Github;

    // FIXME: move this as async test
    #[test]
    fn repository_is_initialized_ok() {
        let repo = Github::new("mock_repo", "cool url");
        let repo = repo.init();

        // FIXME: check whatever give you information regarding
        // the init with success
        assert!(false);
    }
}
