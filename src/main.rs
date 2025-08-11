use anyhow::Context;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::fs::write;

#[derive(Parser)]
struct Cli {
    mid: String,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct Video {
    aid: String,
    bid: String,
    title: String,
    cover: String,
    time: u64,
}
#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();
    let mut aid = "".to_string();
    let mut videos = vec![];
    loop {
        let data = loop {
            let result: Result<_, anyhow::Error> = async {
                let data: serde_json::Value = client
                    .get(format!(
                        "https://app.biliapi.com/x/v2/space/archive/cursor?vmid={}&aid={}",
                        &cli.mid, &aid,
                    ))
                    .send()
                    .await?
                    .json()
                    .await?;
                let arr = data["data"]["item"]
                    .as_array()
                    .context("item is not array")?
                    .clone();
                let has_next = data["data"]["has_next"]
                    .as_bool()
                    .context("has_next is not array")?;
                Ok((arr, has_next))
            }
            .await;
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    println!("{err:#}");
                    continue;
                }
            };
            break data;
        };
        for item in data.0 {
            aid = item["param"].as_str().unwrap_or_default().to_string();
            let title = item["title"].as_str().unwrap_or_default().to_string();
            videos.push(Video {
                aid: aid.clone(),
                bid: item["bvid"].as_str().unwrap_or_default().to_string(),
                title: title.clone(),
                cover: item["cover"].as_str().unwrap_or_default().to_string(),
                time: item["ctime"].as_u64().unwrap_or_default(),
            });
            println!("{title}")
        }
        if !data.1 {
            break;
        }
    }
    write(
        "videos.json",
        serde_json::to_string_pretty(&videos).unwrap(),
    )
    .await
    .unwrap();
}
