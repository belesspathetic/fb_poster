#![allow(dead_code)]
use crate::utils::{collect_file, get_file_name, get_response, Secrets};

use anyhow::{Ok, Result};
use reqwest::{multipart, Client};

pub struct Photo {
    pub secrets: Secrets,
    pub path: String,
}

impl Photo {
    pub fn new(secrets: Secrets, path: String) -> Self {
        Self {
            secrets: secrets,
            path: path,
        }
    }

    pub async fn send(&self) -> Result<()> {
        println!("PROCESS: opening photo...");
        let buffer = collect_file(&self.path);
        println!("SUCCESS: file set");
        println!("PROCESS: sending reqwest...");
        let url = format!(
            "https://graph.facebook.com/v19.0/{}/photos",
            self.secrets.page_id
        );

        let cl = Client::new();
        let name = get_file_name(&self.path);
        let part = multipart::Part::bytes(buffer).file_name(name);
        let reqbody = multipart::Form::new()
            .text("access_token", self.secrets.access_token.to_owned())
            .part("source", part);

        let resp = cl.post(url).multipart(reqbody).send().await?;

        get_response(resp).await?;

        Ok(())
    }
}
