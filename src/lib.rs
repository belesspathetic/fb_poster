#![allow(dead_code)]
use anyhow::{anyhow, Ok, Result};
use reqwest::{header::CONTENT_TYPE, Client, Response};
use serde::{Serialize};
mod tests;

struct Secrets {
    access_token: String,
    page_id: String,
}

impl Secrets {
    fn new(access_token: &str, page_id: &str) -> Self {
        Self {
            access_token: access_token.to_string(),
            page_id: page_id.to_string(),
        }
    }
}

#[derive(Serialize)]
struct Post {
    access_token: String,
    message: Option<String>,
    link: Option<String>,
}

impl Post {
    fn new(secrets: &Secrets, your_message: Option<String>, your_link: Option<String>) -> Self {
        Self {
            access_token: secrets.access_token.clone(),
            message: your_message,
            link: your_link,
        }
    }

    async fn send(&self, secrets: &Secrets) -> Result<()> {
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

async fn get_response(resp: Response) -> Result<()> {
    if !resp.status().is_success() {
        return Err(anyhow!("ERROR: server response is {}", resp.text().await?));
    } else {
        println!("SUCCESS: {}", resp.text().await?)
    };
    Ok(())
}
