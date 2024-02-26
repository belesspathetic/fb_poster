An unofficial Rust API client for [Facebook](https://facebook.com/) post uploads.

## 🪛 Requirements

- Create a Fecebook app on [Developer Page](https://developers.facebook.com/)

- Your Facebook app must be in Live mode to make your posts visible for others.

- Take ACCESS_TOKEN from [Graph API Explorer](https://developers.facebook.com/tools/explorer/).
You can get 2 months token by pressing info icon.

- Add the desired permissions to allow your app to make posts.
    - ```pages_manage_engagement```
    - ```pages_manage_posts```
    - ```pages_read_engagement```
    - ```pages_read_user_engagement```
    - ```publish_video``` permission, if you need to publish a video

- Take PAGE_ID from page that you planning to do post.

- More useful information you can find in [Offical Facebook API Documentation](https://developers.facebook.com/docs/graph-api/)
Current version v19.0.

## 🪧 Usage

### Post

```rust
use fb_poster::*;
use anyhow::{Ok, Result};

const ACCESS_TOKEN: &str = "YOUR_ACCESS_TOKEN";
const PAGE_ID: &str = "YOUR_PAGE_ID";


#[tokio::main]
async fn main() -> Result<()> {
    // Bring your secrets into a scope
    let secrets = Secrets::new(ACCESS_TOKEN, PAGE_ID);

    
    let message = "Your message".to_string();
    let link = "https://your_link".to_string();

    // Build a body for a request
    let body = Post::new(secrets)
    .with_message(message)
    .with_link(link);

    // Sending and get repsonse
    body.send().await?;

    Ok(())
}
```

### Photo

```rust
use fb_poster::*;
use anyhow::{Ok, Result};

const ACCESS_TOKEN: &str = "YOUR_ACCESS_TOKEN";
const PAGE_ID: &str = "YOUR_PAGE_ID";


#[tokio::main]
async fn main() -> Result<()> {
    // Bring your secrets into a scope
    let secrets = Secrets::new(ACCESS_TOKEN, PAGE_ID);

    let path = "/path/to/photo.png".to_string();

    // Build a body for a request
    let body = Photo::new(secrets, path);

    // Sending and get repsonse
    body.send(&secrets).await?;

    Ok(())
}
```

### Video

```rust
use fb_poster::*;
use anyhow::{Ok, Result};

const ACCESS_TOKEN: &str = "YOUR_ACCESS_TOKEN";
const PAGE_ID: &str = "YOUR_PAGE_ID";


#[tokio::main]
async fn main() -> Result<()> {
    // Bring your secrets into a scope
    let secrets = Secrets::new(ACCESS_TOKEN, PAGE_ID);


    let path = "/path/to/video".to_string(); // or url for .hosted_video()
    let title = "Title".to_string();
    let description = "Description".to_string();
    let thumb = "path/to/thumb".to_string();

    // Build a body for a request
    let body = Video::new(secrets)
    .local_video(path)
    .with_title(title)
    .with_description(description)
    .with_thumbnail(thumb)


    // Sending and get repsonse
    body.send().await?;

    Ok(())
}
```

## ✅ Features

- [x] Post
  - [x] With Message
  - [x] With Link
- [x] Photo

**Non-Resumable Upload (Video limitation is 1GB 20min)**

- [x] Video
  - [x] Local Video
  - [x] Hosted Video
  - [x] With Title
  - [x] With Description
  - [x] With Thumbnail

