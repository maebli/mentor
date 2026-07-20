//! Application root and a tiny hash-based router.
//!
//! We deliberately avoid `leptos_router` here. Mentor is deployed to GitHub
//! Pages under the `/mentor/` sub-path with no server-side rewrites, where the
//! History API needs base-path juggling and a 404 fallback. Hash routing
//! (`#/agora`) sidesteps all of that: the browser never asks the server for the
//! route, so any path works from any host.

use crate::home::Home;
use crate::tools;
use leptos::prelude::*;

/// The current route slug, e.g. `""` for home or `"agora"` for a tool.
fn current_slug() -> String {
    let hash = window().location().hash().unwrap_or_default();
    hash.trim_start_matches('#')
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

#[component]
pub fn App() -> impl IntoView {
    let slug = RwSignal::new(current_slug());

    // Keep the signal in step with the address bar. The listener must outlive
    // this function, so we leak its handle for the lifetime of the page.
    let handle = window_event_listener(leptos::ev::hashchange, move |_| {
        slug.set(current_slug());
    });
    std::mem::forget(handle);

    view! {
        <div class="app">
            {move || {
                let s = slug.get();
                match s.as_str() {
                    "" => view! { <Home /> }.into_any(),
                    other => tools::route(other),
                }
            }}
        </div>
    }
}

/// Shown for an unknown slug.
#[component]
pub fn NotFound(slug: String) -> impl IntoView {
    view! {
        <div class="notfound">
            <a class="back" href="#/">"← Mentor"</a>
            <h1>"Lost the thread"</h1>
            <p>"No tool answers to " <code>{slug}</code> "."</p>
        </div>
    }
}
