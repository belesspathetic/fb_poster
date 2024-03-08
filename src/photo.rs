#![allow(dead_code)]
use crate::utils::{collect_file, get_file_name, get_response, Secrets};

use anyhow::{Ok, Result};
use reqwest::{multipart, Client};

pub struct Photo {
    pub secrets: Secrets,
    pub path: String,
    pub text: Option<String>,
}

impl Photo {
    pub fn new(secrets: Secrets, path: String) -> Self {
        Self {
            secrets: secrets,
            path: path,
            text: None,
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
        let mut reqbody = multipart::Form::new()
            .text("access_token", self.secrets.access_token.to_owned())
            .part("source", part);

        if let Some(msg) = &self.text {
            println!("PROCESS: adding message");

            reqbody = reqbody.text("message", msg.to_owned());
        }

        let resp = cl.post(url).multipart(reqbody).send().await?;

        get_response(resp).await?;

        Ok(())
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.text = Some(message);
        self
    }
}
