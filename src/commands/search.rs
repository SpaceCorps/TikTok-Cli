//! `tiktok search <QUERIES>` command.

use std::io::IsTerminal;

use serde_json::{Value, json};

use crate::account;
use crate::cli::SearchArgs;
use crate::error::{Error, Result};
use crate::output;

pub fn run(args: SearchArgs) -> Result<()> {
    let resolved = account::resolve(&args.auth)?;

    let queries: Vec<String> =
        args.queries.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if queries.is_empty() {
        return Err(Error::invalid("At least one search query is required."));
    }

    let search_section = if args.users || args.section.eq_ignore_ascii_case("user") {
        "/user"
    } else if args.section.eq_ignore_ascii_case("video") {
        "/video"
    } else {
        ""
    };

    if std::io::stderr().is_terminal() {
        eprintln!("Searching {} query/queries (this may take a while)...", queries.len());
    }

    let mut body = serde_json::Map::new();
    body.insert("searchQueries".into(), json!(queries));
    body.insert("resultsPerPage".into(), json!(args.limit));
    body.insert("searchSection".into(), json!(search_section));
    if let Some(profiles) = args.profiles {
        body.insert("maxProfilesPerQuery".into(), json!(profiles));
    }
    if let Some(comments) = args.comments {
        body.insert("commentsPerPost".into(), json!(comments));
    }

    let client = resolved.client();
    let res = client.scrape(&Value::Object(body))?;
    output::write(&res);
    Ok(())
}
