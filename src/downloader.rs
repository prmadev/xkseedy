use bytes;
use std::io::Cursor;
use thiserror::Error;

pub async fn download(input: String) -> Result<reqwest::Request, DownloaderErr> {
    let req = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse(&input).expect(""));

    Ok(req)
}

pub async fn write_file(file_name: String, information: bytes::Bytes) -> Result<(), DownloaderErr> {
    let mut file = std::fs::File::create(file_name)?;

    let mut content = Cursor::new(information);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

#[derive(Error, Debug)]
pub enum DownloaderErr {
    #[error("Problem getting file: {0}")]
    DownloaderErr(#[from] reqwest::Error),
    #[error("Cannot create file: {0}")]
    CannotCreateFile(#[from] std::io::Error),
}
