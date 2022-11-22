mod downloader;
mod lib;

use tokio::main;

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // defining a client for downloading stuff
    let client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(std::time::Duration::new(20, 0))
        .build()?;
    // --- making a list of links --- //

    // create a request for the latest comic
    println!("::\tdownloading the latest comic.");
    let latest_comic_request = downloader::download(String::from(lib::LATEST)).await?;

    // downloading the latest comic data
    let latest_comic_data = lib::ComicInput::build(
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
    let links = lib::create_list(latest_comic_data.num() - 2650);

    // --- download data --- //
    // create concurrency handlers for requests of data of each item
    println!("::\tdownloading datas");
    let mut handlers = vec![];
    for link in links {
        handlers.push(tokio::task::spawn(async move {
            let resp = reqwest::Client::builder()
                .connect_timeout(std::time::Duration::new(20, 0))
                .connection_verbose(true)
                .build()
                .expect("we just had an incident")
                .get(reqwest::Url::parse(&link).unwrap())
                .send()
                .await
                .expect("shit happened");

            if resp.status().is_success() {
                let data = lib::ComicInput::build(resp.text().await.unwrap()).unwrap();

                let resp_comic = reqwest::Client::new()
                    .execute(downloader::download(data.image_url()).await.expect(""))
                    .await
                    .unwrap();
                downloader::write_file(
                    format!("{}.jpg", data.num()),
                    bytes::Bytes::from(resp_comic.bytes().await.expect("")),
                )
                .await
                .expect("error in writing to file");

                println!(":: downnloaded:{}", data.num());
            }
        }));
    }
    for handle in handlers {
        handle.await?;
    }

    // let id = lib::ComicInput::build(a.await.unwrap()).expect("ooooooooo");
    // --- downloading  comics --- //

    // println!(
    //     "::\tdownloading comic. there are {} comics to download",
    //     comic_datas.len()
    // );

    // let mut count: u16 = 0;
    // // for (k, v) in downlad_handlers.iter_mut() {
    //     downloader::write_file(
    //         format!("{}.jpg", k),
    //         bytes::Bytes::from(v.await?.await?.text().await?),
    //     )
    //     .await?;
    //     count += 1;
    //     println!("::\tdownloaded {}/{} ", count, comic_datas.len());
    // }

    // let g = reqwest::get(comic_data.image_url()).await?;

    // downloader::write_file(
    //     String::from(comic_data.num().to_string()),
    //     &g.bytes().await?,
    // )
    // .await?;

    Ok(())
}

// async fn download(link: String) -> Option<lib::ComicInput> {
// async fn download(
//     link: String,
// ) -> impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>> {
//     println!("::\t making request for downloading: {link}");
//     let req_data = downloader::download(link).await.unwrap();

//     let client = reqwest::Client::builder()
//         .no_proxy()
//         .connect_timeout(std::time::Duration::new(5, 0))
//         .https_only(true)
//         .build()
//         .unwrap();

//     client.execute(req_data)
//     // if !resp.status().is_success() {
//     // println!("status not 200");
//     return None;
//     // };

//     // Some(lib::ComicInput::build(&resp.text().await.expect("ooooooooo")).unwrap())
// }
