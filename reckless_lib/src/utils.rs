use crate::errors::RecklessError;
use crate::url::URL;
use git2;
use log::debug;
use std::path::Path;

pub fn get_plugin_info_from_path(path: &Path) -> Result<(String, String), RecklessError> {
    match path.parent() {
        Some(parent_path) => {
            let path_to_plugin = parent_path.to_path_buf().to_string_lossy().to_string();
            let plugin_name = parent_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            Ok((path_to_plugin, plugin_name))
        }
        None => Err(RecklessError::new(1, "Incorrect path")),
    }
}

pub fn clone_recursive_fix(repo: git2::Repository, url: &URL) -> Result<(), RecklessError> {
    let repository = repo.submodules().unwrap_or_default();
    debug!("SUBMODULE COUNT: {}", repository.len());
    for (index, sub) in repository.iter().enumerate() {
        debug!("URL {}: {}", index + 1, sub.url().unwrap());
        let path = format!("{}/{}", &url.path_string, sub.path().to_str().unwrap());
        match git2::Repository::clone(sub.url().unwrap(), &path) {
            // Fix error handling
            Ok(_) => {
                debug!("ADDED {}", sub.url().unwrap());
                debug!("AT PATH {}", &path);
                Ok(())
            }
            Err(err) => Err(RecklessError::new(1, err.message())),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::create_dir_all;

    use std::env;
    use std::fs::remove_dir_all;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init() {
        // ignore error
        INIT.call_once(|| {
            env_logger::init();
        });
    }

    fn create_dir_in_home(relative_path: &str) -> String {
        let mut path = env::home_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap();
        path = format!("{}/{}", path, relative_path);
        let os_path = Path::new(&path);
        create_dir_all(os_path).unwrap();
        path
    }

    #[test]
    fn test_create_dir_in_home() {
        init();
        let dir = ".reckless";
        let path = create_dir_in_home(dir);
        assert_eq!(Path::new(&path).exists(), true);
        remove_dir_all(path).unwrap();
    }
}
