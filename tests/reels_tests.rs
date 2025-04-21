mod constants;
use constants::{TEST_ACCESS_TOKEN, TEST_PAGE_ID};

#[tokio::test]
#[ignore = "it works"]
async fn send_reel() -> anyhow::Result<()> {
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.mp4".to_string();
    let body = fb_poster::reels::Reels::new(secret).local_video(path);

    body.send().await?;

    Ok(())
}

#[tokio::test]
#[ignore = "it works"]
async fn send_reel_with_description() -> anyhow::Result<()> {
    let description = "description".to_string();
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.mp4".to_string();
    let body = fb_poster::reels::Reels::new(secret)
        .local_video(path)
        .with_description(description);

    body.send().await?;

    Ok(())
}
