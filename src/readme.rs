//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a CLAUDE.md; `--json` gives the same rules as data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "tiktok",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run tiktok accounts list or login",
            },
        });
        return;
    }
    println!("{README}");
}

/// The Apify / TikTok scraper API version.
pub const API_VERSION: &str = "2.0.0";

const RULES: &[&str] = &[
    "Pass --account <name>, --api-key <key>, or set APIFY_TOKEN / TIKTOK_API_KEY in your environment.",
    "Run 'tiktok accounts list' to see configured accounts, or 'tiktok login' to authenticate.",
    "On code auth_required, stop and surface the remediation string. Do not retry.",
    "Use --json when parsing command output programmatically or in agent tool loops.",
    "TikTok scraping calls run synchronously via Apify and may take several seconds.",
    "All errors print a structured envelope to stderr and exit with non-zero status.",
];

const README: &str = r#"# tiktok - agent operating manual

A native CLI for scraping TikTok videos, user profiles, hashtags, and search results via Apify.
Results are YAML on stdout by default, errors are YAML on stderr, and `--json` switches both to JSON.
Prompts and warnings go to stderr, so stdout is always clean and safe to parse.

## Authentication & Accounts

Every command that accesses TikTok/Apify accepts either `--account <name>` (short `-a`)
using credentials stored securely in your OS keystore, or `--api-key <key>` (or `APIFY_TOKEN` / `TIKTOK_API_KEY`).

### Managing accounts

    tiktok login [<name>] [--api-key <key>]  # prompts or accepts API token
    tiktok accounts add <name> --api-key <key> [--force]
    printf %s "$TOKEN" | tiktok accounts add <name> --api-key-stdin
    tiktok accounts list [--check]
    tiktok accounts test <name>
    tiktok accounts remove <name> --yes

`add` validates the token against the Apify API (`users/me`) before storing it in your operating system
keystore (macOS Keychain, Windows DPAPI, or Linux Secret Service). Non-secret metadata is stored
in `config.yaml`.

## Core Commands

### 1. Profile — Scrape videos from user profiles

    tiktok profile <USERNAMES> [OPTIONS]

Arguments:
    <USERNAMES>                Comma-separated TikTok usernames or URLs (e.g. nike,adidas or https://www.tiktok.com/@nike)

Options:
    --limit <N>                Number of videos per profile (default: 1)
    --sort <ORDER>             Sort order: latest, popular, oldest (default: latest)
    --since <DATE>             Only videos published after this date (e.g. 2025-01-01)
    --until <DATE>             Only videos published before this date
    --sections <SECTIONS>      Sections to scrape: videos, reposts (default: videos)
    --exclude-pinned           Exclude pinned posts
    --comments <N>             Max comments per post

Examples:
    tiktok profile nike --limit 10
    tiktok profile https://www.tiktok.com/@nike --sort popular --json

### 2. Hashtag — Scrape videos by hashtag

    tiktok hashtag <HASHTAGS> [OPTIONS]

Arguments:
    <HASHTAGS>                 Comma-separated hashtags (e.g. fitness,workout or #coding)

Options:
    --limit <N>                Number of videos per hashtag (default: 1)
    --comments <N>             Max comments per post

Examples:
    tiktok hashtag "coding" --limit 5
    tiktok hashtag "fitness,workout" --limit 20 --json

### 3. Search — Search TikTok for videos or users

    tiktok search <QUERIES> [OPTIONS]

Arguments:
    <QUERIES>                  Comma-separated search queries

Options:
    --limit <N>                Number of results per query (default: 1)
    --section <SECTION>        Search section: top, video, user (default: top)
    --users                    Shorthand to search users (equivalent to --section user)
    --profiles <N>             Number of profiles per query when searching users (default: 10)
    --comments <N>             Max comments per post

Examples:
    tiktok search "machine learning" --limit 10
    tiktok search "tech" --users --profiles 5 --json

### 4. Video — Scrape specific TikTok videos by URL

    tiktok video <URLS> [OPTIONS]

Arguments:
    <URLS>                     Comma-separated TikTok video URLs

Options:
    --related <N>              Scrape N related videos per URL
    --comments <N>             Max comments per post

Examples:
    tiktok video "https://www.tiktok.com/@user/video/1234567890"
    tiktok video "https://www.tiktok.com/@user/video/1234567890" --related 5 --json

## Exit Codes & Errors

Failures print YAML (or JSON) to stderr with a machine-readable `code` and matching exit status:

    0  ok             Success
    1  error          Unclassified error - report and stop
    2  network        Network failure or timeout - retry once, then stop
    3  auth_required  Invalid API key or missing permissions - do not retry
    4  not_found      Target resource not found - do not retry
    5  rate_limited   Rate limit or compute budget exceeded - back off before retrying
    6  invalid_input  Invalid arguments or parameters - fix the call
    7  no_account     No account or API key found - run tiktok login or accounts list
"#;
