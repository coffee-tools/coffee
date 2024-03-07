use super::macros::error;
use std::path::Path;

use tokio::fs;

use crate::errors::CoffeeError;

pub fn get_plugin_info_from_path(path: &Path) -> Result<(String, String), CoffeeError> {
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
        None => Err(error!("{}", "Incorrect path")),
    }
}

pub async fn check_dir_or_make_if_missing(path: String) -> Result<(), CoffeeError> {
    if !Path::exists(Path::new(&path.to_owned())) {
        fs::create_dir(path.clone()).await?;
        log::debug!("created dir {path}");
    }
    Ok(())
}

pub async fn copy_dir_if_exist(origin: &str, destination: &str) -> Result<(), CoffeeError> {
    if Path::exists(Path::new(&origin)) {
        fs::copy(origin, destination).await?;
        log::debug!("copying dir from {origin} to {destination}");
    }
    Ok(())
}

pub async fn rm_dir_if_exist(origin: &str) -> Result<(), CoffeeError> {
    if Path::exists(Path::new(&origin)) {
        fs::remove_dir_all(origin).await?;
        log::debug!("rm dir from {origin}");
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
        #![allow(deprecated)]
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
        let dir = ".coffee";
        let path = create_dir_in_home(dir);
        assert!(Path::new(&path).exists());
        remove_dir_all(path).unwrap();
    }
}
