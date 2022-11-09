use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Link {
    git: Option<String>,
    html: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    name: String,
    path: String,
    sha: String,
    size: i32,
    html_url: Option<String>,
    git_url: Option<String>,
    download_url: Option<String>,
    r#type: String,
    _links: Link,
}

impl Content {
    pub fn name(&self) -> String {
        return self.name.clone();
    }
}

#[derive(Serialize, Deserialize)]
struct Commit {
    sha: String,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct Branch {
    name: String,
    commit: Commit,
    protected: bool,
}
