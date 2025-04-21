#![allow(dead_code)]
use crate::utils::{collect_file, get_file_name, Secrets};

use anyhow::{Ok, Result};
use log::debug;
use reqwest::{multipart, Client};

pub struct Photo {
    pub secrets: Secrets,
    pub path: String,
    pub text: Option<String>,
    pub published: Option<bool>,
    pub media_id: Option<String>,
}

impl Photo {
    pub fn new(secrets: Secrets, path: String) -> Self {
        Self {
            secrets: secrets,
            path: path,
            text: None,
            published: None,
            media_id: None,
        }
    }

    pub async fn send(mut self) -> Result<Self> {
        debug!("PROCESS: opening photo...");
        let buffer = collect_file(&self.path);
        debug!("SUCCESS: file set");
        debug!("PROCESS: sending reqwest...");
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
            debug!("PROCESS: adding message");

            reqbody = reqbody.text("message", msg.to_owned());
        }

        let publish_flag = self.published.unwrap_or(true);
        reqbody = reqbody.text("published", publish_flag.to_string());

        let resp = cl.post(url).multipart(reqbody).send().await?;

        let json: serde_json::Value = resp.json().await?;
        debug!("PHOTO RESPONSE: {:?}", json);

        if let Some(id) = json.get("id") {
            self.media_id = Some(id.as_str().unwrap_or_default().to_string());
            Ok(self)
        } else {
            Err(anyhow::anyhow!("No photo ID"))
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.text = Some(message);
        self
    }

    pub fn published(mut self, value: bool) -> Self {
        self.published = Some(value);
        self
    }
}
