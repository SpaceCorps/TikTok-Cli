//! Command line interface definitions and clap hierarchy.

use clap::{Args, Parser, Subcommand};

use crate::account::AuthArgs;

#[derive(Parser, Debug)]
#[command(
    name = "tiktok",
    version,
    about = "CLI for scraping TikTok videos, profiles, hashtags, and search results via Apify",
    long_about = "A high-performance CLI for scraping TikTok videos, user profiles, hashtags, and search results via Apify. Designed for terminal users and LLM agents with clean YAML and JSON output."
)]
pub struct Cli {
    /// Output raw JSON instead of YAML
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Scrape videos from TikTok user profiles
    Profile(ProfileArgs),

    /// Scrape videos by TikTok hashtag
    Hashtag(HashtagArgs),

    /// Search TikTok for videos or users
    Search(SearchArgs),

    /// Scrape specific TikTok videos by URL
    Video(VideoArgs),

    /// Manage stored API credentials and multi-account configurations
    #[command(subcommand)]
    Accounts(AccountsCommand),

    /// Authenticate with Apify/TikTok and store credentials in the OS keystore
    Login(LoginArgs),

    /// Output agent operating manual and machine-readable instructions
    AgentReadme,
}

#[derive(Args, Debug)]
pub struct ProfileArgs {
    /// Comma-separated TikTok usernames or URLs (e.g. nike,adidas or https://www.tiktok.com/@username)
    #[arg(value_name = "USERNAMES")]
    pub usernames: String,

    /// Number of videos per profile
    #[arg(long, default_value = "1", value_name = "N")]
    pub limit: usize,

    /// Sort order: latest, popular, oldest
    #[arg(long, default_value = "latest", value_name = "ORDER")]
    pub sort: String,

    /// Only videos published after this date (e.g. 2025-01-01)
    #[arg(long, value_name = "DATE")]
    pub since: Option<String>,

    /// Only videos published before this date
    #[arg(long, value_name = "DATE")]
    pub until: Option<String>,

    /// Sections to scrape: videos, reposts
    #[arg(long, default_value = "videos", value_name = "SECTIONS")]
    pub sections: String,

    /// Exclude pinned posts
    #[arg(long)]
    pub exclude_pinned: bool,

    /// Max comments per post
    #[arg(long, value_name = "N")]
    pub comments: Option<usize>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

#[derive(Args, Debug)]
pub struct HashtagArgs {
    /// Comma-separated hashtags (e.g. fitness,workout or #coding)
    #[arg(value_name = "HASHTAGS")]
    pub hashtags: String,

    /// Number of videos per hashtag
    #[arg(long, default_value = "1", value_name = "N")]
    pub limit: usize,

    /// Max comments per post
    #[arg(long, value_name = "N")]
    pub comments: Option<usize>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Comma-separated search queries
    #[arg(value_name = "QUERIES")]
    pub queries: String,

    /// Number of results per query
    #[arg(long, default_value = "1", value_name = "N")]
    pub limit: usize,

    /// Search section: top, video, user
    #[arg(long, default_value = "top", value_name = "SECTION")]
    pub section: String,

    /// Shorthand to search users (equivalent to --section user)
    #[arg(long)]
    pub users: bool,

    /// Number of profiles per query when searching users
    #[arg(long, value_name = "N")]
    pub profiles: Option<usize>,

    /// Max comments per post
    #[arg(long, value_name = "N")]
    pub comments: Option<usize>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

#[derive(Args, Debug)]
pub struct VideoArgs {
    /// Comma-separated TikTok video URLs
    #[arg(value_name = "URLS")]
    pub urls: String,

    /// Scrape N related videos per URL
    #[arg(long, value_name = "N")]
    pub related: Option<usize>,

    /// Max comments per post
    #[arg(long, value_name = "N")]
    pub comments: Option<usize>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

#[derive(Subcommand, Debug)]
pub enum AccountsCommand {
    /// Add an account and store its API token in the OS keystore
    Add {
        /// Account name
        name: String,

        /// Apify API token (prompts securely if omitted)
        #[arg(long, value_name = "KEY")]
        api_key: Option<String>,

        /// Read API token from stdin
        #[arg(long)]
        api_key_stdin: bool,

        /// Overwrite account if it already exists
        #[arg(long)]
        force: bool,

        /// Skip verifying the token against the API
        #[arg(long)]
        no_verify: bool,
    },

    /// List configured accounts
    List {
        /// Probe each stored key against the API to verify validity
        #[arg(long)]
        check: bool,
    },

    /// Test stored credentials for an account
    Test {
        /// Account name
        name: String,
    },

    /// Remove an account and delete its API token from the keystore
    Remove {
        /// Account name
        name: String,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
}

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Account name to store token under (defaults to 'default')
    #[arg(default_value = "default")]
    pub name: String,

    /// Apify API token
    #[arg(long, value_name = "KEY")]
    pub api_key: Option<String>,

    /// Read API token from stdin
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not attempt to open browser
    #[arg(long)]
    pub no_browser: bool,

    /// Overwrite account if it already exists
    #[arg(long)]
    pub force: bool,

    /// Skip verifying the token against the API
    #[arg(long)]
    pub no_verify: bool,
}
