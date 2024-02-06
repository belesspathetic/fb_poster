use super::*;

const TEST_ACCESS_TOKEN: &str = "EAAOqY2sZBZAc4BO1izn7EYOxL8wfMvXcO5oWa0pUK7B381liqupuo1LWD5xgce7TVa9fYCZBdnZAbakT0t950ZAPoJrqFdTQendyvGfySSW7i8d2c1z5zwmQJJknFY5Am1t6aYhhUUUUaPdxk6wnp10cZAgpaQDfMViV22S1I567pzkZCoXu5gHXLmw83aOrQb6Uz9FajKTJhty3DUZB8i2CUboZD";
const TEST_PAGE_ID: &str = "128699823665014";

#[tokio::test]
//#[ignore = "it works"]
async fn send_post() -> anyhow::Result<()> {
    let secret = Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let body = Post::new(&secret, Some("hello".to_string()), None);
    body.send(&secret).await?;

    Ok(())
}

