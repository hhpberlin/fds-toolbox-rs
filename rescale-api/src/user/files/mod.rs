use bytes::Bytes;

use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::api::RescaleApiClient;

/// Represents a response file with various properties.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    /// An integer indicating the type of the file.
    /// 1 = input file, 2 = template file, 3 = parameter file, 4 = script file, 5 = output file,
    /// 7 = design variable file, 8 = case file, 9 = optimizer file, 10 = temporary file.
    pub type_id: i32,
    /// The name of the file.
    pub name: String,
    /// The ISO8601 encoded date of when the file is uploaded.
    pub date_uploaded: String,
    /// For output files (typeId = 5), the relative path is the path relative to the root output folder
    /// (user/{user_id}/output/{job_id}/{run_id}/{relative_path}).
    pub relative_path: Option<String>,
    /// The key used to encrypt the files.
    pub encoded_encryption_key: Option<String>,
    /// The download URL of the file.
    pub download_url: String,
    /// A list of users of the file shared with.
    pub shared_with: Vec<String>,
    /// The decrypted file size in byte.
    pub decrypted_size: i32,
    /// The owner of the file.
    pub owner: String,
    /// The absolute path of the file being stored.
    pub path: String,
    /// If the file is already uploaded.
    pub is_uploaded: bool,
    /// If the file can be viewed in browser.
    pub view_in_browser: bool,
    /// The unique identifier of the file.
    pub id: String,
    /// If the file is already deleted.
    pub is_deleted: bool,
    /// The md5 hash of the file.
    pub md5: Option<String>,
}

/// Represents the response for retrieving files owned by the current user.
#[derive(Debug, Deserialize)]
pub struct FileResponse {
    /// The total number of files owned by the current user.
    pub count: i32,
    /// The URL that will return the next page of results.
    pub next: Option<String>,
    /// The URL that will return the previous page of results.
    pub previous: Option<String>,
    /// An array of File objects.
    pub results: Vec<FileMetadata>,
}

// async fn upload_file(token: &str, file_name: &str, file_content: &[u8]) -> Result<FileMetadata, reqwest::Error> {
//     let url = "https://platform.rescale.com/api/v2/files/contents/";
//     let mut headers = HeaderMap::new();
//     headers.insert(AUTHORIZATION, format!("Token {}", token).parse().unwrap());

//     let form = Form::new().part(
//         "file",
//         Part::bytes(file_content)
//             .file_name(file_name)
//             .mime_str("application/octet-stream")?,
//     );

//     let response = reqwest::Client::new()
//         .post(url)
//         .headers(headers)
//         .multipart(form)
//         .send()
//         .await?
//         .json::<FileMetadata>()
//         .await?;

//     Ok(response)
// }

// pub async fn list_all(client: &RescaleApiClient) -> Result<FileResponse, reqwest::Error> {
//     let mut files = list_filtered(client, None, None).await?;
//     let mut next = files.next.clone();

//     while let Some(url) = next {
//         let response = client
//             .request(Method::GET, url)
//             .send()
//             .await?
//             .json::<FileResponse>()
//             .await?;

//         files.results.extend(response.results);
//         next = response.next;
//     }

//     Ok(files)
// }

pub async fn list(client: &RescaleApiClient) -> Result<FileResponse, reqwest::Error> {
    list_filtered(client, None, None).await
}

pub async fn list_filtered(
    client: &RescaleApiClient,
    search: Option<&str>,
    owner: Option<i32>,
) -> Result<FileResponse, reqwest::Error> {
    /// Query parameters for retrieving files owned by the current user.
    #[derive(Debug, Serialize)]
    struct FileQuery<'a> {
        /// The (partial) file name to search for.
        search: Option<&'a str>,
        /// Specify owner=1 to exclude files that have been shared with the current user.
        owner: Option<i32>,
    }

    let response = client
        .request(Method::GET, "files/")
        .query(&FileQuery { search, owner });

    // dbg!(response.try_clone().unwrap().send().await?.json::<serde_json::Value>().await?);

    let response = response.send().await?.json::<FileResponse>().await?;

    Ok(response)
}

pub async fn delete(client: &RescaleApiClient, file_id: &str) -> Result<(), reqwest::Error> {
    let _response = client
        .request(Method::DELETE, format!("files/{file_id}/"))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}

pub async fn get_bytes(client: &RescaleApiClient, file_id: &str) -> Result<Bytes, reqwest::Error> {
    let response = client
        .request(Method::GET, format!("files/{file_id}/contents/"))
        .send()
        .await?
        .error_for_status()?;

    response.bytes().await
}

pub async fn get_lines(
    client: &RescaleApiClient,
    file_id: &str,
) -> Result<Vec<String>, reqwest::Error> {
    #[derive(Debug, Deserialize)]
    struct FileContent {
        lines: Vec<String>,
    }

    let response = client
        .request(Method::GET, format!("files/{file_id}/lines/"))
        .send()
        .await?
        .json::<FileContent>()
        .await?;

    Ok(response.lines)
}
