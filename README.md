# TikTok CLI

[![Release](https://img.shields.io/github/v/release/SpaceCorps/TikTok-Cli?color=orange&label=version)](https://github.com/SpaceCorps/TikTok-Cli/releases/latest)
[![CI](https://github.com/SpaceCorps/TikTok-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/TikTok-Cli/actions/workflows/ci.yml)
[![Docs](https://img.shields.io/badge/docs-online-success)](https://spacecorps.github.io/TikTok-Cli/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A blazing fast, native command-line tool and agent interface for scraping TikTok videos, user profiles, hashtags, and search results via [Apify](https://apify.com). Built in Rust for developers and autonomous AI workflows.

---

## Highlights

- ⚡ **Sub-5ms Startup**: Compiled as a standalone native binary with zero runtime dependencies. Executes in ~1–3 ms with no runtime startup delay.
- 🔐 **OS Keystore Integration**: Store and manage API tokens securely in your host operating system vault (macOS Keychain, Linux Secret Service, Windows DPAPI) via `tiktok login` or `tiktok accounts add`.
- 🌐 **Complete TikTok Scraper Capabilities**: Scrape creator profiles (`profile`), hashtags (`hashtag`), top videos and user accounts (`search`), and individual video URLs with related videos (`video`).
- 🤖 **AI Agent Native**: Clean YAML default output for human and LLM terminal inspection, deterministic `--json` mode for agent loops and `jq`, and structured error envelopes on `stderr`.
- 🛡️ **Backward Compatible**: Supports direct `--api-key <key>`, `APIFY_TOKEN` / `TIKTOK_API_KEY` environment variables, and custom API base URLs via `TIKTOK_API_URL`.

---

## Installation

### Using Cargo

```bash
cargo install --git https://github.com/SpaceCorps/TikTok-Cli --locked
```

### Pre-built Standalone Binaries

Download precompiled standalone binaries directly from [GitHub Releases](https://github.com/SpaceCorps/TikTok-Cli/releases/latest):

| Platform | Architecture | Binary Package |
|:---|:---|:---|
| **macOS** | Apple Silicon (`aarch64`) | [`tiktok-v1.0.0-aarch64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/TikTok-Cli/releases/download/v1.0.0/tiktok-v1.0.0-aarch64-apple-darwin.tar.gz) |
| **macOS** | Intel (`x86_64`) | [`tiktok-v1.0.0-x86_64-apple-darwin.tar.gz`](https://github.com/SpaceCorps/TikTok-Cli/releases/download/v1.0.0/tiktok-v1.0.0-x86_64-apple-darwin.tar.gz) |
| **Linux** | x86_64 (musl static) | [`tiktok-v1.0.0-x86_64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/TikTok-Cli/releases/download/v1.0.0/tiktok-v1.0.0-x86_64-unknown-linux-musl.tar.gz) |
| **Linux** | ARM64 (musl static) | [`tiktok-v1.0.0-aarch64-unknown-linux-musl.tar.gz`](https://github.com/SpaceCorps/TikTok-Cli/releases/download/v1.0.0/tiktok-v1.0.0-aarch64-unknown-linux-musl.tar.gz) |
| **Windows**| x64 (MSVC) | [`tiktok-v1.0.0-x86_64-pc-windows-msvc.zip`](https://github.com/SpaceCorps/TikTok-Cli/releases/download/v1.0.0/tiktok-v1.0.0-x86_64-pc-windows-msvc.zip) |

---

## Quickstart

### 1. Authenticate

```bash
# Interactive login (opens Apify console in browser, prompts for token, saves to OS keystore)
tiktok login

# Or set in environment
export APIFY_TOKEN="your-apify-token"

# Or configure a named account
printf %s "$TOKEN" | tiktok accounts add production --api-key-stdin
```

### 2. Scrape Videos from a Profile

```bash
# Scrape latest 10 videos
tiktok profile nike --limit 10

# Scrape sorted by popularity and output JSON
tiktok profile https://www.tiktok.com/@nike --sort popular --json
```

### 3. Scrape Videos by Hashtag

```bash
# Scrape 5 videos for hashtag
tiktok hashtag "coding" --limit 5

# Scrape multiple hashtags with comments
tiktok hashtag "fitness,workout" --limit 20 --comments 10 --json
```

### 4. Search TikTok for Videos or Users

```bash
# Search top videos
tiktok search "machine learning" --limit 10

# Search user profiles
tiktok search "tech" --users --profiles 5 --json
```

### 5. Scrape Specific Video URLs

```bash
# Scrape video metadata and comments
tiktok video "https://www.tiktok.com/@user/video/1234567890" --comments 20

# Scrape video and 5 related videos
tiktok video "https://www.tiktok.com/@user/video/1234567890" --related 5 --json
```

---

## Command Reference

| Command | Description | Example |
|:---|:---|:---|
| `tiktok profile <USERNAMES>` | Scrape videos from user profiles with sort, date, and comment filters | `tiktok profile nike --limit 10` |
| `tiktok hashtag <HASHTAGS>` | Scrape videos by hashtag with limit and comments options | `tiktok hashtag "coding" --limit 5` |
| `tiktok search <QUERIES>` | Search TikTok for videos or users (via `--users`) | `tiktok search "tech" --users --profiles 5` |
| `tiktok video <URLS>` | Scrape specific TikTok videos by URL and related videos | `tiktok video https://www.tiktok.com/@u/video/123` |
| `tiktok login [name]` | Authenticate interactively via browser token flow | `tiktok login work` |
| `tiktok accounts add <n>` | Add a named account to local OS keystore | `tiktok accounts add work --api-key apify_xxx` |
| `tiktok accounts list` | List configured accounts (pass `--check` to verify) | `tiktok accounts list --check` |
| `tiktok accounts test <n>` | Test connectivity and inspect account identity | `tiktok accounts test work` |
| `tiktok accounts remove <n>` | Remove account and delete token from keystore | `tiktok accounts remove work --yes` |
| `tiktok agent-readme` | Print embedded operating manual for LLM agents | `tiktok agent-readme --json` |

---

## Agent Integration & Exit Codes

`tiktok` outputs structured errors to `stderr` with stable exit codes:

```json
{
  "error": "The Apify API token was rejected.",
  "code": "auth_required",
  "detail": "HTTP 401: Unauthorized",
  "remediation": "tiktok login"
}
```

| Exit Code | Code Name | Meaning |
|:---|:---|:---|
| `0` | `ok` | Success |
| `1` | `error` | Unclassified error |
| `2` | `network` | Network failure or timeout |
| `3` | `auth_required` | Invalid/missing API token |
| `4` | `not_found` | Resource not found |
| `5` | `rate_limited` | Rate limit or compute budget exceeded |
| `6` | `invalid_input` | Parameter or validation error |
| `7` | `no_account` | No account or API token configured |

---

## Credits & License

- Original .NET CLI prototype (`TikTok.Console`) by [Niels Bosma](https://github.com/nielsbosma/TikTok.Console).
- Re-architected in native Rust (Edition 2024) and maintained by [SpaceCorps](https://github.com/SpaceCorps).
- Released under the [MIT License](LICENSE).
