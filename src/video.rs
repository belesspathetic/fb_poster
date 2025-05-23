#![allow(dead_code)]
use crate::utils::{collect_file, get_file_name, get_response, Secrets};

use anyhow::{Ok, Result};
use log::debug;
use reqwest::{multipart, Client};

pub struct Video {
    pub secrets: Secrets,
    pub path: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub thumb: Option<String>,
}

impl Video {
    pub fn new(secrets: Secrets) -> Self {
        Self {
            secrets: secrets,
            path: None,
            url: None,
            title: None,
            description: None,
            thumb: None,
        }
    }

    pub fn local_video(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn hosted_video(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_thumbnail(mut self, path: String) -> Self {
        self.thumb = Some(path);
        self
    }

    pub async fn send(&self) -> Result<()> {
        let url = format!(
            "https://graph-video.facebook.com/v19.0/{}/videos",
            self.secrets.page_id
        );
        let cl = Client::new();
        let mut reqbody =
            multipart::Form::new().text("access_token", self.secrets.access_token.to_owned());

        if let Some(path) = &self.path {
            debug!("PROCESS: adding local video...");
            let buffer = collect_file(path);
            let name = get_file_name(path);
            let part = multipart::Part::bytes(buffer).file_name(name);

            reqbody = reqbody.part("source", part);
        }

        if let Some(url) = &self.url {
            debug!("PROCESS: adding hosted video...");

            reqbody = reqbody.text("file_url", url.to_owned());
        }

        if let Some(title) = &self.title {
            debug!("PROCESS: adding title...");

            reqbody = reqbody.text("title", title.to_owned());
        }

        if let Some(description) = &self.description {
            debug!("PROCESS: adding description...");

            reqbody = reqbody.text("description", description.to_owned());
        }

        if let Some(thumb) = &self.thumb {
            debug!("PROCESS: adding thumbnail...");
            let buffer = collect_file(thumb);
            let name = get_file_name(thumb);
            let part = multipart::Part::bytes(buffer).file_name(name);

            reqbody = reqbody.part("thumb", part);
        }

        debug!("PROCESS: sending reqwest...");

        let resp = cl.post(url).multipart(reqbody).send().await?;
        get_response(resp).await?;

        Ok(())
    }
}
