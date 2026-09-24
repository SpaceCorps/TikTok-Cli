//! `tiktok profile <USERNAMES>` command.

use std::io::IsTerminal;

use serde_json::{Value, json};

use crate::account;
use crate::cli::ProfileArgs;
use crate::error::{Error, Result};
use crate::output;

pub fn run(args: ProfileArgs) -> Result<()> {
    let resolved = account::resolve(&args.auth)?;

    let profiles: Vec<String> =
        args.usernames.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if profiles.is_empty() {
        return Err(Error::invalid("At least one username or profile URL is required."));
    }

    let mut sections: Vec<String> =
        args.sections.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if sections.is_empty() {
        sections.push("videos".to_string());
    }

    if std::io::stderr().is_terminal() {
        eprintln!("Scraping {} profile(s) (this may take a while)...", profiles.len());
    }

    let mut body = serde_json::Map::new();
    body.insert("profiles".into(), json!(profiles));
    body.insert("resultsPerPage".into(), json!(args.limit));
    body.insert("profileSorting".into(), json!(args.sort));
    body.insert("profileScrapeSections".into(), json!(sections));
    body.insert("excludePinnedPosts".into(), json!(args.exclude_pinned));
    if let Some(since) = &args.since {
        body.insert("oldestPostDateUnified".into(), json!(since));
    }
    if let Some(until) = &args.until {
        body.insert("newestPostDate".into(), json!(until));
    }
    if let Some(comments) = args.comments {
        body.insert("commentsPerPost".into(), json!(comments));
    }

    let client = resolved.client();
    let res = client.scrape(&Value::Object(body))?;
    output::write(&res);
    Ok(())
}
