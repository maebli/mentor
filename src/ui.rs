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
