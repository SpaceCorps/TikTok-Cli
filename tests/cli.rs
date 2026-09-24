//! Drives the built binary against an in-process mock of the Apify API. Every test gets its
//! own config directory and the plaintext store, so nothing touches a real keystore or account.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

#[derive(Clone, Debug)]
struct Recorded {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

type Route = (&'static str, &'static str, u16, Value);

struct Mock {
    url: String,
    log: Arc<Mutex<Vec<Recorded>>>,
}

impl Mock {
    /// Routes are (method, path-without-prefix-and-query, status, body). Unmatched requests get a 404.
    fn start(routes: Vec<Route>) -> Mock {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/v2/", listener.local_addr().unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let routes = routes.clone();
                let log = log2.clone();
                std::thread::spawn(move || {
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let mut parts = line.split_whitespace();
                    let method = parts.next().unwrap_or("").to_string();
                    let raw_path = parts.next().unwrap_or("").to_string();
                    let path_without_prefix = raw_path.trim_start_matches("/v2/").to_string();
                    let path_clean = path_without_prefix.split('?').next().unwrap_or(&path_without_prefix).to_string();

                    let mut headers = Vec::new();
                    let mut len = 0usize;
                    loop {
                        let mut h = String::new();
                        reader.read_line(&mut h).unwrap();
                        let h = h.trim_end();
                        if h.is_empty() {
                            break;
                        }
                        if let Some((k, v)) = h.split_once(':') {
                            let (k, v) = (k.trim().to_lowercase(), v.trim().to_string());
                            if k == "content-length" {
                                len = v.parse().unwrap_or(0);
                            }
                            headers.push((k, v));
                        }
                    }
                    let mut buf = vec![0; len];
                    reader.read_exact(&mut buf).unwrap();
                    let body = (len > 0).then(|| serde_json::from_slice(&buf).unwrap());
                    log.lock().unwrap().push(Recorded {
                        method: method.clone(),
                        path: path_without_prefix,
                        headers,
                        body,
                    });

                    let (status, resp) = routes
                        .iter()
                        .find(|(m, p, _, _)| *m == method && *p == path_clean)
                        .map(|(_, _, s, b)| (*s, b.clone()))
                        .unwrap_or((404, json!({"code": "not_found", "error": "no route"})));
                    let text = if status == 204 { String::new() } else { resp.to_string() };
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{text}",
                        text.len()
                    );
                });
            }
        });
        Mock { url, log }
    }

    fn requests(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    fn last(&self, method: &str) -> Recorded {
        self.requests().into_iter().rev().find(|r| r.method == method).expect("no such request")
    }
}

struct Env {
    dir: PathBuf,
    api: String,
}

impl Env {
    fn new(mock: &Mock) -> Env {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tiktok-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Env { dir, api: mock.url.clone() }
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tiktok"))
            .args(args)
            .env("TIKTOK_CONFIG_DIR", &self.dir)
            .env("TIKTOK_SECRET_STORE", "plaintext")
            .env("TIKTOK_ALLOW_PLAINTEXT_STORE", "1")
            .env("TIKTOK_API_URL", &self.api)
            .env_remove("APIFY_TOKEN")
            .env_remove("TIKTOK_API_KEY")
            .output()
            .unwrap()
    }

    fn run_with_env(&self, args: &[&str], key: &str, val: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_tiktok"))
            .args(args)
            .env("TIKTOK_CONFIG_DIR", &self.dir)
            .env("TIKTOK_SECRET_STORE", "plaintext")
            .env("TIKTOK_ALLOW_PLAINTEXT_STORE", "1")
            .env("TIKTOK_API_URL", &self.api)
            .env(key, val)
            .output()
            .unwrap()
    }

    fn json(&self, args: &[&str]) -> (i32, Value, Value) {
        let mut all = args.to_vec();
        all.push("--json");
        let out = self.run(&all);
        let parse = |b: &[u8]| {
            let s = String::from_utf8_lossy(b);
            let s = s.lines().filter(|l| !l.starts_with("warning:")).collect::<Vec<_>>().join("\n");
            serde_json::from_str(&s).unwrap_or(Value::Null)
        };
        (out.status.code().unwrap(), parse(&out.stdout), parse(&out.stderr))
    }

    fn with_account(self) -> Env {
        let (code, out, err) = self.json(&["accounts", "add", "work", "--api-key", "apify_test_token"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(out["status"], "added");
        self
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// =============================================================================================
// Tests
// =============================================================================================

#[test]
fn agent_readme_prints_json() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);
    let (code, out, err) = env.json(&["agent-readme"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["tool"], "tiktok");
    assert_eq!(out["apiVersion"], "2.0.0");
    assert!(out["rules"].is_array());
    assert!(out["exitCodes"]["0"].is_string());
}

#[test]
fn agent_readme_prints_markdown() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);
    let out = env.run(&["agent-readme"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("# tiktok - agent operating manual"));
    assert!(stdout.contains("profile"));
    assert!(stdout.contains("hashtag"));
    assert!(stdout.contains("search"));
    assert!(stdout.contains("video"));
}

#[test]
fn no_account_error() {
    let mock = Mock::start(vec![]);
    let env = Env::new(&mock);
    let (code, _, err) = env.json(&["profile", "nike"]);
    assert_eq!(code, 7); // ErrorCode::NoAccount
    assert_eq!(err["code"], "no_account");
}

#[test]
fn accounts_add_list_test_remove() {
    let mock = Mock::start(vec![(
        "GET",
        "users/me",
        200,
        json!({"data": {"username": "spacecorps", "email": "space@example.com"}}),
    )]);
    let env = Env::new(&mock);

    // 1. Add
    let (code, out, err) = env.json(&["accounts", "add", "personal", "--api-key", "token_pers"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out["status"], "added");
    assert_eq!(out["name"], "personal");
    assert_eq!(out["identity"], "spacecorps (space@example.com)");

    // 2. List
    let (code, list, err) = env.json(&["accounts", "list"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "personal");

    // 3. Test
    let (code, test_res, err) = env.json(&["accounts", "test", "personal"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(test_res["status"], "valid");

    // 4. Remove
    let (code, rm_res, err) = env.json(&["accounts", "remove", "personal", "--yes"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(rm_res["status"], "removed");

    // 5. List is now empty
    let (_, list2, _) = env.json(&["accounts", "list"]);
    assert_eq!(list2.as_array().unwrap().len(), 0);
}

#[test]
fn profile_command() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"username": "tester"}})),
        (
            "POST",
            "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": "7123456789012345678",
                    "text": "Check out this video #coding",
                    "authorMeta": {"name": "nike", "nickName": "Nike"}
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "profile",
        "nike,adidas",
        "-a",
        "work",
        "--limit",
        "5",
        "--sort",
        "popular",
        "--sections",
        "videos,reposts",
        "--exclude-pinned",
        "--comments",
        "10",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());
    assert_eq!(out[0]["id"], "7123456789012345678");

    let req = mock.last("POST");
    assert!(req.path.starts_with("acts/clockworks~tiktok-scraper/run-sync-get-dataset-items"));
    let body = req.body.unwrap();
    assert_eq!(body["profiles"], json!(["nike", "adidas"]));
    assert_eq!(body["resultsPerPage"], 5);
    assert_eq!(body["profileSorting"], "popular");
    assert_eq!(body["profileScrapeSections"], json!(["videos", "reposts"]));
    assert_eq!(body["excludePinnedPosts"], true);
    assert_eq!(body["commentsPerPost"], 10);
}

#[test]
fn hashtag_command() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"username": "tester"}})),
        (
            "POST",
            "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": "7987654321098765432",
                    "text": "Rust is amazing #rustlang",
                    "hashtags": [{"name": "rustlang"}]
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&["hashtag", "#rustlang,#coding", "-a", "work", "--limit", "3", "--comments", "2"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());

    let req = mock.last("POST");
    let body = req.body.unwrap();
    assert_eq!(body["hashtags"], json!(["rustlang", "coding"]));
    assert_eq!(body["resultsPerPage"], 3);
    assert_eq!(body["commentsPerPost"], 2);
}

#[test]
fn search_command_top_and_users() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"username": "tester"}})),
        (
            "POST",
            "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": "111222333",
                    "text": "Machine learning overview"
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    // 1. Search videos
    let (code, out, err) =
        env.json(&["search", "machine learning", "-a", "work", "--limit", "10", "--section", "video"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());

    let req = mock.last("POST");
    let body = req.body.unwrap();
    assert_eq!(body["searchQueries"], json!(["machine learning"]));
    assert_eq!(body["resultsPerPage"], 10);
    assert_eq!(body["searchSection"], "/video");

    // 2. Search users with --users shorthand
    let (code, _, err) = env.json(&["search", "tech", "-a", "work", "--users", "--profiles", "5"]);
    assert_eq!(code, 0, "{err}");
    let req2 = mock.last("POST");
    let body2 = req2.body.unwrap();
    assert_eq!(body2["searchQueries"], json!(["tech"]));
    assert_eq!(body2["searchSection"], "/user");
    assert_eq!(body2["maxProfilesPerQuery"], 5);
}

#[test]
fn video_command() {
    let mock = Mock::start(vec![
        ("GET", "users/me", 200, json!({"data": {"username": "tester"}})),
        (
            "POST",
            "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
            200,
            json!([
                {
                    "id": "1234567890",
                    "webVideoUrl": "https://www.tiktok.com/@user/video/1234567890"
                }
            ]),
        ),
    ]);
    let env = Env::new(&mock).with_account();

    let (code, out, err) = env.json(&[
        "video",
        "https://www.tiktok.com/@user/video/1234567890",
        "-a",
        "work",
        "--related",
        "3",
        "--comments",
        "5",
    ]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());

    let req = mock.last("POST");
    let body = req.body.unwrap();
    assert_eq!(body["postURLs"], json!(["https://www.tiktok.com/@user/video/1234567890"]));
    assert_eq!(body["scrapeRelatedVideos"], true);
    assert_eq!(body["resultsPerPage"], 3);
    assert_eq!(body["commentsPerPost"], 5);
}

#[test]
fn direct_api_key_flag_and_env() {
    let mock = Mock::start(vec![("POST", "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items", 200, json!([]))]);
    let env = Env::new(&mock);

    // 1. Direct flag
    let (code, out, err) = env.json(&["profile", "nike", "--api-key", "token_flag"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.is_array());
    let req = mock.last("POST");
    assert!(req.headers.iter().any(|(k, v)| k == "authorization" && v == "Bearer token_flag"));

    // 2. Direct env var
    let args = vec!["profile", "nike", "--json"];
    let out = env.run_with_env(&args, "APIFY_TOKEN", "token_env");
    assert!(out.status.success());
    let req2 = mock.last("POST");
    assert!(req2.headers.iter().any(|(k, v)| k == "authorization" && v == "Bearer token_env"));
}

#[test]
fn error_code_401() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
        401,
        json!({"error": {"message": "Invalid token"}}),
    )]);
    let env = Env::new(&mock);
    let (code, _, err) = env.json(&["profile", "nike", "--api-key", "bad_token"]);
    assert_eq!(code, 3);
    assert_eq!(err["code"], "auth_required");
}

#[test]
fn error_code_404() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
        404,
        json!({"error": "Not found"}),
    )]);
    let env = Env::new(&mock);
    let (code, _, err) = env.json(&["profile", "nike", "--api-key", "key"]);
    assert_eq!(code, 4);
    assert_eq!(err["code"], "not_found");
}

#[test]
fn error_code_429() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
        429,
        json!({"error": "Rate limit exceeded"}),
    )]);
    let env = Env::new(&mock);
    let (code, _, err) = env.json(&["profile", "nike", "--api-key", "key"]);
    assert_eq!(code, 5);
    assert_eq!(err["code"], "rate_limited");
}

#[test]
fn error_code_400() {
    let mock = Mock::start(vec![(
        "POST",
        "acts/clockworks~tiktok-scraper/run-sync-get-dataset-items",
        400,
        json!({"error": "Bad request"}),
    )]);
    let env = Env::new(&mock);
    let (code, _, err) = env.json(&["profile", "nike", "--api-key", "key"]);
    assert_eq!(code, 6);
    assert_eq!(err["code"], "invalid_input");
}
