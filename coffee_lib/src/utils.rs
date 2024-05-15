use super::macros::error;
use std::path::{Path, PathBuf};

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
    log::trace!("check_dir_or_make_if_missing: `{path}`");
    if !Path::exists(Path::new(&path.to_owned())) {
        fs::create_dir(path.clone()).await?;
        log::debug!("created dir {path}");
    }
    Ok(())
}

pub async fn copy_dir_if_exist(origin: &str, destination: &str) -> Result<(), CoffeeError> {
    log::trace!("copy_dir_if_exist: from: `{origin}` to `{destination}`");
    if Path::exists(Path::new(&origin)) {
        copy_dir_recursive(origin.to_owned(), destination.to_owned()).await?;
        log::debug!("copying dir from {origin} to {destination}");
    }
    Ok(())
}

async fn copy_dir_recursive(source: String, destination: String) -> Result<(), CoffeeError> {
    async fn inner_copy_dir_recursive(
        source: PathBuf,
        destination: PathBuf,
    ) -> Result<(), CoffeeError> {
        check_dir_or_make_if_missing(destination.to_string_lossy().to_string()).await?;

        let mut entries = fs::read_dir(source).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            let dest_path = destination.join(entry.file_name());
            log::debug!("copy entry {:?} in {:?}", entry, dest_path);
            if file_type.is_dir() {
                // Here we use Box::pin to allow recursion
                let fut = inner_copy_dir_recursive(entry.path(), dest_path);
                Box::pin(fut).await?;
            } else if file_type.is_file() {
                fs::copy(entry.path(), &dest_path).await?;
            }
        }

        Ok(())
    }
    let source = Path::new(&source);
    let destination = Path::new(&destination);
    log::info!("{:?} - {:?}", source, destination);
    inner_copy_dir_recursive(source.to_path_buf(), destination.to_path_buf()).await
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
