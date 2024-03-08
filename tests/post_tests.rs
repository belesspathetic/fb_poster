mod constants;

use constants::{TEST_ACCESS_TOKEN, TEST_PAGE_ID};

#[tokio::test]
#[ignore = "it works"]
async fn send_post() -> anyhow::Result<()> {
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let body = fb_poster::post::Post::new(secret)
        .with_message("hello".to_string())
        .with_link("https://google.com".to_string());
    body.send().await?;

    Ok(())
}
