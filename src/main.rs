mod downloader;
mod lib;

use async_recursion::async_recursion;
use lib::{create_list, ComicInput, LATEST};
use reqwest::Client;
use std::{fs, path::PathBuf, time};
use thiserror::Error;
use tokio::main;

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conf = Config {
        at_a_time: 100,
        path: PathBuf::from("/home/a/xkcd"),
    };

    if !conf.path.is_dir() {
        let _ = fs::create_dir_all(&conf.path)?;
    }

    // defining a client for downloading stuff
    let client = Client::builder()
        .no_proxy()
        .connect_timeout(time::Duration::new(20, 0))
        .build()?;
    // --- making a list of links --- //

    // create a request for the latest comic
    println!("::\tdownloading the latest comic.");
    let latest_comic_request = downloader::download(String::from(LATEST)).await?;

    // downloading the latest comic data
    let latest_comic_data = ComicInput::build(
        client
            .execute(latest_comic_request)
            .await?
            .text()
            .await
            .expect("ooooooooo"),
    )
    .unwrap();

    println!("::\tlatest comic is numbered: {}", latest_comic_data.num());

    // create a list of links that will be used to doesnload comics
    let links = create_list(latest_comic_data.num());
    println!("::\tgenerated link lists");
    // downloading items in a non-rude manner
    splitted_processor(links, conf.clone()).await?;

    Ok(())
}

#[async_recursion]
async fn splitted_processor(links: Vec<String>, conf: Config) -> Result<(), MassProcessError> {
    if links.len() > conf.at_a_time {
        println!("::\tsplitting files.");
        let (part_one, part_two) = links.split_at(conf.at_a_time);
        mass_processor(part_one.to_vec(), conf.path.clone()).await?;
        println!("::\tone split done, recursing to the next");
        splitted_processor(part_two.to_vec(), conf).await?;
    } else {
        mass_processor(links, conf.path.clone()).await?;
    }
    Ok(())
}

async fn mass_processor(links: Vec<String>, path: PathBuf) -> Result<(), MassProcessError> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(1);

    println!("::\tproccessing  {} links.", links.len());

    let mut handlers = vec![];

    for link in links {
        let p = path.clone();
        let txx = tx.clone();
        handlers.push(tokio::task::spawn(async move {
            item_processor(link, p, txx).await
        }));
    }

    // _ = tokio::task::spawn(async move {
    //     while let Some(message) = rx.recv().await {
    //         println!(":: {}", message);
    //     }
    // });

    for handle in handlers {
        handle.await??;
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum MassProcessError {
    #[error("could not joing different future handlers: {0}")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("could not process a comic: {0}")]
    ItemProcessError(#[from] ItemProcessError),
}

async fn item_processor(
    link: String,
    path: PathBuf,
    txx: tokio::sync::mpsc::Sender<String>,
) -> Result<(), ItemProcessError> {
    let resp = Client::builder()
        .connect_timeout(time::Duration::new(20, 0))
        .connection_verbose(true)
        .build()
        .expect("we just had an incident")
        .get(reqwest::Url::parse(&link).unwrap())
        .send()
        .await
        .expect("shit happened");

    // txx.send(format!("started connection for : {}", link))
    // .await
    // .expect("join problems, all over!");

    if resp.status().is_success() {
        let data = ComicInput::build(resp.text().await.unwrap()).unwrap();

        let resp_comic = Client::new()
            .execute(downloader::download(data.image_url()).await.expect(""))
            .await
            .unwrap();

        // txx.send(format!("downloaded: {}", link))
        // .await
        // .expect("join problems, all over!");

        let file_path = format!("{}/{}.jpg", path.to_string_lossy(), data.num());
        downloader::write_file(
            file_path.clone(),
            bytes::Bytes::from(resp_comic.bytes().await.expect("")),
        )
        .await
        .expect("error in writing to file");

        // txx.send(format!("saved to: {}", file_path))
        // .await
        // .expect("join problems, all over!");
        println!("saved to: {}", file_path)
    }
    Ok(())
}

#[derive(Error, Debug)]
enum ItemProcessError {}

#[derive(Clone)]
struct Config {
    path: PathBuf,
    at_a_time: usize,
}
