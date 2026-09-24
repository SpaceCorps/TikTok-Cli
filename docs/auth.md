---
title: "Authentication Guide"
description: "Authentication methods, OS keystore credential storage, and error handling for developers and AI agents using the TikTok CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for TikTok CLI

This document outlines authentication methods, credential storage, and error handling for developers and autonomous AI agents using the TikTok CLI.

## Overview
The TikTok CLI communicates with the Apify v2 API (`https://api.apify.com/v2/`). Authentication uses Apify API tokens generated in the Apify Console. Tokens can be stored in the host operating system's native keystore, passed as command-line flags, or supplied via environment variables.

## Prerequisites
- An Apify account ([apify.com](https://apify.com))
- An API token generated from the integrations page (`https://console.apify.com/account/integrations`)
- TikTok CLI installed on your machine (`cargo install --git https://github.com/SpaceCorps/TikTok-Cli --locked`)

## Authentication Methods

### 1. Interactive Login (`tiktok login`)
The recommended approach for interactive developer workstations:
```bash
tiktok login [account_name]
```
1. The CLI launches your default web browser directly to `https://console.apify.com/account/integrations`.
2. Copy your Apify API token (`apify_api_...`).
3. Paste the token into the terminal prompt (masked without terminal echo).
4. The CLI validates the token by calling the Apify `users/me` endpoint.
5. The API token is securely encrypted in the OS keystore (macOS Keychain, Windows DPAPI, or Linux Secret Service) under the account name (defaults to `default`).

### 2. Multi-Account Keystore (`tiktok accounts add`)
Add named accounts for distinct environments, projects, or client organizations:
```bash
# Add with interactive secure prompt
tiktok accounts add marketing

# Pipe token from standard input (prevents shell history leakage)
printf %s "$APIFY_TOKEN" | tiktok accounts add marketing --api-key-stdin

# Replace existing account credentials
tiktok accounts add marketing --api-key "$NEW_TOKEN" --force
```

### 3. Environment Variable (`APIFY_TOKEN` or `TIKTOK_API_KEY`)
For CI/CD pipelines, containerized agents, and headless environments where no keystore daemon is available:
```bash
export APIFY_TOKEN="your-apify-token"
tiktok profile nike
```

### 4. Direct Command Flag (`--api-key`)
Pass the token per invocation:
```bash
tiktok profile nike --api-key "$APIFY_TOKEN"
```

## Account Inspection & Verification
List and test stored accounts:
```bash
tiktok accounts list              # lists stored accounts and secret store type
tiktok accounts list --check      # tests API connectivity for each account
tiktok accounts test <name>       # checks validity and reports username/email
tiktok accounts remove <name> -y  # removes credentials from local keystore
```

## Custom API Base URL
To point the CLI at a proxy or mock server, configure `TIKTOK_API_URL` or `APIFY_API_URL`:
```bash
export TIKTOK_API_URL="http://localhost:8080/v2/"
tiktok profile nike
```

## Error Codes
When authentication fails, commands exit with standard machine-readable exit codes and error envelopes:
- `auth_required` (exit code 3): API token invalid, revoked, or insufficient plan permissions.
- `no_account` (exit code 7): No account or token found.
- `rate_limited` (exit code 5): Apify compute unit limit or rate limit exceeded.
