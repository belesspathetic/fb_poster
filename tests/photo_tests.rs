mod constants;
use constants::{TEST_ACCESS_TOKEN, TEST_PAGE_ID};

#[tokio::test]
#[ignore = "it works"]
async fn send_photo() -> anyhow::Result<()> {
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.png".to_string();
    let body = fb_poster::photo::Photo::new(secret, path);

    body.send().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "it works"]
async fn send_photo_with_message() -> anyhow::Result<()> {
    let message = "hello world!".to_string();
    let secret = fb_poster::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.png".to_string();
    let body = fb_poster::photo::Photo::new(secret, path).with_message(message);

    body.send().await?;

    Ok(())
}
