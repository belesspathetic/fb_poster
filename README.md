An unofficial Rust API client for [Facebook](https://facebook.com/) post uploads.

## ⚙️ Requirements

- Create a Fecebook app on [Developer Page](https://developers.facebook.com/)

- Your Facebook app must be in Live mode to make your posts visible for others.

- Take ACCESS_TOKEN from [Graph API Explorer](https://developers.facebook.com/tools/explorer/).
You can get 2 months token by pressing info "i" icon ^_^

- Add the desired permissions to allow your app to make posts.
    - '''pages_manage_engagement'''
    - '''pages_manage_posts'''
    - '''pages_read_engagement'''
    - '''pages_read_user_engagement'''
    - '''publish_video''' permission, if you are publishing a video to the Page

- Take PAGE_ID from page that you planning to do post.

- More useful information you can find in [Offical Facebook API Documentation](https://developers.facebook.com/docs/graph-api/)
Current version v19.0

## 🗣️ Usage

'''rust
use fb_poster::*;
use anyhow::{Ok, Result};

const ACCESS_TOKEN: &str = "YOUR_ACCESS_TOKEN";
const PAGE_ID: &str = "YOUR_PAGE_ID";


#[tokio::main]
async fn main() -> Result<()>{
    // Bring your secrets into a scope
    let secrets = Secrets::new(ACCESS_TOKEN, PAGE_ID);

    // Build a body for a request
    let your_message = "Hello World!".to_string();
    let your_link = "www.internet.net".to_string(); // you can left this field as None 
    let body = Post::new(&secrets, Some(your_message), Some(your_link)); // if you dont want upload a link

    // Sending and get repsonse
    body.send(&secrets).await?;

    Ok(())
}
'''


## Features

- [x] Post
- [ ] Photo
- [ ] Video
- [ ] Live Video

