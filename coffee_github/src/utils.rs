use coffee_lib::errors::CoffeeError;
use coffee_lib::url::URL;
use log::debug;

pub async fn clone_recursive_fix(repo: git2::Repository, url: &URL) -> Result<(), CoffeeError> {
    let repository = repo.submodules().unwrap_or_default();
    debug!("submodule count: {}", repository.len());
    for (index, sub) in repository.iter().enumerate() {
        debug!("url {}: {}", index + 1, sub.url().unwrap());
        let path = format!("{}/{}", &url.path_string, sub.path().to_str().unwrap());
        match git2::Repository::clone(sub.url().unwrap(), &path) {
            // Fix error handling
            Ok(_) => {
                debug!("added {}", sub.url().unwrap());
                debug!("at path {}", &path);
                Ok(())
            }
            Err(err) => Err(CoffeeError::new(1, err.message())),
        }?;
    }
    Ok(())
}
