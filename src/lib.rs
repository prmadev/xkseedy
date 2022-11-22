use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct ComicInput {
    month: String,
    num: u32,
    link: String,
    year: String,
    news: String,
    safe_title: String,
    transcript: String,
    alt: String,
    img: String,
    title: String,
    day: String,
}

pub const LATEST: &str = "https://xkcd.com/info.0.json";

pub fn create_list(latest: u32) -> Vec<String> {
    let mut download_list: Vec<String> = vec![];
    for i in 1..=latest {
        download_list.push(format!("https://xkcd.com/{}/info.0.json", i))
    }

    download_list
}

impl ComicInput {
    pub fn build(resp: String) -> Result<ComicInput, ComicInputErr> {
        Ok(serde_json::from_str::<ComicInput>(&resp)
            .map_err(|e| ComicInputErr::SerializationProblem(e, String::from(resp)))?)
    }

    pub fn image_url(&self) -> String {
        self.img.to_owned()
    }

    pub fn num(&self) -> u32 {
        self.num
    }

    pub fn title(&self) -> String {
        self.safe_title.to_owned()
    }
}

#[derive(Error, Debug)]
pub enum ComicInputErr {
    #[error("serialization of the file went side ways: {0}, {0}")]
    SerializationProblem(serde_json::Error, String),
}
