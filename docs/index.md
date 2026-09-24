---
title: "TikTok CLI"
description: "A blazing fast native command-line tool and agent interface for scraping TikTok videos, user profiles, hashtags, and search results via Apify. Built in Rust for developers and autonomous AI agents."
author: "SpaceCorps"
date: "2026-09-24"
canonical: "https://spacecorps.github.io/TikTok-Cli/index.md"
---

# TikTok CLI

A blazing fast native command-line tool and agent interface for scraping TikTok videos, user profiles, hashtags, and search results via Apify. Built in Rust for developers and autonomous AI agents.

## Quickstart

```bash
# Authenticate interactively via browser token flow
tiktok login

# Or pass your API token directly via environment variable
export APIFY_TOKEN=your-apify-token

# Scrape videos from a profile
tiktok profile nike --limit 10

# Scrape videos by hashtag
tiktok hashtag "coding" --limit 5

# Search TikTok for videos
tiktok search "machine learning" --limit 10

# Search TikTok for user accounts
tiktok search "tech" --users --profiles 5

# Scrape a specific video by URL
tiktok video "https://www.tiktok.com/@user/video/1234567890"
```

## Features

- **Blazing Fast Native Rust**: Sub-5ms startup times with zero runtime dependencies.
- **AI Agent Native**: Clean YAML default output, structured JSON (`--json`) mode, and standardized machine-readable error envelopes on `stderr`.
- **Secure Keystore Integration**: Secrets stored in native macOS Keychain, Windows DPAPI, or Linux Secret Service with zero plain-text leaks.
- **Multi-Account Workspaces**: Isolate testing, production, and client accounts safely.
- **Complete TikTok Coverage**: `profile`, `hashtag`, `search` (videos and users), and `video` metadata and related items.

## When to Use This CLI

Use the `tiktok` CLI whenever you need to:
- Retrieve video transcripts, statistics, music details, and comments from TikTok.
- Scrape creator portfolios and analyze user upload history and trends.
- Track hashtag engagement, challenges, and viral content.
- Automate market research, brand sentiment, and social listening pipelines in agent tool loops.

## Documentation Links

- [llms.txt](https://spacecorps.github.io/TikTok-Cli/llms.txt)
- [Full Agent Manual](https://spacecorps.github.io/TikTok-Cli/llms-full.txt)
- [Pricing](https://spacecorps.github.io/TikTok-Cli/pricing.md)
- [Authentication Guide](https://spacecorps.github.io/TikTok-Cli/auth.md)
- [About](https://spacecorps.github.io/TikTok-Cli/about.html)
- [Contact & Support](https://spacecorps.github.io/TikTok-Cli/contact.html)
- [Privacy Policy](https://spacecorps.github.io/TikTok-Cli/privacy.html)
- [GitHub Repository](https://github.com/SpaceCorps/TikTok-Cli)
