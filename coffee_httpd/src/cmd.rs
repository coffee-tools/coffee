//! Coffee http daemon arguments definition.
use clap::Parser;

/// Coffee main command line definition for the command line tools.
#[derive(Debug, Parser)]
#[clap(name = "coffee_httpd")]
#[clap(about = "Coffee HTTP daemon")]
pub struct HttpdArgs {
    #[clap(long, value_parser)]
    pub conf: Option<String>,
    #[clap(long, value_parser)]
    pub network: Option<String>,
    #[clap(long, value_parser)]
    pub cln_path: String,
    #[clap(long, value_parser)]
    pub data_dir: Option<String>,
    #[clap(long, value_parser)]
    pub host: Option<String>,
    #[clap(long, value_parser)]
    pub port: Option<u64>,
}

impl coffee_core::CoffeeArgs for HttpdArgs {
    fn command(&self) -> coffee_core::CoffeeOperation {
        unimplemented!()
    }

    fn conf(&self) -> Option<String> {
        self.conf.clone()
    }

    fn data_dir(&self) -> Option<String> {
        self.data_dir.clone()
    }

    fn network(&self) -> Option<String> {
        self.network.clone()
    }

    // We don't need to verify the nurse in the httpd
    // daemon.
    // there is no endpoint for `nurse` in the httpd
    fn skip_verify(&self) -> bool {
        true
    }
}
