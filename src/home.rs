use crate::registry::{self, Category, ToolMeta};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use leptos::prelude::*;
use std::cmp::Reverse;

fn method_mark(slug: &str) -> impl IntoView {
    let path = match slug {
        "agora" => "M3 4h12v10H3z M6 8h6 M8 17v3h13V9h-3",
        "morpheus" => "M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z",
        "ariadne" => "M9 2h6v5H9z M12 7v5 M5 16v-4h14v4 M2 16h6v5H2z M16 16h6v5h-6z",
        "socrates" => "M3 3h6v4H3z M6 7v5h6 M12 10h7v4h-7z M15 14v5h6",
        "cetus" => "M2 12h20 M7 12 3 5 M12 12 8 5 M17 12l-4-7 M7 12l-4 7 M12 12l-4 7 M17 12l-4 7",
        "herakles" => "M5 3h14v18H5z M8 7h1m3 0h4 M8 12h1m3 0h4 M8 17h1m3 0h4",
        "cassandra" => "m12 3 10 17H2z M12 9v5m0 3v.2",
        "pythia" => "M3 3v18h18 M6 16l4-6 4 3 6-8",
        "themis" => "M3 4h18v16H3z M3 9h18 M9 4v16 M13 14l2 2 3-4",
        "tyche" => "M2 20h20 M3 17c4 0 4-13 9-13s5 13 9 13",
        "artemis" => "M20 12a8 8 0 1 1-8-8 M16 12a4 4 0 1 1-4-4 M12 12l9-9 M17 3h4v4",
        "metis" => "M12 6c-3-2-6-2-10-2v14c4 0 7 0 10 2 3-2 6-2 10-2V4c-4 0-7 0-10 2z M12 6v14",
        _ => "m12 2 10 7-4 12H6L2 9z M12 6l6 5-4 6-6-2-2-5z",
    };
    view! {
        <svg class="method-mark" viewBox="0 0 24 24" aria-hidden="true" fill="none"
            stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path d=path />
        </svg>
    }
}

fn card(meta: &'static ToolMeta) -> impl IntoView {
    view! {
        <a class="card" href=meta.href()>
            <span class="card-meta">
                <span class="method-icon">{method_mark(meta.slug)}</span>
                <span>{meta.greek}</span>
                <span class="card-arrow" aria-hidden="true">"↗"</span>
            </span>
            <h3>{meta.title}</h3>
            <p>{meta.tagline}</p>
            <span class="card-category">{meta.category.title()}</span>
        </a>
    }
}

fn matching_tools(query: &str, category: Option<Category>) -> Vec<&'static ToolMeta> {
    let candidates = registry::tools()
        .iter()
        .filter(|tool| category.is_none_or(|category| tool.category == category));
    let query = query.trim();
    if query.is_empty() {
        return candidates.collect();
    }
    let matcher = SkimMatcherV2::default();
    let mut matches: Vec<_> = candidates
        .filter_map(|tool| {
            let score = matcher.fuzzy_match(&tool.haystack(), query)?;
            let title_score = matcher.fuzzy_match(tool.title, query);
            Some(((title_score, score), tool))
        })
        .collect();
    matches.sort_by_key(|(rank, _)| Reverse(*rank));
    matches.into_iter().map(|(_, tool)| tool).collect()
}

#[component]
pub fn Home() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let category = RwSignal::new(None::<Category>);
    let matches = Memo::new(move |_| matching_tools(&query.get(), category.get()));
    let library = NodeRef::<leptos::html::H2>::new();
    let sync = NodeRef::<leptos::html::Div>::new();
    let browse = move |_| {
        if let Some(heading) = library.get() {
            let _ = heading.focus();
            heading.scroll_into_view();
        }
    };
    let show_sync = move |_| {
        if let Some(panel) = sync.get() {
            let _ = panel.focus();
            panel.scroll_into_view();
        }
    };

    view! {
        <main class="home">
            <header class="masthead">
                <a class="brand" href="#/" aria-label="Mentor home"><span class="brand-mark" aria-hidden="true">"M"</span>"Mentor"</a>
                <nav aria-label="Site navigation">
                    <button on:click=browse>"Tool library"</button>
                    <button on:click=show_sync>"GitHub sync"</button>
                </nav>
            </header>
            <section class="hero" aria-labelledby="hero-title">
                <div class="hero-copy">
                    <h1 id="hero-title">"A field guide to "<em>"visual planning."</em></h1>
                    <p>"Explore software designs, trace root causes, and weigh decisions with tools you can use in your browser."</p>
                    <button class="browse" on:click=browse>"Browse tools "<span aria-hidden="true">"↓"</span></button>
                </div>
                <img class="hero-art" src="athena-owl.webp" width="900" height="600" alt="" fetchpriority="high" />
            </section>
            <section class="catalogue" aria-labelledby="library-title">
                <div class="library-head">
                    <h2 id="library-title" tabindex="-1" node_ref=library>"Tool library"</h2>
                    <label class="search-box">
                        <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.6">
                            <circle cx="10" cy="10" r="6.5" /><path d="m15 15 5 5" />
                        </svg>
                        <input class="search" type="search" aria-label="Search tools" placeholder="Search tools or techniques"
                            autocomplete="off" prop:value=move || query.get()
                            on:input=move |ev| query.set(event_target_value(&ev)) />
                    </label>
                </div>
                <div class="filter-bar">
                    <div class="filters" role="group" aria-label="Filter by category">
                        {std::iter::once(None).chain(Category::ORDER.into_iter().map(Some)).map(|item| {
                            view! {
                                <button aria-pressed=move || (item == category.get()).to_string()
                                    on:click=move |_| category.set(item)>
                                    {item.map_or("All tools", Category::title)}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                    <span class="match-count" role="status">{move || matches.with(|tools| match tools.len() {
                        1 => "1 tool".to_owned(),
                        n => format!("{n} tools"),
                    })}</span>
                </div>
                {move || matches.with(|tools| {
                    if tools.is_empty() {
                        view! {
                            <div class="empty">
                                <h3>"No matching tools"</h3>
                                <button class="browse" on:click=move |_| {
                                    query.set(String::new());
                                    category.set(None);
                                }>"Show all tools"</button>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div class="grid">{tools.iter().copied().map(card).collect_view()}</div> }.into_any()
                    }
                })}
            </section>
            <div class="sync-section" node_ref=sync tabindex="-1"><crate::sync::GitHubPanel /></div>
            <footer class="home-foot"><span>"Mentor"</span><a href="https://github.com/maebli/mentor">"Source on GitHub ↗"</a></footer>
        </main>
    }
}
