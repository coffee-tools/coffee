use std::fmt;

use serde::{Deserialize, Serialize};

/// This struct will make sure our URL's are of the
/// correct format and will also check correctness
/// of associated fields
#[derive(Clone, Serialize, Deserialize)]
pub struct URL {
    /// the url name in case of remote
    pub name: String,
    /// the url string
    pub url_string: String,
    /// the coffee path associated with the url
    pub path_string: String,
    /// the repo name associated with the url
    pub repo_name: String,
}

/// Handle GitHub HTTP links
fn remove_dot_git_from_url(url: &str) -> &str {
    match url.strip_suffix(".git") {
        Some(s) => s,
        None => url,
    }
}

/// Handle URLs with a trailing "/"
fn remove_trailing_slash_from_url(url: &str) -> &str {
    match url.strip_suffix("/") {
        Some(s) => s,
        None => url,
    }
}

/// Handle coffee non-compliant URLs
fn handle_incorrect_url(mut url: &str) -> String {
    url = remove_trailing_slash_from_url(url);
    url = remove_dot_git_from_url(url);
    url.to_string()
}

/// Get repo_name field from the URL
fn get_repo_name_from_url(url: &str) -> String {
    let repo_name = url.split("/").last().unwrap().to_string();
    repo_name
}

impl URL {
    /// Build a new URL and initialize its fields
    pub fn new(local_path: &str, url: &str, remote_name: &str) -> Self {
        URL {
            name: remote_name.to_owned(),
            url_string: handle_incorrect_url(url),
            path_string: format!("{local_path}/repositories/{remote_name}"),
            repo_name: get_repo_name_from_url(url),
        }
    }
}

impl fmt::Display for URL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "repo_name: {}, url: {}, path: {}",
            self.repo_name, self.url_string, self.path_string
        )
    }
}

#[cfg(test)]
mod tests {
    use super::URL;

    #[test]
    fn test_remote() {
        let u = "https://github.com/lightningd/plugins";
        let url = URL::new("/tmp/", u, "lightningd_plugins");
        assert_eq!(url.repo_name, "plugins");
        assert_eq!(url.url_string, u);
        println!("{}", &url);
    }
}
