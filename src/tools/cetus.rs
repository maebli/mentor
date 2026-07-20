//! Cetus — Ishikawa (fishbone) diagram. PLACEHOLDER.
//! To be implemented following the pattern in `agora.rs`.

use crate::registry;
use crate::ui::ToolShell;
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Ishikawa_diagram";

#[component]
pub fn CetusTool() -> impl IntoView {
    let meta = registry::find("cetus").expect("registered");
    let left = view! { <div class="editor"><p class="canvas-empty">"Editor coming soon."</p></div> }.into_any();
    let right = view! { <div class="canvas"><p class="canvas-empty">"Visual coming soon."</p></div> }.into_any();
    view! { <ToolShell meta=meta reference=REFERENCE left=left right=right /> }
}
