//! Shared UI: the tool shell, the text editor pane, and small helpers.
//!
//! These are the pieces every editor tool reuses so that the "text on the left,
//! syntax-checked, live visual on the right" experience is identical everywhere.

use crate::dsl::{ParseIssue, Severity};
use crate::registry::ToolMeta;
use gloo_storage::{LocalStorage, Storage};
use js_sys::{Array, Function, Reflect};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

/// HTML-escape text destined for `inner_html`.
fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            _ => o.push(c),
        }
    }
    o
}

/// True for an 8D discipline token like `d1` … `d8` (also `d0`).
fn is_discipline(tok: &str) -> bool {
    let b = tok.as_bytes();
    b.len() == 2 && (b[0] == b'd' || b[0] == b'D') && b[1].is_ascii_digit() && b[1] <= b'8'
}

/// Highlight one line of a tool's text format into HTML spans: `#` comments, a
/// leading keyword (from `keywords` or a `dN` discipline), and the `:` after it.
fn highlight_line(line: &str, keywords: &[&str]) -> String {
    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);
    let mut s = esc(indent);
    if rest.is_empty() {
        return s;
    }
    if rest.starts_with('#') {
        s.push_str("<span class=\"hl-comment\">");
        s.push_str(&esc(rest));
        s.push_str("</span>");
        return s;
    }
    // The keyword token runs up to the first whitespace or colon.
    let end = rest
        .char_indices()
        .find(|(_, c)| c.is_whitespace() || *c == ':')
        .map(|(i, _)| i)
        .unwrap_or(rest.len());
    let (token, after) = rest.split_at(end);
    if !token.is_empty() && (keywords.iter().any(|k| *k == token) || is_discipline(token)) {
        s.push_str("<span class=\"hl-keyword\">");
        s.push_str(&esc(token));
        s.push_str("</span>");
    } else {
        s.push_str(&esc(token));
    }
    if let Some(value) = after.strip_prefix(':') {
        s.push_str("<span class=\"hl-punct\">:</span>");
        s.push_str(&esc(value));
    } else {
        s.push_str(&esc(after));
    }
    s
}

/// Highlight the whole text, one line at a time (line count is preserved so the
/// layer stays aligned with the textarea).
fn highlight_html(text: &str, keywords: &[&str]) -> String {
    text.split('\n')
        .map(|l| highlight_line(l, keywords))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Call `window.MentorExport.<method>(...args)` (defined in `public/export.js`).
/// Resolved at call time so load order with the wasm module doesn't matter; a
/// no-op if the helper isn't present.
fn call_export(method: &str, args: &[JsValue]) {
    let win: JsValue = match web_sys::window() {
        Some(w) => w.into(),
        None => return,
    };
    let Ok(obj) = Reflect::get(&win, &JsValue::from_str("MentorExport")) else { return };
    if obj.is_undefined() || obj.is_null() {
        return;
    }
    let Ok(func) = Reflect::get(&obj, &JsValue::from_str(method)) else { return };
    let Ok(func) = func.dyn_into::<Function>() else { return };
    let arr = Array::new();
    for a in args {
        arr.push(a);
    }
    let _ = func.apply(&obj, &arr);
}

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
    /// The tool's source text, exported by the "Copy/Save text" buttons.
    text: RwSignal<String>,
    /// The editor / left side.
    left: AnyView,
    /// The visual / right side.
    right: AnyView,
) -> impl IntoView {
    // The visual is the artifact we rasterise; `.canvas` excludes the toolbar.
    let sel = ".pane-visual .canvas";
    let png_name = format!("{}.png", meta.slug);
    let txt_name = format!("{}.txt", meta.slug);

    let copy_png = move |_| call_export("copyPng", &[JsValue::from_str(sel)]);
    let save_png = {
        let png_name = png_name.clone();
        move |_| {
            call_export(
                "downloadPng",
                &[JsValue::from_str(sel), JsValue::from_str(&png_name)],
            )
        }
    };
    let copy_txt = move |_| call_export("copyText", &[JsValue::from_str(&text.get_untracked())]);
    let save_txt = {
        let txt_name = txt_name.clone();
        move |_| {
            call_export(
                "downloadText",
                &[
                    JsValue::from_str(&text.get_untracked()),
                    JsValue::from_str(&txt_name),
                ],
            )
        }
    };

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
                <section class="pane pane-visual">
                    <div class="export-bar">
                        <button class="exp" on:click=copy_png title="Copy the diagram to the clipboard as a PNG image">"Copy PNG"</button>
                        <button class="exp" on:click=save_png title="Download the diagram as a PNG image">"Save PNG"</button>
                        <span class="exp-sep"></span>
                        <button class="exp" on:click=copy_txt title="Copy the source text to the clipboard">"Copy text"</button>
                        <button class="exp" on:click=save_txt title="Download the source text">"Save text"</button>
                    </div>
                    {right}
                </section>
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
    /// Keywords to highlight for this tool's format (e.g. `["problem", "why"]`).
    #[prop(optional)]
    keywords: &'static [&'static str],
) -> impl IntoView {
    let ta_ref = NodeRef::<leptos::html::Textarea>::new();
    let gutter_ref = NodeRef::<leptos::html::Div>::new();
    let hl_ref = NodeRef::<leptos::html::Pre>::new();
    let line_count = Memo::new(move |_| text.get().matches('\n').count() + 1);
    let highlighted = move || highlight_html(&text.get(), keywords);

    // Tab inserts two spaces (the DSL's indentation unit) instead of moving
    // focus. `set_range_text` edits the value and caret natively; the textarea is
    // uncontrolled (initial value only) so updating the signal won't reset the
    // caret.
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Tab" {
            ev.prevent_default();
            if let Some(ta) = ta_ref.get() {
                let start = ta.selection_start().ok().flatten().unwrap_or(0);
                let _ = ta.set_range_text("  ");
                let _ = ta.set_selection_range(start + 2, start + 2);
                text.set(ta.value());
            }
        }
    };
    // Keep the gutter and the highlight layer aligned with the textarea's scroll.
    let on_scroll = move |_| {
        if let Some(ta) = ta_ref.get() {
            let (top, left) = (ta.scroll_top(), ta.scroll_left());
            if let Some(g) = gutter_ref.get() {
                g.set_scroll_top(top);
            }
            if let Some(h) = hl_ref.get() {
                h.set_scroll_top(top);
                h.set_scroll_left(left);
            }
        }
    };

    view! {
        <div class="editor">
            <div class="editor-hint">{syntax_hint}</div>
            <div class="editor-main">
                <div class="gutter" node_ref=gutter_ref>
                    {move || {
                        (1..=line_count.get())
                            .map(|n| view! { <div class="ln">{n}</div> })
                            .collect_view()
                    }}
                </div>
                <div class="code">
                    // Highlighted text sits behind a transparent-text textarea.
                    <pre class="highlight" node_ref=hl_ref aria-hidden="true" inner_html=highlighted></pre>
                    <textarea
                        class="editor-area"
                        node_ref=ta_ref
                        spellcheck="false"
                        wrap="off"
                        autocomplete="off"
                        autocapitalize="off"
                        prop:value=text.get_untracked()
                        on:input=move |ev| text.set(event_target_value(&ev))
                        on:keydown=on_keydown
                        on:scroll=on_scroll
                    ></textarea>
                </div>
            </div>
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
