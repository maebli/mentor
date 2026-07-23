//! Tool registry -> component wiring.
//!
//! `route` maps a URL slug to the tool's root component. To add a tool: create
//! its module, add a `mod` line, and add one match arm here. The metadata for
//! the home page and search lives separately in `crate::registry`.

mod agora;
mod ariadne;
mod cassandra;
mod cetus;
mod herakles;
mod metis;
mod morpheus;
mod pythia;
mod socrates;
mod themis;
mod tyche;

use crate::app::NotFound;
use leptos::prelude::*;

pub fn route(slug: &str) -> AnyView {
    match slug {
        "agora" => view! { <agora::AgoraTool /> }.into_any(),
        "morpheus" => view! { <morpheus::MorpheusTool /> }.into_any(),
        "ariadne" => view! { <ariadne::AriadneTool /> }.into_any(),
        "socrates" => view! { <socrates::SocratesTool /> }.into_any(),
        "cetus" => view! { <cetus::CetusTool /> }.into_any(),
        "herakles" => view! { <herakles::HeraklesTool /> }.into_any(),
        "cassandra" => view! { <cassandra::CassandraTool /> }.into_any(),
        "pythia" => view! { <pythia::PythiaTool /> }.into_any(),
        "themis" => view! { <themis::ThemisTool /> }.into_any(),
        "tyche" => view! { <tyche::TycheTool /> }.into_any(),
        "metis" => view! { <metis::MetisPage /> }.into_any(),
        other => view! { <NotFound slug=other.to_string() /> }.into_any(),
    }
}
