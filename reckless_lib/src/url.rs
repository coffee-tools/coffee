use std::fmt;

use crate::utils::get_reckless_dir;

/// This struct will make sure our URL's are of the
/// correct format and will also check correctness
/// of associated fields
#[derive(Clone)]
pub struct URL {
    /// the url name in case of remote
    pub name: String,
    /// the url string
    pub url_string: String,
    /// the reckless path associated with the url
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

/// Handle Reckless non-compliant URLs
fn handle_incorrect_url(mut url: &str) -> String {
    url = remove_trailing_slash_from_url(&url);
    url = remove_dot_git_from_url(&url);
    url.to_string()
}

/// Get repo_name field from the URL
fn get_repo_name_from_url(url: &str) -> String {
    let repo_name = url.split("/").last().unwrap().to_string();
    repo_name
}

/// Get path field from the URL
fn get_path_from_url(url: &str, remote_name: Option<&str>) -> String {
    match remote_name {
        Some(name) => return format!("{}/{}", get_reckless_dir("repositories"), name),
        None => {
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
            )
        }
    };
}

impl URL {
    /// Build a new URL and initialize its fields
    pub fn new(url: &str, remote_name: Option<&str>) -> Self {
        match remote_name {
            Some(name) => URL {
                name: name.to_string(),
                url_string: handle_incorrect_url(&url),
                path_string: get_path_from_url(&url, Some(name)),
                repo_name: get_repo_name_from_url(&url),
            },
            None => URL {
                name: get_repo_name_from_url(&url),
                url_string: handle_incorrect_url(&url),
                path_string: get_path_from_url(&url, None),
                repo_name: get_repo_name_from_url(&url),
            },
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
    use crate::url::get_path_from_url;

    use super::URL;

    #[test]
    fn test_remote() {
        let u = "https://github.com/lightningd/plugins";
        let url = URL::new(u, Some("lightningd_plugins"));
        assert_eq!(url.repo_name, "plugins");
        assert_eq!(url.url_string, u);
        assert_eq!(
            url.path_string,
            get_path_from_url(u, Some("lightningd_plugins"))
        );
        println!("{}", &url);
    }

    #[test]
    fn test_plugin() {
        let u = "https://github.com/lightningd/plugins";
        let url = URL::new(u, None);
        assert_eq!(url.repo_name, "plugins");
        assert_eq!(url.url_string, u);
        assert_eq!(url.path_string, get_path_from_url(u, None));
        println!("{}", &url);
    }
}
