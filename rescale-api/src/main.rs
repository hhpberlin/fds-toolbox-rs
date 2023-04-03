use std::{env, fs};

use fds_toolbox_core::formats::smv::Smv;
use rescale_api::{api::RescaleApiClient, user::files};

pub mod api;
pub mod user;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = env!("RESCALE_API_KEY");
    let client = RescaleApiClient::new_eu(key);

    if env::args().any(|x| x == "dl") {
        let files = files::list_filtered(&client, Some("_devc.csv"), None).await?;
        for file in files.results {
            let content = files::get_bytes(&client, &file.id).await?;
            fs::write(file.name, content)?;
        }
    }

    let files = files::list_filtered(&client, Some(".smv"), None).await?;
    dbg!(&files);
    // dbg!(file);
    // dbg!(files::get_bytes(&client, &file.id).await?);

    for file in files.results {
        println!("-- Downloading {:?}", file.path);
        let content = files::get_bytes(&client, &file.id).await?;

        println!(" > {} bytes", content.len());

        // convert content from Bytes to &str
        let text = std::str::from_utf8(content.as_ref())?;

        let _sim = Smv::parse_with_warn_stdout(text).unwrap();
    }

    Ok(())
}
