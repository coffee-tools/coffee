use crate::errors::RecklessError;
use git2;
use log::debug;
use std::env;
use std::fs::create_dir_all;
use std::path::Path;
use std::path::PathBuf;

pub fn create_dir_in_home(relative_path: &str) {
    let mut path = env::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    path = format!("{}/{}", path, relative_path);
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

pub fn get_reckless_dir(dir: &str) -> String {
    let mut path = env::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    path = format!("{}/.reckless/{}", path, dir);
    path
}

pub fn get_dir_path_from_url(url: &str) -> String {
    return format!(
        "{}/{}",
        get_reckless_dir("repositories"),
        &url.split(".com")
            .last()
            .unwrap()
            .strip_prefix("/")
            .unwrap()
            .trim()
            .to_string()
    );
}

fn slice_from_end(string: &str, size: usize) -> Option<&str> {
    string
        .char_indices()
        .rev()
        .nth(size)
        .map(|(i, _)| &string[i..])
}

pub fn get_repo_name_from_url(url: &str) -> String {
    let mut repo_name = url.split("/").last().unwrap().to_string();
    if slice_from_end(url, 3).unwrap().to_string() == ".git" {
        repo_name = repo_name.strip_suffix(".git").unwrap().to_string();
    };
    repo_name
}

pub fn get_plugin_info_from_path(path: PathBuf) -> Result<(String, String), RecklessError> {
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
        init();
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
