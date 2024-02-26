#![allow(dead_code)]
use crate::utils::{get_response, Secrets};

use anyhow::{Ok, Result};
use reqwest::{multipart, Client};

pub struct Post {
    pub secrets: Secrets,
    pub message: Option<String>,
    pub link: Option<String>,
}

impl Post {
    pub fn new(secrets: Secrets) -> Self {
        Self {
            secrets: secrets,
            message: None,
            link: None,
        }
    }

    pub fn with_message(mut self, text: String) -> Self {
        self.message = Some(text);
        self
    }

    pub fn with_link(mut self, link: String) -> Self {
        self.link = Some(link);
        self
    }

    pub async fn send(&self) -> Result<()> {
        println!("PROCESS: sending reqwest...");
        let url = format!(
            "https://graph.facebook.com/v19.0/{}/feed",
            self.secrets.page_id
        );
        let cl = Client::new();

        let mut reqbody =
            multipart::Form::new().text("access_token", self.secrets.access_token.to_owned());

        if let Some(msg) = &self.message {
            println!("PROCESS: adding message");

            reqbody = reqbody.text("message", msg.to_owned());
        }

        if let Some(link) = &self.link {
            println!("PROCESS: adding link");

            reqbody = reqbody.text("link", link.to_owned());
        }

        let resp = cl.post(url).multipart(reqbody).send().await?;

        get_response(resp).await?;

        Ok(())
    }
}
