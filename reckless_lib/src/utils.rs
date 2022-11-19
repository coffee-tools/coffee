use crate::errors::RecklessError;
use git2;
use log::debug;
use std::env;
use std::fs::create_dir_all;
use std::path::Path;

pub fn create_dir_in_home(relative_path: &str) {
    let mut path = env::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    path = path + "/" + relative_path;
    let path = Path::new(&path);
    match create_dir_all(path) {
        Ok(_) => {
            debug!("Successfully created directory at {}", path.display());
        }
        Err(err) => {
            println!("ERROR!: {:?}", err);
        }
    };
}

pub fn get_dir_path_from_url(url: &str) -> String {
    let path = env::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    return format!(
        "{}/.reckless{}",
        path,
        &url.split(".com").last().unwrap().trim().to_string()
    );
}

pub fn clone_recursive_fix(repo: git2::Repository, url: &str) -> Result<(), RecklessError> {
    let repository = repo.submodules().unwrap_or_default();
    debug!("SUBMODULE COUNT: {}", repository.len());
    for (index, sub) in repository.iter().enumerate() {
        debug!("URL {}: {}", index + 1, sub.url().unwrap());
        let path = format!(
            "{}/{}",
            get_dir_path_from_url(url),
            sub.path().to_str().unwrap()
        );
        match git2::Repository::clone(sub.url().unwrap(), path) {
            // Fix error handling
            Ok(_) => {
                debug!("ADDED {}", sub.url().unwrap());
                Ok(())
            }
            Err(err) => Err(RecklessError::new(1, err.message())),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::create_dir_in_home;
    use std::env;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init() {
        // ignore error
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    #[test]
    fn test_create_dir_in_home() {
        let dir = ".reckless";
        create_dir_in_home(dir);
        let mut path = env::home_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
            .to_owned();
        path = format!("{}/{}", path, dir);
        assert_eq!(Path::new(&path).exists(), true);
    }
}
