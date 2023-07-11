use coffee_lib::errors::CoffeeError;
use coffee_lib::url::URL;
use log::debug;

use coffee_lib::types::response::UpgradeStatus;

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

pub fn fast_forward(path: &str, branch: &str) -> Result<UpgradeStatus, CoffeeError> {
    let repo = git2::Repository::open(path).map_err(|err| CoffeeError::new(1, err.message()))?;

    repo.find_remote("origin")
        .map_err(|err| CoffeeError::new(1, err.message()))?
        .fetch(&[branch], None, None)
        .map_err(|err| CoffeeError::new(1, err.message()))?;

    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|err| CoffeeError::new(1, err.message()))?;

    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_head)
        .map_err(|err| CoffeeError::new(1, err.message()))?;

    let analysis = repo
        .merge_analysis(&[&fetch_commit])
        .map_err(|err| CoffeeError::new(1, err.message()))?;

    if analysis.0.is_up_to_date() {
        Ok(UpgradeStatus::UpToDate)
    } else if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch);
        let mut reference = repo
            .find_reference(&refname)
            .map_err(|err| CoffeeError::new(1, err.message()))?;

        reference
            .set_target(fetch_commit.id(), "Fast-Forward")
            .map_err(|err| CoffeeError::new(1, err.message()))?;

        repo.set_head(&refname)
            .map_err(|err| CoffeeError::new(1, err.message()))?;

        match repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force())) {
            Ok(()) => return Ok(UpgradeStatus::Updated),
            Err(err) => return Err(CoffeeError::new(1, err.message())),
        }
    } else {
        Err(CoffeeError::new(
            1,
            "Error trying to pull the latest changes",
        ))
    }
}
