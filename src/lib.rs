#![allow(dead_code)]
use anyhow::{Ok, Result};
use reqwest::{header::CONTENT_TYPE, multipart, Client};
use serde::Serialize;

mod utils;
use utils::{get_response, collect_file};


pub struct Secrets {
    access_token: String,
    page_id: String,
}

impl Secrets {
    pub fn new(access_token: &str, page_id: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
            page_id: page_id.to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct Post {
    access_token: String,
    message: Option<String>,
    link: Option<String>,
}

impl Post {
    pub fn new(secrets: &Secrets, your_message: Option<String>, your_link: Option<String>) -> Self {
        Self {
            access_token: secrets.access_token.clone(),
            message: your_message,
            link: your_link,
        }
    }

    pub async fn send(&self, secrets: &Secrets) -> Result<()> {
        println!("PROCESS: sending your reqwest...");
        let url = format!("https://graph.facebook.com/v19.0/{}/feed", secrets.page_id);
        let cl = Client::new();
        let resp = cl
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .json(&self)
            .send()
            .await?;

        get_response(resp).await?;

        Ok(())
    }
}

pub struct Photo {
    access_token: String,
    path: String,
}

impl Photo {
    pub fn new(secrets: &Secrets, path: String) -> Self {
        Self {
            access_token: secrets.access_token.clone(),
            path: path,
        }
    }

    pub async fn send(&self, secrets: Secrets) -> Result<()> {
        println!("PROCESS: opening your file...");
        let buffer = collect_file(&self.path);
        println!("SUCCESS: file set");
        println!("PROCESS: sending your reqwest...");
        let url = format!(
            "https://graph.facebook.com/v19.0/{}/photos",
            secrets.page_id
        );

        let cl = Client::new();

        let part = multipart::Part::bytes(buffer).file_name("test.png");
        let reqbody = multipart::Form::new()
            .text("access_token", secrets.access_token.clone())
            .part("source", part);

        let resp = cl.post(url).multipart(reqbody).send().await?;

        get_response(resp).await?;

        Ok(())
    }
}
