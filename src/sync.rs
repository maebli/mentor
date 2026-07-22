//! Optional GitHub sync: commit every tool's text to a repo of the user's own.
//!
//! Auth goes through the Cloudflare Worker broker (see `worker/`): a popup to
//! `<worker>/auth` runs the GitHub OAuth flow and `postMessage`s a user token
//! back. Committing then happens straight from the browser against
//! `api.github.com` (which allows CORS), assembling one commit via the Git Data
//! API so all tools land in a single commit.

use crate::registry;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use js_sys::Reflect;
use leptos::prelude::*;
use serde_json::{json, Value};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

const WORKER_BASE: &str = "https://mentor-github-oauth.maebli.workers.dev";
const WORKER_ORIGIN: &str = "https://mentor-github-oauth.maebli.workers.dev";
const TOKEN_KEY: &str = "mentor:gh_token";
const REPO_KEY: &str = "mentor:gh_repo";
const API: &str = "https://api.github.com";

pub fn load_token() -> Option<String> {
    LocalStorage::get::<String>(TOKEN_KEY).ok().filter(|t| !t.is_empty())
}
fn store_token(token: &str) {
    let _ = LocalStorage::set(TOKEN_KEY, token.to_string());
}
pub fn clear_token() {
    LocalStorage::delete(TOKEN_KEY);
}
pub fn load_repo() -> String {
    LocalStorage::get::<String>(REPO_KEY).unwrap_or_default()
}
pub fn store_repo(repo: &str) {
    let _ = LocalStorage::set(REPO_KEY, repo.to_string());
}

/// Open the OAuth popup. The Worker will `postMessage` the token back, caught by
/// the `message` listener installed in the panel component.
pub fn connect() {
    let url = format!("{WORKER_BASE}/auth");
    let _ = window().open_with_url_and_target_and_features(
        &url,
        "mentor-github-oauth",
        "popup,width=720,height=820",
    );
}

/// Pull a token out of a `postMessage` payload from our broker, or None.
pub fn token_from_message(origin: &str, data: &JsValue) -> Option<String> {
    if origin != WORKER_ORIGIN {
        return None;
    }
    let source = Reflect::get(data, &JsValue::from_str("source")).ok()?;
    if source.as_string()? != "mentor-github-oauth" {
        return None;
    }
    let payload = Reflect::get(data, &JsValue::from_str("payload")).ok()?;
    let token = Reflect::get(&payload, &JsValue::from_str("token")).ok()?;
    let token = token.as_string()?;
    store_token(&token);
    Some(token)
}

/// Collect the files to commit: each editor tool's text plus the CRC board JSON.
fn gather_files() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for tool in registry::tools() {
        if let Ok(text) = LocalStorage::get::<String>(format!("mentor:{}", tool.slug)) {
            if !text.trim().is_empty() {
                files.push((format!("mentor/{}.txt", tool.slug), text));
            }
        }
    }
    // The CRC board stores raw JSON under its own key (not double-encoded).
    if let Ok(board) = LocalStorage::get::<Value>("crc-board") {
        if let Ok(pretty) = serde_json::to_string_pretty(&board) {
            files.push(("mentor/agora.board.json".to_string(), pretty));
        }
    }
    files
}

async fn get_json(token: &str, url: &str) -> Result<Value, String> {
    let resp = Request::get(url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("{} on GET {}", resp.status(), short_path(url)));
    }
    resp.json::<Value>().await.map_err(|e| e.to_string())
}

async fn send_json(method: &str, token: &str, url: &str, body: Value) -> Result<Value, String> {
    let builder = match method {
        "POST" => Request::post(url),
        "PATCH" => Request::patch(url),
        _ => return Err("bad method".into()),
    };
    let resp = builder
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        let status = resp.status();
        let detail = resp.text().await.unwrap_or_default();
        return Err(format!("{} on {} {} — {}", status, method, short_path(url), truncate(&detail, 160)));
    }
    resp.json::<Value>().await.map_err(|e| e.to_string())
}

fn short_path(url: &str) -> String {
    url.strip_prefix(API).unwrap_or(url).to_string()
}
fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n { s.to_string() } else { format!("{}…", &s[..n]) }
}

/// Commit every gathered file to `owner/repo` as a single Git Data API commit on
/// the default branch. Returns the new commit's web URL.
pub async fn commit_all(token: &str, owner: &str, repo: &str) -> Result<String, String> {
    let files = gather_files();
    if files.is_empty() {
        return Err("nothing to commit — open a tool and enter some text first".into());
    }

    // Default branch + its latest commit.
    let repo_info = get_json(token, &format!("{API}/repos/{owner}/{repo}")).await?;
    let branch = repo_info["default_branch"].as_str().unwrap_or("main").to_string();

    let ref_info = get_json(token, &format!("{API}/repos/{owner}/{repo}/git/ref/heads/{branch}"))
        .await
        .map_err(|e| format!("{e}. (Does the repo have at least one commit, e.g. a README?)"))?;
    let head_sha = ref_info["object"]["sha"]
        .as_str()
        .ok_or("could not read branch head")?
        .to_string();

    let head_commit = get_json(token, &format!("{API}/repos/{owner}/{repo}/git/commits/{head_sha}")).await?;
    let base_tree = head_commit["tree"]["sha"].as_str().ok_or("no base tree")?.to_string();

    // One tree carrying every file inline (blobs created implicitly).
    let tree: Vec<Value> = files
        .iter()
        .map(|(path, content)| {
            json!({ "path": path, "mode": "100644", "type": "blob", "content": content })
        })
        .collect();
    let new_tree = send_json(
        "POST",
        token,
        &format!("{API}/repos/{owner}/{repo}/git/trees"),
        json!({ "base_tree": base_tree, "tree": tree }),
    )
    .await?;
    let tree_sha = new_tree["sha"].as_str().ok_or("no tree sha")?.to_string();

    let message = format!("Update Mentor tools ({} files)", files.len());
    let commit = send_json(
        "POST",
        token,
        &format!("{API}/repos/{owner}/{repo}/git/commits"),
        json!({ "message": message, "tree": tree_sha, "parents": [head_sha] }),
    )
    .await?;
    let commit_sha = commit["sha"].as_str().ok_or("no commit sha")?.to_string();

    send_json(
        "PATCH",
        token,
        &format!("{API}/repos/{owner}/{repo}/git/refs/heads/{branch}"),
        json!({ "sha": commit_sha, "force": false }),
    )
    .await?;

    Ok(commit["html_url"].as_str().unwrap_or("").to_string())
}

/// GitHub login name for the current token (to show "connected as …").
pub async fn whoami(token: &str) -> Result<String, String> {
    let user = get_json(token, &format!("{API}/user")).await?;
    Ok(user["login"].as_str().unwrap_or("").to_string())
}

/// Spawn an async task from an event handler.
pub fn run<F>(fut: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    spawn_local(fut);
}

/// The "Sync to GitHub" panel shown on the home page.
#[component]
pub fn GitHubPanel() -> impl IntoView {
    let token = RwSignal::new(load_token());
    let user = RwSignal::new(None::<String>);
    let repo = RwSignal::new(load_repo());
    let status = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    // Catch the token the OAuth popup posts back.
    let handle = window_event_listener(leptos::ev::message, move |ev: web_sys::MessageEvent| {
        if let Some(tok) = token_from_message(&ev.origin(), &ev.data()) {
            token.set(Some(tok));
        }
    });
    std::mem::forget(handle);

    // Resolve the login once connected.
    Effect::new(move |_| {
        if let Some(tok) = token.get() {
            if user.get_untracked().is_none() {
                run(async move {
                    if let Ok(login) = whoami(&tok).await {
                        user.set(Some(login));
                    }
                });
            }
        }
    });

    let on_commit = move |_| {
        let Some(tok) = token.get_untracked() else { return };
        let r = repo.get_untracked();
        store_repo(&r);
        let Some((owner, name)) = r.trim().split_once('/') else {
            status.set("Enter the repo as owner/name (e.g. maebli/mentor-notes).".into());
            return;
        };
        let (owner, name) = (owner.trim().to_string(), name.trim().to_string());
        if owner.is_empty() || name.is_empty() {
            status.set("Enter the repo as owner/name.".into());
            return;
        }
        busy.set(true);
        status.set("Committing…".into());
        run(async move {
            match commit_all(&tok, &owner, &name).await {
                Ok(url) => status.set(format!("Committed ✓  {url}")),
                Err(e) => status.set(format!("✗ {e}")),
            }
            busy.set(false);
        });
    };

    let disconnect = move |_| {
        clear_token();
        token.set(None);
        user.set(None);
        status.set(String::new());
    };

    view! {
        <section class="gh-panel">
            <div class="gh-head">
                <span class="gh-title">"⑃ Sync to GitHub"</span>
                {move || {
                    token
                        .get()
                        .is_some()
                        .then(|| view! { <button class="gh-link" on:click=disconnect>"disconnect"</button> })
                }}
            </div>
            {move || match token.get() {
                None => view! {
                    <div class="gh-body">
                        <p class="gh-note">
                            "Commit your tools' text to a repo of your own. Your GitHub token stays in this browser."
                        </p>
                        <button class="gh-btn" on:click=move |_| connect()>"Connect GitHub"</button>
                    </div>
                }
                    .into_any(),
                Some(_) => view! {
                    <div class="gh-body">
                        <p class="gh-note">
                            "Connected"
                            {move || user.get().map(|u| format!(" as {u}")).unwrap_or_default()}
                            ". Pick a repo you've installed the app on, then commit."
                        </p>
                        <div class="gh-row">
                            <input
                                class="gh-input"
                                placeholder="owner/repo  (e.g. maebli/mentor-notes)"
                                autocapitalize="off"
                                autocomplete="off"
                                spellcheck="false"
                                prop:value=move || repo.get()
                                on:input=move |ev| repo.set(event_target_value(&ev))
                            />
                            <button class="gh-btn" prop:disabled=move || busy.get() on:click=on_commit>
                                "Commit all"
                            </button>
                        </div>
                        <a
                            class="gh-link"
                            href="https://github.com/settings/installations"
                            target="_blank"
                            rel="noreferrer"
                        >
                            "manage which repos the app can write ↗"
                        </a>
                        {move || {
                            let s = status.get();
                            (!s.is_empty()).then(|| view! { <p class="gh-status">{s}</p> })
                        }}
                    </div>
                }
                    .into_any(),
            }}
        </section>
    }
}
