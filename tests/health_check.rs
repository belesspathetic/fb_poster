#![allow(dead_code)]

#[tokio::test]
#[ignore = "it works"]
async fn health_check() -> anyhow::Result<()> {
    let url = "https://facebook.com/";
    let cl = reqwest::Client::new();

    let resp = cl.get(url).send().await?;

    assert!(resp.status().is_success());

    Ok(())
}
