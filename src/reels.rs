#![allow(dead_code)]
use crate::utils::{collect_file, get_file_size, get_response, Secrets};

use anyhow::{Ok, Result};
use log::debug;
use reqwest::{header::AUTHORIZATION, multipart, Client};
use serde_json::{json, Value};

pub struct Reels {
    pub secrets: Secrets,
    pub path: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
}

struct Session {
    video_id: String,
    upload_url: String,
    cl: Client,
}

impl Reels {
    pub fn new(secrets: Secrets) -> Self {
        Self {
            secrets: secrets,
            path: None,
            url: None,
            description: None,
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

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    async fn initial_session(&self) -> Result<Session> {
        let endpoint = format!(
            "https://graph.facebook.com/v19.0/{}/video_reels",
            self.secrets.page_id
        );
        let cl = Client::new();

        let data = json!({
            "upload_phase": "start",
            "access_token": self.secrets.access_token
        });

        let resp = cl
            .post(endpoint)
            .header("Content-Type", "application/json")
            .body(data.to_string())
            .send()
            .await?;

        let resp_string = resp.text().await?;

        debug!("{}", resp_string);

        let (video_id, upload_url) = Reels::parse_from_json(resp_string.to_string());

        let session = Session {
            video_id: video_id,
            upload_url: upload_url,
            cl: cl,
        };

        Ok(session)
    }

    async fn reel_upload(&self, session: &Session) -> Result<()> {
        let endpoint = format!(
            "https://rupload.facebook.com/video-upload/v19.0/{}",
            session.video_id
        );

        if let Some(path) = &self.path {
            debug!("PROCESS: uploading...");
            let buffer = collect_file(path);
            let size = get_file_size(path).expect("cant open a file");

            let resp = session
                .cl
                .post(&endpoint)
                .header(
                    AUTHORIZATION,
                    format!("OAuth {}", self.secrets.access_token),
                )
                .header("offset", "0".to_string())
                .header("file_size", size.to_string())
                .body(buffer)
                .send()
                .await?;
            get_response(resp).await?;
        }

        if let Some(url) = &self.url {
            debug!("PROCESS: uploading...");
            let resp = session
                .cl
                .post(&endpoint)
                .header(
                    AUTHORIZATION,
                    format!("OAuth {}", self.secrets.access_token),
                )
                .header("file_url", url)
                .send()
                .await?;
            get_response(resp).await?;
        }
        Ok(())
    }

    async fn publish(&self, session: &Session) -> Result<()> {
        let endpoint = format!(
            "https://graph.facebook.com/v19.0/{}/video_reels",
            self.secrets.page_id
        );

        let mut reqbody = multipart::Form::new()
            .text("access_token", self.secrets.access_token.to_owned())
            .text("video_id", session.video_id.to_owned())
            .text("upload_phase", "finish")
            .text("video_state", "PUBLISHED");

        if let Some(desc) = &self.description {
            reqbody = reqbody.text("description", desc.to_owned());
        }

        let resp = session.cl.post(endpoint).multipart(reqbody).send().await?;
        get_response(resp).await?;
        Ok(())
    }

    pub async fn send(self) -> Result<()> {
        let session = Self::initial_session(&self).await?;

        Self::reel_upload(&self, &session).await?;

        Self::publish(&self, &session).await?;
        debug!("SUCCESS: Video published");
        Ok(())
    }

    fn parse_from_json(resp: String) -> (String, String) {
        let parsed: Value = serde_json::from_str(&resp).expect("cant parse json");

        let video_id = parsed["video_id"].as_str().unwrap().to_string();
        let upload_url = parsed["upload_url"].as_str().unwrap().to_string();

        (video_id, upload_url)
    }
}
