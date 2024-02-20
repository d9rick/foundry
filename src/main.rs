extern crate anyhow;
extern crate clap;
extern crate futures_util;
extern crate indicatif;
extern crate reqwest;

use anyhow::Result;
use clap::{Arg, Command};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() {
    // initialize the command and its arguments
    let matches = Command::new("foundry")
        .version("0.1.0") // declare program metadata
        .author("Derick Walker <dcwalker@usc.edu>")
        .about("A simple wget clone written in rust.")
        .arg(
            Arg::new("URL") // define arguments needed
                .help("The URL to fetch data from.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("Output File") // define arguments needed
                .default_value("downloaded_file")
                .help("The file to dowload your data to.")
                .index(2),
        )
        .get_matches();

    // print starting information
    let url = matches.get_one::<String>("URL").unwrap();
    let filename = matches.get_one::<String>("Output File").unwrap();

    println!("Url is: {}", url);
    println!("Filename is: {}", filename);

    // download the file
    println!("Downloading file...");
    let result = download(&url, false, filename).await;

    match result {
        Ok(_) => println!("File downloaded successfully!"),
        Err(e) => println!("Error downloading file: {}", e),
    }
}

async fn download(target: &str, quiet_mode: bool, fname: &str) -> Result<()> {
    let url = reqwest::Url::parse(target)?;
    let client = Client::new();

    let resp = client.get(url.clone()).send().await?.error_for_status()?;

    if quiet_mode {
        println!("HTTP request sent... {}", resp.status());
    }
    let pb = create_progress_bar(quiet_mode, "Downloading", resp.content_length());

    let mut file = AsyncFile::create(fname).await?;

    let mut stream = resp.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?; // Correctly handle the Result<Bytes, Error>
        file.write_all(&chunk).await?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Download complete");

    Ok(())
}

fn create_progress_bar(quiet_mode: bool, msg: &'static str, length: Option<u64>) -> ProgressBar {
    let bar = match quiet_mode {
        true => ProgressBar::hidden(),
        false => match length {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        },
    };

    bar.set_message(msg);
    match length.is_some() {
        true => bar
            .set_style(ProgressStyle::default_bar()
                .template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} eta: {eta}").unwrap()
                .progress_chars("=> ")),
        false => bar.set_style(ProgressStyle::default_spinner()),
    };

    bar
}
