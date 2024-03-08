mod constants;
use constants::{TEST_ACCESS_TOKEN, TEST_PAGE_ID};

#[tokio::test]
#[ignore = "it works"]
async fn send_local_video_with_all_params() -> anyhow::Result<()> {
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.mp4".to_string();
    let body = fb_poster::video::Video::new(secret)
        .local_video(path)
        .with_title("title".to_string())
        .with_description("description".to_string());

    body.send().await?;
    Ok(())
}
