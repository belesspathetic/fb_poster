use crate::utils::{collect_file, get_file_size, get_response, Secrets};

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info};
use reqwest::{
    header::{HeaderMap, AUTHORIZATION},
    multipart, Client, Response, StatusCode, Url,
};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

const GRAPH_BASE: &str = "https://graph.facebook.com";
const RUPLOAD_BASE: &str = "https://rupload.facebook.com";

const FB_GRAPH_API_VERSION: &str = "v25.0";
const IG_GRAPH_API_VERSION: &str = "v27.0";

const IG_POLL_INTERVAL: Duration = Duration::from_secs(10);
const IG_MAX_ATTEMPTS: u32 = 240; // ~40 минут на скачивание и обработку видео со стороны Meta

#[derive(Default, Clone)]
struct MediaSource {
    path: Option<String>,
    url: Option<String>,
    description: Option<String>,
    thumbnail: Option<String>,
}

impl MediaSource {
    async fn upload_to(&self, cl: &Client, access_token: &str, endpoint: &str) -> Result<()> {
        if self.path.is_none() && self.url.is_none() {
            bail!("neither local video path nor hosted video url is set");
        }

        if let Some(path) = &self.path {
            debug!("Uploading local file: {} -> {}", path, endpoint);
            println!("[UPLOAD] reading local file path={path}");

            let buffer =
                collect_file(path).with_context(|| format!("failed to read file {}", path))?;
            let size = get_file_size(path)?;
            println!("[UPLOAD] sending {size} bytes to {endpoint}");

            let resp = cl
                .post(endpoint)
                .header(AUTHORIZATION, format!("OAuth {}", access_token))
                .header("offset", "0")
                .header("file_size", size)
                .body(buffer)
                .send()
                .await?;

            get_response(resp).await?;
            println!("[UPLOAD] local file upload finished");
        }

        if let Some(url) = &self.url {
            debug!("Uploading from URL: {} -> {}", url, endpoint);
            println!("[UPLOAD] sending hosted video_url={url} to {endpoint}");

            let resp = cl
                .post(endpoint)
                .header(AUTHORIZATION, format!("OAuth {}", access_token))
                .header("file_url", url)
                .send()
                .await?;

            get_response(resp).await?;
            println!("[UPLOAD] hosted video request accepted");
        }

        Ok(())
    }
}

pub mod facebook {
    use super::*;

    pub struct Reel {
        secrets: Secrets,
        media: MediaSource,
    }

    struct Session {
        video_id: String,
        upload_url: String,
    }

    impl Reel {
        pub fn new(secrets: Secrets) -> Self {
            Self {
                secrets,
                media: MediaSource::default(),
            }
        }

        pub fn local_video(mut self, path: impl Into<String>) -> Self {
            self.media.path = Some(path.into());
            self
        }

        pub fn hosted_video(mut self, url: impl Into<String>) -> Self {
            self.media.url = Some(url.into());
            self
        }

        pub fn with_description(mut self, description: impl Into<String>) -> Self {
            self.media.description = Some(description.into());
            self
        }

        pub fn with_thumbnail(mut self, path: impl Into<String>) -> Self {
            let p = path.into();
            if !p.is_empty() {
                self.media.thumbnail = Some(p);
            }
            self
        }

        async fn start_session(&self, cl: &Client) -> Result<Session> {
            info!("FB STEP 1: Start upload session");
            println!("[FB] step 1/4: starting upload session for page_id={}", self.secrets.page_id);

            let endpoint = format!(
                "{}/{}/{}/video_reels",
                GRAPH_BASE, FB_GRAPH_API_VERSION, self.secrets.page_id
            );

            let data = json!({
                "upload_phase": "start",
                "access_token": self.secrets.access_token
            });

            let resp = cl.post(&endpoint).json(&data).send().await?;
            let val = get_response(resp).await?;

            let video_id = val["video_id"]
                .as_str()
                .ok_or_else(|| anyhow!("missing video_id"))?
                .to_string();

            let upload_url = val["upload_url"]
                .as_str()
                .ok_or_else(|| anyhow!("missing upload_url"))?
                .to_string();

            info!("FB session created: video_id={}", video_id);
            println!("[FB] session created: video_id={video_id}, upload_url={upload_url}");
            Ok(Session {
                video_id,
                upload_url,
            })
        }

        async fn upload_thumbnail(&self, cl: &Client, video_id: &str) -> Result<()> {
            let Some(path) = &self.media.thumbnail else {
                println!("[FB] step 3/4: no thumbnail set, skipping");
                return Ok(());
            };

            info!("FB STEP 3: Upload thumbnail");
            println!("[FB] step 3/4: uploading thumbnail from {path} for video_id={video_id}");

            let endpoint = format!(
                "{}/{}/{}/thumbnails",
                GRAPH_BASE, FB_GRAPH_API_VERSION, video_id
            );

            let form = multipart::Form::new()
                .text("access_token", self.secrets.access_token.clone())
                .text("is_preferred", "true")
                .part(
                    "source",
                    multipart::Part::bytes(collect_file(path)?).file_name("thumbnail.jpg"),
                );

            let resp = cl.post(endpoint).multipart(form).send().await?;
            get_response(resp).await?;
            println!("[FB] thumbnail uploaded for video_id={video_id}");
            Ok(())
        }

        async fn publish_session(&self, cl: &Client, video_id: &str) -> Result<String> {
            info!("FB STEP 4: Publish video");
            println!("[FB] step 4/4: publishing video_id={video_id}");

            let endpoint = format!(
                "{}/{}/{}/video_reels",
                GRAPH_BASE, FB_GRAPH_API_VERSION, self.secrets.page_id
            );

            let mut form = multipart::Form::new()
                .text("access_token", self.secrets.access_token.clone())
                .text("video_id", video_id.to_string())
                .text("upload_phase", "finish")
                .text("video_state", "PUBLISHED");

            if let Some(desc) = &self.media.description {
                form = form.text("description", desc.clone());
            }

            let resp = cl.post(endpoint).multipart(form).send().await?;
            let publish_json: Value = get_response(resp).await?;

            let final_id = publish_json["video_id"]
                .as_str()
                .or_else(|| publish_json["id"].as_str())
                .unwrap_or(video_id)
                .to_string();

            info!("FB SUCCESS: published video_id={}", final_id);
            println!("[FB] published successfully: final_id={final_id}");
            Ok(final_id)
        }

        pub async fn send(self) -> Result<String> {
            println!("[FB] starting Reel workflow");
            let cl = Client::new();
            let session = self.start_session(&cl).await?;
            println!("[FB] step 2/4: uploading video to session upload_url");
            self.media
                .upload_to(&cl, &self.secrets.access_token, &session.upload_url)
                .await?;
            self.upload_thumbnail(&cl, &session.video_id).await?;
            self.publish_session(&cl, &session.video_id).await
        }
    }
}

pub mod instagram {
    use super::*;

    pub struct Reel {
        secrets: Secrets,
        ig_user_id: String,
        media: MediaSource,
    }

    impl Reel {
        pub fn new(secrets: Secrets, ig_user_id: impl Into<String>) -> Result<Self> {
            let id = ig_user_id.into();
            let resolved_id = if id.trim().is_empty() {
                secrets
                    .ig_user_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| anyhow!("ig_user_id must not be empty"))?
                    .to_string()
            } else {
                id.trim().to_string()
            };

            Ok(Self {
                secrets,
                ig_user_id: resolved_id,
                media: MediaSource::default(),
            })
        }

        pub fn local_video(mut self, path: impl Into<String>) -> Self {
            self.media.path = Some(path.into());
            self.media.url = None;
            self
        }

        pub fn hosted_video(mut self, url: impl Into<String>) -> Self {
            self.media.url = Some(url.into());
            self.media.path = None;
            self
        }

        pub fn with_description(mut self, description: impl Into<String>) -> Self {
            self.media.description = Some(description.into());
            self
        }

        async fn create_container(&self, cl: &Client) -> Result<(String, Option<String>)> {
            info!("IG STEP 1: Create media container");
            println!(
                "[IG] step 1/3: creating REELS container for ig_user_id={}",
                self.ig_user_id
            );

            if self.media.path.is_none() && self.media.url.is_none() {
                bail!("neither local video path nor hosted video url is set");
            }

            let endpoint = format!(
                "{}/{}/{}/media",
                GRAPH_BASE, IG_GRAPH_API_VERSION, self.ig_user_id
            );

            let mut params: Vec<(&str, &str)> = vec![
                ("media_type", "REELS"),
                ("access_token", self.secrets.access_token.as_str()),
            ];

            if let Some(desc) = self
                .media
                .description
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                params.push(("caption", desc));
            }

            if self.media.path.is_some() {
                params.push(("upload_type", "resumable"));
                println!("[IG] container mode: resumable local upload");
            } else if let Some(url) = self.media.url.as_deref() {
                params.push(("video_url", url));
                println!("[IG] container mode: hosted video_url={url}");
            }

            let resp = cl.post(&endpoint).query(&params).send().await?;
            let val = parse_ig_json(resp).await?;

            let container_id = val["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| anyhow!("missing Instagram container id in response: {val}"))?
                .to_string();

            let upload_uri = val["uri"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(str::to_string);

            info!("IG container created: id={}", container_id);
            println!(
                "[IG] container created: container_id={container_id}, upload_uri={upload_uri:?}"
            );
            Ok((container_id, upload_uri))
        }

        async fn wait_until_ready(&self, cl: &Client, container_id: &str) -> Result<()> {
            info!("IG STEP 3: Waiting for container processing");
            println!(
                "[IG] step 2/3: polling container_id={container_id} (up to {IG_MAX_ATTEMPTS} attempts, every {}s)",
                IG_POLL_INTERVAL.as_secs()
            );

            let endpoint = format!(
                "{}/{}/{}",
                GRAPH_BASE, IG_GRAPH_API_VERSION, container_id
            );

            for attempt in 1..=IG_MAX_ATTEMPTS {
                let resp = cl
                    .get(&endpoint)
                    .query(&[
                        ("fields", "status_code,status"),
                        ("access_token", self.secrets.access_token.as_str()),
                    ])
                    .send()
                    .await?;

                let val = parse_ig_json(resp).await?;
                let status = val["status_code"].as_str().unwrap_or("UNKNOWN");

                debug!(
                    "IG container {} status (attempt {}/{}): {} | {}",
                    container_id, attempt, IG_MAX_ATTEMPTS, status, val["status"]
                );
                println!(
                    "[IG] poll {attempt}/{IG_MAX_ATTEMPTS}: container_id={container_id}, status_code={status}, status={}",
                    val["status"]
                );

                match status {
                    "FINISHED" => {
                        println!("[IG] container_id={container_id} is FINISHED and ready to publish");
                        return Ok(());
                    }
                    "ERROR" | "EXPIRED" => {
                        println!("[IG] container_id={container_id} failed with status={status}: {val}");
                        bail!(
                            "Instagram container {} failed with status {}: {}",
                            container_id,
                            status,
                            val
                        );
                    }
                    _ => sleep(IG_POLL_INTERVAL).await,
                }
            }

            println!("[IG] timed out waiting for container_id={container_id}");
            bail!(
                "Instagram container {} timed out while processing",
                container_id
            )
        }

        async fn publish_container(&self, cl: &Client, container_id: &str) -> Result<String> {
            info!("IG STEP 4: Publish media container");
            println!(
                "[IG] step 3/3: publishing container_id={container_id} for ig_user_id={}",
                self.ig_user_id
            );

            let endpoint = format!(
                "{}/{}/{}/media_publish",
                GRAPH_BASE, IG_GRAPH_API_VERSION, self.ig_user_id
            );

            let resp = cl
                .post(&endpoint)
                .query(&[
                    ("creation_id", container_id),
                    ("access_token", self.secrets.access_token.as_str()),
                ])
                .send()
                .await?;

            let val = parse_ig_json(resp).await?;

            let media_id = val["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| anyhow!("missing Instagram media id in publish response: {val}"))?
                .to_string();

            info!("IG SUCCESS: published media_id={}", media_id);
            println!("[IG] published successfully: media_id={media_id}");
            Ok(media_id)
        }

        pub async fn send(self) -> Result<String> {
            println!("[IG] starting Reel send workflow");
            let cl = Client::builder()
                .timeout(Duration::from_secs(60 * 60))
                .build()?;

            let (container_id, upload_uri) = self.create_container(&cl).await?;

            if self.media.path.is_some() {
                let upload_endpoint = upload_uri.unwrap_or_else(|| {
                    format!(
                        "{}/ig-api-upload/{}/{}",
                        RUPLOAD_BASE, IG_GRAPH_API_VERSION, container_id
                    )
                });
                println!("[IG] uploading local video to {upload_endpoint}");
                self.media
                    .upload_to(&cl, &self.secrets.access_token, &upload_endpoint)
                    .await?;
            } else if self.media.url.is_some() {
                println!("[IG] hosted URL already passed to container, Meta will fetch it");
            }

            self.wait_until_ready(&cl, &container_id).await?;
            self.publish_container(&cl, &container_id).await
        }
    }

    async fn parse_ig_json(resp: Response) -> Result<Value> {
        let status = resp.status();
        let headers = resp.headers().clone();
        let text = resp.text().await?;
        let json: Value = serde_json::from_str(&text).unwrap_or(Value::Null);

        if !status.is_success() || json.get("error").is_some() || json.get("debug_info").is_some() {
            let formatted = format_ig_api_error(status, &headers, &text, &json);
            println!("[IG] API error:\n{formatted}");
            bail!("{formatted}");
        }

        Ok(json)
    }

    fn format_ig_api_error(
        status: StatusCode,
        headers: &HeaderMap,
        text: &str,
        json: &Value,
    ) -> String {
        let mut out = format!("HTTP {status}\n");

        for name in [
            "x-fb-trace-id",
            "x-fb-request-id",
            "x-fb-rev",
            "www-authenticate",
        ] {
            if let Some(v) = headers.get(name).and_then(|v| v.to_str().ok()) {
                out += &format!("{name}: {v}\n");
            }
        }

        if let Some(err) = json.get("error") {
            out += &format!(
                "error.code = {}\nerror.subcode = {}\nerror.type = {}\nerror.message = {}\nerror.user_msg = {}\n",
                err["code"],
                err["error_subcode"],
                err["type"],
                err["message"],
                err["error_user_msg"]
            );
        }

        if let Some(dbg) = json.get("debug_info") {
            out += &format!(
                "debug_info.type = {}\ndebug_info.message = {}\ndebug_info.retriable = {}\n",
                dbg["type"], dbg["message"], dbg["retriable"]
            );
        }

        out += &format!("raw body: {text}");
        out
    }
}