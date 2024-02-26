use log::debug;
use tokio::process::Command;

use coffee_lib::errors::CoffeeError;
use coffee_lib::macros::error;
use coffee_lib::url::URL;
use coffee_lib::{commit_id, get_repo_info, sh};

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
            Err(err) => Err(error!("{}", err.message())),
        }?;
    }
    Ok(())
}

pub async fn git_upgrade(
    path: &str,
    branch: &str,
    verbose: bool,
) -> Result<UpgradeStatus, CoffeeError> {
    let repo = git2::Repository::open(path).map_err(|err| error!("{}", err.message()))?;

    let (local_commit, _) = get_repo_info!(repo);

    let mut cmd = format!("git fetch origin\n");
    cmd += &format!("git reset --hard origin/{branch}");
    sh!(path, cmd, verbose);

    let (upstream_commit, date) = get_repo_info!(repo);

    if local_commit == upstream_commit {
        Ok(UpgradeStatus::UpToDate(upstream_commit, date))
    } else {
        Ok(UpgradeStatus::Updated(upstream_commit, date))
    }
}

pub async fn git_checkout(
    path: &str,
    branch: &str,
    verbose: bool,
) -> Result<UpgradeStatus, CoffeeError> {
    let repo = git2::Repository::open(path).map_err(|err| error!("{}", err.message()))?;
    let (local_commit, _) = get_repo_info!(repo);

    let mut cmd = format!("git fetch origin\n");
    cmd += &format!("git reset --hard\n");
    cmd += &format!("git checkout origin/{branch}");
    sh!(path, cmd, verbose);

    let (upstream_commit, date) = get_repo_info!(repo);

    if local_commit == upstream_commit {
        Ok(UpgradeStatus::UpToDate(upstream_commit, date))
    } else {
        Ok(UpgradeStatus::Updated(upstream_commit, date))
    }
}
