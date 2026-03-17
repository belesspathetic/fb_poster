use crate::utils::{collect_file, get_file_name, get_response, Secrets};

use anyhow::{anyhow, Context, Result};
use log::info;
use reqwest::{multipart, Client};
use serde_json::Value;

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
            secrets,
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

    pub async fn send(&self) -> Result<String> {
        info!("STEP 1: Validate input");

        if self.path.is_none() && self.url.is_none() {
            return Err(anyhow!("No video source provided"));
        }

        if self.path.is_some() && self.url.is_some() {
            return Err(anyhow!("Use either local file OR URL, not both"));
        }

        let endpoint = format!(
            "https://graph-video.facebook.com/v25.0/{}/videos",
            self.secrets.page_id
        );

        let cl = Client::new();

        let mut form =
            multipart::Form::new().text("access_token", self.secrets.access_token.clone());

        // 🎬 VIDEO
        if let Some(path) = &self.path {
            info!("Uploading LOCAL video: {}", path);

            let buffer = collect_file(path)
                .with_context(|| format!("failed to read file {}", path))?;

            let name = get_file_name(path);

            let part = multipart::Part::bytes(buffer)
                .file_name(name)
                .mime_str("video/mp4")?;

            form = form.part("source", part);
        }

        if let Some(url) = &self.url {
            info!("Uploading VIDEO by URL: {}", url);

            form = form.text("file_url", url.clone());
        }

        // 📝 META
        if let Some(title) = &self.title {
            form = form.text("title", title.clone());
        }

        if let Some(description) = &self.description {
            form = form.text("description", description.clone());
        }

        info!("STEP 2: Send request to Facebook");

        let resp = cl.post(&endpoint).multipart(form).send().await?;

        let json: Value = get_response(resp).await?;

        let video_id = json["id"]
            .as_str()
            .ok_or_else(|| anyhow!("missing video id in response"))?
            .to_string();

        info!("Video uploaded: id={}", video_id);

        // 🖼 THUMBNAIL (after upload!)
        if let Some(path) = &self.thumb {
            self.upload_thumbnail(&cl, &video_id, path).await?;
        }

        Ok(video_id)
    }

    async fn upload_thumbnail(
        &self,
        cl: &Client,
        video_id: &str,
        path: &str,
    ) -> Result<()> {
        info!("STEP 3: Upload thumbnail");

        let endpoint = format!(
            "https://graph.facebook.com/v25.0/{}/thumbnails",
            video_id
        );

        let form = multipart::Form::new()
            .text("access_token", self.secrets.access_token.clone())
            .text("is_preferred", "true")
            .part(
                "source",
                multipart::Part::bytes(collect_file(&path.to_string())?)
                    .file_name("thumb.jpg"),
            );

        let resp = cl.post(endpoint).multipart(form).send().await?;

        get_response(resp).await?;

        info!("Thumbnail uploaded");

        Ok(())
    }
}