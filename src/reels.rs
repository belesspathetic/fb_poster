use crate::utils::{collect_file, get_file_size, get_response, Secrets};

use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use reqwest::{header::AUTHORIZATION, multipart, Client};
use serde_json::json;

pub struct Reels {
    pub secrets: Secrets,
    pub path: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
}

struct Session {
    video_id: String,
    upload_url: String,
    cl: Client,
}

impl Reels {
    pub fn new(secrets: Secrets) -> Self {
        Self {
            secrets,
            path: None,
            url: None,
            description: None,
            thumbnail: None,
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

    pub fn with_thumbnail(mut self, path: String) -> Self {
        self.thumbnail = Some(path);
        self
    }

    async fn initial_session(&self) -> Result<Session> {
        info!("STEP 1: Start upload session");

        let endpoint = format!(
            "https://graph.facebook.com/v25.0/{}/video_reels",
            self.secrets.page_id
        );

        let cl = Client::new();

        let data = json!({
            "upload_phase": "start",
            "access_token": self.secrets.access_token
        });

        let resp = cl.post(&endpoint).json(&data).send().await?;

        let json = get_response(resp).await?;

        let video_id = json["video_id"]
            .as_str()
            .ok_or_else(|| anyhow!("missing video_id"))?
            .to_string();

        let upload_url = json["upload_url"]
            .as_str()
            .ok_or_else(|| anyhow!("missing upload_url"))?
            .to_string();

        info!("Session created: video_id={}", video_id);

        Ok(Session {
            video_id,
            upload_url,
            cl,
        })
    }

    async fn reel_upload(&self, session: &Session) -> Result<()> {
        info!("STEP 2: Upload video");

        let endpoint = &session.upload_url;

        if let Some(path) = &self.path {
            debug!("Uploading local file: {}", path);

            let buffer = collect_file(path)
                .with_context(|| format!("failed to read file {}", path))?;

            let size = get_file_size(path)?;

            let resp = session
                .cl
                .post(endpoint)
                .header(AUTHORIZATION, format!("OAuth {}", self.secrets.access_token))
                .header("offset", "0")
                .header("file_size", size)
                .body(buffer)
                .send()
                .await?;

            get_response(resp).await?;
        }

        if let Some(url) = &self.url {
            debug!("Uploading from URL: {}", url);

            let resp = session
                .cl
                .post(endpoint)
                .header(AUTHORIZATION, format!("OAuth {}", self.secrets.access_token))
                .header("file_url", url)
                .send()
                .await?;

            get_response(resp).await?;
        }

        Ok(())
    }

    async fn upload_thumbnail(&self, session: &Session) -> Result<()> {
        if self.thumbnail.is_none() {
            return Ok(());
        }

        info!("STEP 3: Upload thumbnail");

        let path = self.thumbnail.as_ref().unwrap();

        let endpoint = format!(
            "https://graph.facebook.com/v25.0/{}/thumbnails",
            session.video_id
        );

        let form = multipart::Form::new()
            .text("access_token", self.secrets.access_token.clone())
            .text("is_preferred", "true")
            .part(
                "source",
                multipart::Part::bytes(collect_file(path)?)
                    .file_name("thumbnail.jpg"),
            );

        let resp = session.cl.post(endpoint).multipart(form).send().await?;

        get_response(resp).await?;

        Ok(())
    }

    async fn publish(&self, session: &Session) -> Result<()> {
        info!("STEP 4: Publish video");

        let endpoint = format!(
            "https://graph.facebook.com/v25.0/{}/video_reels",
            self.secrets.page_id
        );

        let mut reqbody = multipart::Form::new()
            .text("access_token", self.secrets.access_token.clone())
            .text("video_id", session.video_id.clone())
            .text("upload_phase", "finish")
            .text("video_state", "PUBLISHED");

        if let Some(desc) = &self.description {
            reqbody = reqbody.text("description", desc.clone());
        }

        let resp = session.cl.post(endpoint).multipart(reqbody).send().await?;

        get_response(resp).await?;

        Ok(())
    }

    pub async fn send(self) -> Result<()> {
        let session = self.initial_session().await?;

        self.reel_upload(&session).await?;

        self.upload_thumbnail(&session).await?;

        self.publish(&session).await?;

        info!("SUCCESS: Video fully published");

        Ok(())
    }
}