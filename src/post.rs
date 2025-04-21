#![allow(dead_code)]
use std::fs;

use crate::
    utils::{collect_file, get_file_name, get_response, Secrets};

use anyhow::{Ok, Result};
use log::debug;
use reqwest::{multipart, Client};

pub struct Post {
    pub secrets: Secrets,
    pub message: Option<String>,
    pub link: Option<String>,
    pub url: Option<String>,
}

impl Post {
    pub fn new(secrets: Secrets) -> Self {
        Self {
            secrets: secrets,
            message: None,
            link: None,
            url: None,
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

    pub fn with_photo(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub async fn send(&self) -> Result<()> {
        debug!("PROCESS: sending reqwest...");
        let endpoint = format!(
            "https://graph.facebook.com/v19.0/{}/feed",
            self.secrets.page_id
        );
        let cl = Client::new();

        let mut reqbody =
            multipart::Form::new().text("access_token", self.secrets.access_token.to_owned());

        if let Some(msg) = &self.message {
            debug!("PROCESS: adding message");

            reqbody = reqbody.text("message", msg.to_owned());
        }

        if let Some(link) = &self.link {
            debug!("PROCESS: adding link");

            reqbody = reqbody.text("link", link.to_owned());
        }


        if let Some(url) = &self.url {
            reqbody = reqbody.text("source", url.clone());
        }

        let resp = cl.post(endpoint).multipart(reqbody).send().await?;

        debug!("{:?}", resp);

        get_response(resp).await?;

        Ok(())
    }
}
