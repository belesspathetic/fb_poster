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

