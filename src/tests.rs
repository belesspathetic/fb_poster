#![allow(dead_code)]
const TEST_ACCESS_TOKEN: &str = "";
const TEST_PAGE_ID: &str = "";

#[tokio::test]
#[ignore = "it works"]
async fn send_post() -> anyhow::Result<()> {
    let secret = crate::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let body = crate::post::Post::new(secret)
        .with_message("hello".to_string())
        .with_link("https://google.com".to_string());
    body.send().await?;

    Ok(())
}

#[tokio::test]
//#[ignore = "it works"]
async fn send_photo() -> anyhow::Result<()> {
    let secret = crate::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.png".to_string();
    let body = crate::photo::Photo::new(secret, path);

    body.send().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "it works"]
async fn send_local_video_with_all_params() -> anyhow::Result<()> {
    let secret = crate::utils::Secrets::new(TEST_ACCESS_TOKEN, TEST_PAGE_ID);
    let path = "./test.mp4".to_string();
    let body = crate::video::Video::new(secret)
        .local_video(path)
        .with_title("title".to_string())
        .with_description("description".to_string())
        .with_thumbnail("./thumbnail.jpg".to_string());

    body.send().await?;
    Ok(())
}
