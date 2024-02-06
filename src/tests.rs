use super::*;

const TEST_ACCESS_TOKEN: &str = "";
const TEST_PAGE_ID: &str = "";

#[tokio::test]
#[ignore = "it works"]
async fn send_post() -> anyhow::Result<()> {
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let body = Post::new(&secret, Some("hello".to_string()), None);
    body.send(&secret).await?;

    Ok(())
}

#[tokio::test]

async fn send_photo() -> Result<()> {
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = r"test.png".to_string();
    let body = Photo::new(&secret, path);

    body.send(secret).await?;
    Ok(())
}
