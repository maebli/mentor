//! Shared UI: the tool shell, the text editor pane, and small helpers.
//!
//! These are the pieces every editor tool reuses so that the "text on the left,
//! syntax-checked, live visual on the right" experience is identical everywhere.

use crate::dsl::{ParseIssue, Severity};
use crate::registry::ToolMeta;
use gloo_storage::{LocalStorage, Storage};
use leptos::prelude::*;

/// A `String` signal that loads from and saves to `localStorage` under `key`.
///
/// Used so a user's work in each tool survives a page reload. The initial value
/// is whatever was stored, falling back to `default`.
pub fn use_persisted(key: &'static str, default: &str) -> RwSignal<String> {
    let storage_key = format!("mentor:{key}");
    let initial: String =
        LocalStorage::get(&storage_key).unwrap_or_else(|_| default.to_string());
    let sig = RwSignal::new(initial);
    Effect::new(move |_| {
        let value = sig.get();
        let _ = LocalStorage::set(&storage_key, value);
    });
    sig
}

/// Header + two-pane frame shared by every editor tool. Pass the editor pane and
/// the visual pane as children via the `left` and `right` slots.
#[component]
pub fn ToolShell(
    meta: &'static ToolMeta,
    /// Link to a reference explaining the technique (e.g. Wikipedia).
    #[prop(into)]
    reference: String,
    /// The editor / left side.
    left: AnyView,
    /// The visual / right side.
    right: AnyView,
) -> impl IntoView {
    view! {
        <div class="tool">
            <header class="tool-head">
                <a class="back" href="#/">"← Mentor"</a>
                <div class="tool-title">
                    <span class="glyph">{meta.glyph}</span>
                    <h1>{meta.greek}</h1>
                    <span class="technique">{meta.title}</span>
                </div>
                <a class="reference" href=reference target="_blank" rel="noreferrer">
                    "reference ↗"
                </a>
            </header>
            <p class="tool-tagline">{meta.tagline}</p>
            <div class="split">
                <section class="pane pane-text">{left}</section>
                <section class="pane pane-visual">{right}</section>
            </div>
        </div>
    }
}

/// A monospace textarea with a live issue list underneath. `text` is the source
/// of truth for the whole tool; `issues` is derived from parsing it.
#[component]
pub fn EditorPane(
    text: RwSignal<String>,
    #[prop(into)] issues: Signal<Vec<ParseIssue>>,
    /// Short syntax reminder shown above the editor.
    #[prop(into)]
    syntax_hint: String,
) -> impl IntoView {
    view! {
        <div class="editor">
            <div class="editor-hint">{syntax_hint}</div>
            <textarea
                class="editor-area"
                spellcheck="false"
                prop:value=move || text.get()
                on:input=move |ev| text.set(event_target_value(&ev))
            ></textarea>
            <IssueList issues=issues />
        </div>
    }
}

/// Renders parse issues, or a clean "no problems" note when there are none.
#[component]
pub fn IssueList(#[prop(into)] issues: Signal<Vec<ParseIssue>>) -> impl IntoView {
    view! {
        <div class="issues">
            {move || {
                let items = issues.get();
                if items.is_empty() {
                    view! { <div class="issue ok">"✓ no problems"</div> }.into_any()
                } else {
                    items
                        .into_iter()
                        .map(|i| {
                            let cls = match i.severity {
                                Severity::Error => "issue err",
                                Severity::Warning => "issue warn",
                            };
                            let badge = match i.severity {
                                Severity::Error => "error",
                                Severity::Warning => "warn",
                            };
                            view! {
                                <div class=cls>
                                    <span class="issue-badge">{badge}</span>
                                    <span class="issue-line">"line " {i.line}</span>
                                    <span class="issue-msg">{i.message}</span>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            }}
        </div>
    }
}
