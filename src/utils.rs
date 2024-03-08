#![allow(dead_code)]
use anyhow::{anyhow, Ok, Result};
use reqwest::Response;
use std::{
    fs::{metadata, File},
    io::Read,
    path::Path,
};
use log::debug;

pub async fn get_response(resp: Response) -> Result<()> {
    if !resp.status().is_success() {
        return Err(anyhow!("ERROR: server response is {}", resp.text().await?));
    } else {
        debug!("SUCCESS: {}", resp.text().await?)
    };
    Ok(())
}

pub fn collect_file(path: &String) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut file = File::open(path).expect("cant open file");
    file.read_to_end(&mut buffer).expect("cant read file");

    buffer
}

pub fn get_file_name(path: &String) -> String {
    let name = Path::new(&path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    name
}

pub fn get_file_size(path: &String) -> Result<u64> {
    let metadata = metadata(path)?;
    let file_size = metadata.len();
    Ok(file_size)
}

pub struct Secrets {
    pub access_token: String,
    pub page_id: String,
}

impl Secrets {
    pub fn new(access_token: &str, page_id: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
            page_id: page_id.to_string(),
        }
    }
}
