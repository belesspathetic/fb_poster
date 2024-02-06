use std::{fs::File, io::Read};
use reqwest::Response;
use anyhow::{anyhow, Result, Ok};

pub async fn get_response(resp: Response) -> Result<()> {
    if !resp.status().is_success() {
        return Err(anyhow!("ERROR: server response is {}", resp.text().await?));
    } else {
        println!("SUCCESS: {}", resp.text().await?)
    };
    Ok(())
}

pub fn collect_file(path: &String) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut file = File::open(path).expect("cant open file");
    file.read_to_end(&mut buffer).expect("cant read file");

    buffer
}