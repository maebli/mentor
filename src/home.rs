//! The front page: a hero, a fuzzy search box, and the catalogue of tools
//! grouped by category. Everything here is driven by `registry::tools()`.

use crate::registry::{self, Category, ToolMeta};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use leptos::prelude::*;

fn card(meta: &'static ToolMeta) -> impl IntoView {
    view! {
        <a class="card" href=meta.href()>
            <span class="card-glyph">{meta.glyph}</span>
            <span class="card-greek">{meta.greek}</span>
            <span class="card-title">{meta.title}</span>
            <span class="card-tagline">{meta.tagline}</span>
        </a>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    let query = RwSignal::new(String::new());

    let results = move || {
        let q = query.get();
        let q = q.trim();
        if q.is_empty() {
            // Grouped, in category order.
            Category::ORDER
                .iter()
                .map(|cat| {
                    let cards = registry::tools()
                        .iter()
                        .filter(|t| t.category == *cat)
                        .map(card)
                        .collect_view();
                    view! {
                        <section class="group">
                            <h2 class="group-title">{cat.title()}</h2>
                            <div class="grid">{cards}</div>
                        </section>
                    }
                })
                .collect_view()
                .into_any()
        } else {
            // Flat, ranked by fuzzy score.
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(i64, &'static ToolMeta)> = registry::tools()
                .iter()
                .filter_map(|t| {
                    matcher
                        .fuzzy_match(&t.haystack(), q)
                        .map(|score| (score, t))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));

            if scored.is_empty() {
                view! { <p class="empty">"No tools match “" {q.to_string()} "”."</p> }
                    .into_any()
            } else {
                let cards = scored.into_iter().map(|(_, t)| card(t)).collect_view();
                view! {
                    <section class="group">
                        <div class="grid">{cards}</div>
                    </section>
                }
                .into_any()
            }
        }
    };

    view! {
        <div class="home">
            <header class="hero">
                <span class="hero-glyph">"🦉"</span>
                <h1>"Mentor"</h1>
                <p class="hero-sub">
                    "Visual tools for planning software — from the guides of Greek myth. "
                    "Fast, offline-friendly, and yours."
                </p>
                <input
                    class="search"
                    type="search"
                    placeholder="Search tools…  (try “root cause” or “cards”)"
                    autocomplete="off"
                    prop:value=move || query.get()
                    on:input=move |ev| query.set(event_target_value(&ev))
                />
            </header>
            <div class="catalogue">{results}</div>
            <footer class="home-foot">
                <span>"Built in Rust · Leptos · WebAssembly"</span>
            </footer>
        </div>
    }
}
