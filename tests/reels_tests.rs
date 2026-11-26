mod constants;
use constants::{TEST_ACCESS_TOKEN, TEST_PAGE_ID, TEST_IG_USER_ID};

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

#[tokio::test]
#[ignore = "it works"]
async fn send_reel_with_thumbnail() -> anyhow::Result<()> {
    init_logger();
    let description = "description".to_string();
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.mp4".to_string();
    let thumb = "./test.png".to_string();
    let body = fb_poster::reels::Reels::new(secret)
        .local_video(path)
        .with_description(description)
        .with_thumbnail(thumb);

    body.send().await?;

    Ok(())
}

#[tokio::test]
async fn send_reel_to_instagram() -> anyhow::Result<()> {
    init_logger();
    let description = "description".to_string();
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.mp4".to_string();
    let body = fb_poster::reels::Reels::new(secret)
        .local_video(path)
        .with_description(description)
        .instagram(TEST_IG_USER_ID);

    let res = body.send().await?;
    assert!(!res.facebook_id.is_empty());
    assert!(res.instagram_id.as_deref().is_some_and(|id| !id.is_empty()));

    Ok(())
}

#[tokio::test]
#[ignore = "it works"]
async fn send_reel_to_instagram_from_secrets() -> anyhow::Result<()> {
    init_logger();
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID)
        .with_ig_user_id(TEST_IG_USER_ID);
    let path = "./test.mp4".to_string();
    let body = fb_poster::reels::Reels::new(secret)
        .local_video(path)
        .instagram("");

    let res = body.send().await?;
    assert!(res.instagram_id.is_some());

    Ok(())
}

fn init_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}