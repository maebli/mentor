//! Metis — heuristics reference page. PLACEHOLDER.
//! A read-only reference (not an editor): a searchable, visual gallery of
//! heuristics. To be implemented.

use crate::registry;
use crate::ui::ToolShell;
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Heuristic";

#[component]
pub fn MetisPage() -> impl IntoView {
    let meta = registry::find("metis").expect("registered");
    let left = view! { <div class="editor"><p class="canvas-empty">"Reference coming soon."</p></div> }.into_any();
    let right = view! { <div class="canvas"><p class="canvas-empty">"Gallery coming soon."</p></div> }.into_any();
    view! { <ToolShell meta=meta reference=REFERENCE left=left right=right /> }
}
