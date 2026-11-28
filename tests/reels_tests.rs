mod constants;
use constants::{TEST_ACCESS_TOKEN, TEST_IG_USER_ID, TEST_PAGE_ID, TEST_SERVER_URL};
use fb_poster::{
    reel::{facebook, instagram},
    utils::Secrets,
};

#[tokio::test]
#[ignore = "publishes to Facebook"]
async fn send_reel() -> anyhow::Result<()> {
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let reel_id: String = facebook::Reel::new(secret)
        .local_video("./test.mp4")
        .send()
        .await?;

    assert!(!reel_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Facebook"]
async fn send_reel_with_description() -> anyhow::Result<()> {
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let reel_id: String = facebook::Reel::new(secret)
        .local_video("./test_video.mp4")
        .with_description("description")
        .send()
        .await?;

    assert!(!reel_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Facebook"]
async fn send_reel_with_thumbnail() -> anyhow::Result<()> {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let reel_id: String = facebook::Reel::new(secret)
        .local_video("./test_video.mp4")
        .with_description("description")
        .with_thumbnail("./test.png")
        .send()
        .await?;

    assert!(!reel_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Facebook from a hosted URL"]
async fn send_hosted_reel_to_facebook() -> anyhow::Result<()> {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let reel_id: String = facebook::Reel::new(secret)
        .hosted_video(TEST_SERVER_URL)
        .with_description("hosted description")
        .send()
        .await?;

    assert!(!reel_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Instagram"]
async fn send_reel_to_instagram() -> anyhow::Result<()> {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let media_id: String = instagram::Reel::new(secret, TEST_IG_USER_ID)?
        .local_video("./test.mp4")
        .with_description("description")
        .send()
        .await?;

    assert!(!media_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Instagram from a hosted URL"]
async fn send_hosted_reel_to_instagram() -> anyhow::Result<()> {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let media_id: String = instagram::Reel::new(secret, TEST_IG_USER_ID)?
        .hosted_video(TEST_SERVER_URL)
        .with_description("hosted description")
        .send()
        .await?;

    assert!(!media_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Instagram"]
async fn send_long_video_to_instagram() -> anyhow::Result<()> {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let media_id: String = instagram::Reel::new(secret, TEST_IG_USER_ID)?
        .local_video("./long_test.mp4")
        .with_description("long video description")
        .send()
        .await?;

    assert!(!media_id.is_empty());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Instagram"]
async fn send_hosted_reel_rejects_non_http_url() -> anyhow::Result<()> {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let result = instagram::Reel::new(secret, TEST_IG_USER_ID)?
        .hosted_video("ftp://example.com/video.mp4")
        .send()
        .await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
#[ignore = "publishes to Instagram"]
async fn send_reel_to_instagram_empty_id_fails() {
    init_logger();
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let created = instagram::Reel::new(secret, "");
    assert!(created.is_err());
}

fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}