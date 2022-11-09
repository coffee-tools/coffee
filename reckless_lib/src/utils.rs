use error_chain::error_chain;
use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::{env, fs::create_dir_all, path::Path};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[allow(dead_code)]
fn create_dir_in_home(relative_path: &str) {
    let mut path = env::home_dir()
        .unwrap()
        .into_os_string()
        .into_string()
        .unwrap();
    path = path + "/" + relative_path;
    let path = Path::new(&path);
    match create_dir_all(path) {
        Ok(_) => {
            println!("DONE!")
        }
        Err(err) => {
            println!("ERROR!: {:?}", err);
        }
    };
}

#[allow(dead_code)]
fn create_dir(absolute_path: &str) {
    match create_dir_all(absolute_path) {
        Ok(_) => {
            println!("DONE!")
        }
        Err(err) => {
            println!("ERROR!: {:?}", err);
        }
    };
}

#[allow(dead_code)]
#[tokio::main]
async fn download_github_repo() -> Result<()> {
    let url = "https://github.com/dart-lightning/lndart.cln/archive/refs/heads/main.zip";
    let response = reqwest::get(url).await?;
    let path = Path::new("./plugin.zip");
    let mut file = match File::create(&path) {
        Err(err) => panic!("ERROR! {}", err),
        Ok(file) => file,
    };
    let content = response.bytes().await?;
    file.write_all(&content)?;
    Ok(())
}

#[allow(dead_code)]
#[tokio::main]
async fn get_files(url: &str) {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "reckless-rs")
        .send()
        .await
        .unwrap();

    let mut res = Vec::new();
    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<Vec<data::Content>>().await {
                Ok(parsed) => res = parsed,
                Err(_) => println!("ERROR WHILE PARSING!"),
            };
        }
        _ => {
            panic!("UNEXPECTED ERROR");
        }
    };

    println!("{}", res.len());
    for x in res.iter() {
        println!("{}", x.name());
    }
}

#[allow(dead_code)]
#[tokio::main]
async fn get_branches() {
    let url = "https://api.github.com/repos/dart-lightning/lndart.cln/branches";
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "reckless-rs")
        .send()
        .await
        .unwrap();

    let mut res = Vec::new();
    match response.status() {
        reqwest::StatusCode::OK => {
            match response.json::<Vec<Branch>>().await {
                Ok(parsed) => res = parsed,
                Err(_) => println!("ERROR WHILE PARSING!"),
            };
        }
        _ => {
            panic!("UNEXPECTED ERROR");
        }
    };

    println!("{}", res.len());
}
