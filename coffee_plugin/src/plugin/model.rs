//! Model to encode and decode the core lightning plugin response!
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct InstallReq {
    pub(crate) name: String,
}

unsafe impl Sync for InstallReq {}

#[derive(Deserialize)]
pub(crate) struct RemoteReq {
    pub(crate) cmd: String,
    pub(crate) name: String,
    pub(crate) url: Option<String>,
}

unsafe impl Sync for RemoteReq {}

pub(crate) enum RemoteCmd {
    Add,
    Rm,
}

impl TryFrom<String> for RemoteCmd {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "add" => Ok(Self::Add),
            "rm" => Ok(Self::Rm),
            _ => Err(format!("command {value} not supported")),
        }
    }
}

impl RemoteReq {
    pub fn cmd(&self) -> Result<RemoteCmd, String> {
        RemoteCmd::try_from(self.cmd.clone())
    }

    pub fn url(&self) -> String {
        self.url.clone().unwrap()
    }
}
