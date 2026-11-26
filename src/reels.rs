use crate::utils::{collect_file, get_file_size, get_response, Secrets};

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use reqwest::{header::AUTHORIZATION, multipart, Client};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

const GRAPH_API_VERSION: &str = "v25.0";

pub struct Reels {
    pub secrets: Secrets,
    pub path: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub post_to_instagram: bool,
    pub ig_user_id: Option<String>,
}

struct Session {
    video_id: String,
    upload_url: String,
    cl: Client,
}

pub struct PublishResult {
    pub facebook_id: String,
    pub instagram_id: Option<String>,
}

impl Reels {
    pub fn new(secrets: Secrets) -> Self {
        let ig_user_id = secrets.ig_user_id.clone();
        Self {
            secrets,
            path: None,
            url: None,
            description: None,
            thumbnail: None,
            post_to_instagram: false,
            ig_user_id,
        }
    }

    pub fn local_video(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn hosted_video(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_thumbnail(mut self, path: impl Into<String>) -> Self {
        self.thumbnail = Some(path.into());
        self
    }

    pub fn instagram(mut self, ig_user_id: impl Into<String>) -> Self {
        let id = ig_user_id.into();
        if !id.is_empty() {
            self.ig_user_id = Some(id);
        }
        self.post_to_instagram = true;
        self
    }

    fn resolve_ig_user_id(&self) -> Result<&str> {
        self.ig_user_id
            .as_deref()
            .or(self.secrets.ig_user_id.as_deref())
            .ok_or_else(|| anyhow!("missing ig_user_id for Instagram publishing"))
    }

    async fn upload_media_source(&self, cl: &Client, endpoint: &str) -> Result<()> {
        if self.path.is_none() && self.url.is_none() {
            bail!("neither local video path nor hosted video url is set");
        }

        if let Some(path) = &self.path {
            debug!("Uploading local file: {} -> {}", path, endpoint);

            let buffer = collect_file(path)
                .with_context(|| format!("failed to read file {}", path))?;
            let size = get_file_size(path)?;

            let resp = cl
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
            debug!("Uploading from URL: {} -> {}", url, endpoint);

            let resp = cl
                .post(endpoint)
                .header(AUTHORIZATION, format!("OAuth {}", self.secrets.access_token))
                .header("file_url", url)
                .send()
                .await?;

            get_response(resp).await?;
        }

        Ok(())
    }

    // --- Facebook Reels ---

    async fn fb_initial_session(&self, cl: &Client) -> Result<Session> {
        info!("FB STEP 1: Start upload session");

        let endpoint = format!(
            "https://graph.facebook.com/{}/{}/video_reels",
            GRAPH_API_VERSION, self.secrets.page_id
        );

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

        info!("FB session created: video_id={}", video_id);

        Ok(Session {
            video_id,
            upload_url,
            cl: cl.clone(),
        })
    }

    async fn fb_upload_thumbnail(&self, session: &Session) -> Result<()> {
        let Some(path) = &self.thumbnail else {
            return Ok(());
        };

        info!("FB STEP 3: Upload thumbnail");

        let endpoint = format!(
            "https://graph.facebook.com/{}/{}/thumbnails",
            GRAPH_API_VERSION, session.video_id
        );

        let form = multipart::Form::new()
            .text("access_token", self.secrets.access_token.clone())
            .text("is_preferred", "true")
            .part(
                "source",
                multipart::Part::bytes(collect_file(path)?).file_name("thumbnail.jpg"),
            );

        let resp = session.cl.post(endpoint).multipart(form).send().await?;
        get_response(resp).await?;

        Ok(())
    }

    async fn fb_publish(&self, session: &Session) -> Result<String> {
        info!("FB STEP 4: Publish video");

        let endpoint = format!(
            "https://graph.facebook.com/{}/{}/video_reels",
            GRAPH_API_VERSION, self.secrets.page_id
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
        let publish_json: Value = get_response(resp).await?;

        let final_id = publish_json["video_id"]
            .as_str()
            .or_else(|| publish_json["id"].as_str())
            .unwrap_or(&session.video_id)
            .to_string();

        Ok(final_id)
    }

    async fn publish_facebook(&self, cl: &Client) -> Result<String> {
        let session = self.fb_initial_session(cl).await?;
        self.upload_media_source(&session.cl, &session.upload_url).await?;
        self.fb_upload_thumbnail(&session).await?;
        let id = self.fb_publish(&session).await?;
        info!("FB SUCCESS: published video_id={}", id);
        Ok(id)
    }

    // --- Instagram Reels ---

    async fn ig_create_container(&self, cl: &Client, ig_user_id: &str) -> Result<String> {
        info!("IG STEP 1: Create media container");

        let endpoint = format!(
            "https://graph.facebook.com/{}/{}/media",
            GRAPH_API_VERSION, ig_user_id
        );

        let mut payload = json!({
            "media_type": "REELS",
            "access_token": self.secrets.access_token
        });

        if let Some(desc) = &self.description {
            payload["caption"] = json!(desc);
        }

        if self.path.is_some() {
            // Для локального файла используем resumable upload через rupload.facebook.com
            payload["upload_type"] = json!("resumable");
        } else if let Some(url) = &self.url {
            payload["video_url"] = json!(url);
        }

        let resp = cl.post(&endpoint).json(&payload).send().await?;
        let val = get_response(resp).await?;

        let container_id = val["id"]
            .as_str()
            .ok_or_else(|| anyhow!("missing Instagram container id"))?
            .to_string();

        info!("IG container created: id={}", container_id);
        Ok(container_id)
    }

    async fn ig_wait_until_ready(&self, cl: &Client, container_id: &str) -> Result<()> {
        info!("IG STEP 3: Waiting for container processing");

        let endpoint = format!(
            "https://graph.facebook.com/{}/{}",
            GRAPH_API_VERSION, container_id
        );

        for attempt in 1..=40 {
            let resp = cl
                .get(&endpoint)
                .query(&[
                    ("fields", "status_code"),
                    ("access_token", self.secrets.access_token.as_str()),
                ])
                .send()
                .await?;

            let val = get_response(resp).await?;
            let status = val["status_code"].as_str().unwrap_or("");

            debug!("IG container {} status (attempt {}): {}", container_id, attempt, status);

            match status {
                "FINISHED" => return Ok(()),
                "ERROR" | "EXPIRED" => {
                    bail!("Instagram container {} failed with status: {}", container_id, status);
                }
                _ => sleep(Duration::from_secs(3)).await,
            }
        }

        bail!("Instagram container {} timed out while processing", container_id)
    }

    async fn ig_publish_container(&self, cl: &Client, ig_user_id: &str, container_id: &str) -> Result<String> {
        info!("IG STEP 4: Publish media container");

        let endpoint = format!(
            "https://graph.facebook.com/{}/{}/media_publish",
            GRAPH_API_VERSION, ig_user_id
        );

        let payload = json!({
            "creation_id": container_id,
            "access_token": self.secrets.access_token
        });

        let resp = cl.post(&endpoint).json(&payload).send().await?;
        let val = get_response(resp).await?;

        let media_id = val["id"]
            .as_str()
            .ok_or_else(|| anyhow!("missing Instagram media id"))?
            .to_string();

        info!("IG SUCCESS: published media_id={}", media_id);
        Ok(media_id)
    }

    async fn publish_instagram(&self, cl: &Client) -> Result<String> {
        let ig_user_id = self.resolve_ig_user_id()?;
        let container_id = self.ig_create_container(cl, ig_user_id).await?;

        if self.path.is_some() {
            let upload_endpoint = format!(
                "https://rupload.facebook.com/ig-api-upload/{}/{}",
                GRAPH_API_VERSION, container_id
            );
            self.upload_media_source(cl, &upload_endpoint).await?;
        }

        self.ig_wait_until_ready(cl, &container_id).await?;
        self.ig_publish_container(cl, ig_user_id, &container_id).await
    }

    pub async fn send(self) -> Result<PublishResult> {
        let cl = Client::new();

        let facebook_id = self.publish_facebook(&cl).await?;

        let instagram_id = if self.post_to_instagram {
            Some(self.publish_instagram(&cl).await?)
        } else {
            None
        };

        Ok(PublishResult {
            facebook_id,
            instagram_id,
        })
    }
}