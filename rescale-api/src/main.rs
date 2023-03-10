use rescale_api::{api::RescaleApiClient, user::files};

pub mod api;
pub mod user;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = env!("RESCALE_API_KEY");
    let client = RescaleApiClient::new_eu(key);

    let files = files::list_filtered(&client, Some(".smv"), None).await?;
    dbg!(&files);
    let file = &files.results[4];
    // dbg!(file);
    dbg!(files::get_bytes(&client, &file.id).await?);

    Ok(())
}
