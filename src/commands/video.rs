//! `tiktok video <URLS>` command.

use std::io::IsTerminal;

use serde_json::{Value, json};

use crate::account;
use crate::cli::VideoArgs;
use crate::error::{Error, Result};
use crate::output;

pub fn run(args: VideoArgs) -> Result<()> {
    let resolved = account::resolve(&args.auth)?;

    let urls: Vec<String> = args.urls.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if urls.is_empty() {
        return Err(Error::invalid("At least one video URL is required."));
    }

    if std::io::stderr().is_terminal() {
        eprintln!("Scraping {} video(s) (this may take a while)...", urls.len());
    }

    let mut body = serde_json::Map::new();
    body.insert("postURLs".into(), json!(urls));
    body.insert("scrapeRelatedVideos".into(), json!(args.related.is_some()));
    body.insert("resultsPerPage".into(), json!(args.related.unwrap_or(1)));
    if let Some(comments) = args.comments {
        body.insert("commentsPerPost".into(), json!(comments));
    }

    let client = resolved.client();
    let res = client.scrape(&Value::Object(body))?;
    output::write(&res);
    Ok(())
}
