# AGENTS.md

Notes for whoever extends this next.

`tiktok` is a native Rust CLI for scraping TikTok videos, user profiles, hashtags, and search results via Apify,
built to be driven by an LLM agent. It replaced a .NET global tool (`TikTok.Console`) by Niels Bosma and preserves its
interface: the same commands and flags, YAML-first output, the same error envelope and exit codes, and adds secure native
OS keystores and rich multi-account isolation.

For the manual the *agent* reads, run `tiktok agent-readme` — that text lives in `src/readme.rs` and is
the tool's actual interface for its main audience. This file is for the human or AI engineer editing the source.

## Commands

```bash
cargo build --release              # target/release/tiktok
cargo test                         # unit tests + tests/cli.rs against in-process mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install to PATH
```

Use a throwaway config directory when testing so you never touch real credentials:

```bash
export TIKTOK_CONFIG_DIR=$(mktemp -d) TIKTOK_SECRET_STORE=plaintext TIKTOK_ALLOW_PLAINTEXT_STORE=1
```

| Variable | Effect |
| --- | --- |
| `TIKTOK_CONFIG_DIR` | Overrides the config and secrets storage directory |
| `TIKTOK_SECRET_STORE` | Forces a backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `TIKTOK_ALLOW_PLAINTEXT_STORE=1` | Permits the plaintext fallback where no keystore exists |
| `TIKTOK_API_URL` | Overrides the API base URL — how `tests/cli.rs` points at its mock |
| `APIFY_TOKEN` / `TIKTOK_API_KEY` | Direct fallback API token (also supported via `--api-key`) |

## Codebase Layout

```
src/
  main.rs          argument parsing, the --json pre-scan, clap errors -> invalid_input envelopes
  cli.rs           the whole command tree (clap derive); help text lives here
  commands/
    mod.rs         routing and execution
    profile.rs     profile <USERNAMES> implementation
    hashtag.rs     hashtag <HASHTAGS> implementation
    search.rs      search <QUERIES> implementation
    video.rs       video <URLS> implementation
    login.rs       interactive browser login + token verification
    accounts.rs    accounts add | list | test | remove
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode, API error hints
  error.rs         ErrorCode (= exit code) and Error {message, detail, remediation}
  output.rs        YAML by default, JSON with --json, the error envelope, the obj! macro
  account.rs       --account or direct key resolution; identity probing via users/me
  config.rs        config.yaml, atomic writes, 0600 file permissions, cross-process lock
  secrets.rs       macOS Keychain (/usr/bin/security), libsecret (secret-tool), Windows DPAPI, plaintext
  readme.rs        agent-readme text, and the API spec version this build targets
tests/cli.rs       in-process TCP mock HTTP server integration test suite
docs/              SEO landing page, Schema.org JSON-LD, agent discovery, llms.txt, trust anchors
```

## Why It Is Built This Way

1. **Blocking HTTP, No Async Runtime Bloat:**
   CLIs make 1–3 requests. An async runtime like Tokio adds binary size and cold start latency. Using blocking `ureq` with `rustls` gives startup in 1–3 ms.

2. **The Keychain Goes Through `/usr/bin/security`:**
   Items created through `/usr/bin/security` never prompt for confirmation loops on unsigned recompiled binaries.

3. **Responses Stay `serde_json::Value`:**
   Upstream APIs frequently add new properties. Parsing directly as `Value` preserves dynamic fields without dropping information or breaking schemas.

4. **Multi-Account Safety:**
   Mutating and querying API commands accept `--account <name>` (short `-a <name>`). Users can configure multiple separate accounts safely.

5. **Self-Documentation:**
   `<bin> agent-readme [--json]` allows any LLM agent to inspect the tool's complete capabilities without network roundtrips.

## Invariants

- **Exit codes match `ErrorCode` values (0..7):** Agents branch on these exit codes. Never change their numeric values.
- **Never print plain-text secrets:** Secrets live only in native OS keystores (or encrypted DPAPI blob), never in `config.yaml`.
- **`stdout` is strictly for data output:** All diagnostic messages, warnings, prompts, and error envelopes must go to `stderr`.
- **All tests in `tests/cli.rs` must remain 100% deterministic and offline:** Zero external network requests during `cargo test`.
