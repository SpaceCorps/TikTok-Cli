//! `tiktok hashtag <HASHTAGS>` command.

use std::io::IsTerminal;

use serde_json::{Value, json};

use crate::account;
use crate::cli::HashtagArgs;
use crate::error::{Error, Result};
use crate::output;

pub fn run(args: HashtagArgs) -> Result<()> {
    let resolved = account::resolve(&args.auth)?;

    let hashtags: Vec<String> = args
        .hashtags
        .split(',')
        .map(str::trim)
        .map(|s| s.trim_start_matches('#'))
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    if hashtags.is_empty() {
        return Err(Error::invalid("At least one hashtag is required."));
    }

    if std::io::stderr().is_terminal() {
        eprintln!("Scraping {} hashtag(s) (this may take a while)...", hashtags.len());
    }

    let mut body = serde_json::Map::new();
    body.insert("hashtags".into(), json!(hashtags));
    body.insert("resultsPerPage".into(), json!(args.limit));
    if let Some(comments) = args.comments {
        body.insert("commentsPerPost".into(), json!(comments));
    }

    let client = resolved.client();
    let res = client.scrape(&Value::Object(body))?;
    output::write(&res);
    Ok(())
}
