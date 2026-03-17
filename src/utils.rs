use anyhow::{anyhow, Result};
use log::{debug, error};
use reqwest::Response;
use serde_json::Value;
use std::{
    fs::{metadata, File},
    io::Read,
    path::Path,
};

pub async fn get_response(resp: Response) -> Result<Value> {
    let status = resp.status();
    let text = resp.text().await?;

    debug!("RESPONSE: {}", text);

    if !status.is_success() {
        error!("HTTP ERROR {}: {}", status, text);
        return Err(anyhow!("server error: {}", text));
    }

    let json: Value = serde_json::from_str(&text)
        .map_err(|_| anyhow!("invalid json response: {}", text))?;

    Ok(json)
}

pub fn collect_file(path: &String) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn get_file_size(path: &String) -> Result<u64> {
    Ok(metadata(path)?.len())
}

#[derive(Clone)]
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
pub fn get_file_name(path: &String) -> String {
    let name = Path::new(&path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    name
}