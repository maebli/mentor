//! Tool registry -> component wiring.
//!
//! `route` maps a URL slug to the tool's root component. To add a tool: create
//! its module, add a `mod` line, and add one match arm here. The metadata for
//! the home page and search lives separately in `crate::registry`.

mod agora;
mod ariadne;
mod artemis;
mod atlas;
mod cassandra;
mod cetus;
mod chronos;
mod danaus;
mod demeter;
mod echo;
mod hephaestus;
mod herakles;
mod hermes;
mod hestia;
mod iris;
mod metis;
mod morpheus;
mod psyche;
mod pythia;
mod socrates;
mod themis;
mod tyche;

use crate::app::NotFound;
use leptos::prelude::*;

fn source_program<Model: Clone + PartialEq + Send + Sync + 'static>(
    slug: &'static str,
    sample: &'static str,
    hint: &'static str,
    method_reference: &'static str,
    keywords: &'static [&'static str],
    parse: fn(&str) -> Result<Model, Vec<crate::dsl::ParseIssue>>,
    render: fn(Model, RwSignal<String>) -> AnyView,
) -> impl IntoView {
    use crate::ui::{EditorPane, ToolShell};
    let meta = crate::registry::find(slug).expect("registered tool");
    let plans = crate::plans::Plans::open(slug, sample);
    let text = plans.text;
    let model = Memo::new(move |_| text.with(|source| parse(source)));
    let issues = Signal::derive(move || {
        model.with(|result| result.as_ref().err().cloned().unwrap_or_default())
    });
    let left = view! { <EditorPane text=text issues=issues syntax_hint=hint keywords=keywords /> }
        .into_any();
    let right = view! { <div class="canvas">{move || match model.get() {
        Ok(value) => render(value, text),
        Err(_) => view! { <p class="canvas-empty">"Resolve the errors in the source editor to update this view."</p> }.into_any(),
    }}</div> }.into_any();
    view! { <ToolShell meta=meta plans=plans method_reference=method_reference left=left right=right /> }
}

pub fn route(slug: &str) -> AnyView {
    match slug {
        "hephaestus" => view! { <hephaestus::HephaestusTool /> }.into_any(),
        "danaus" => view! { <danaus::DanausTool /> }.into_any(),
        "echo" => view! { <echo::EchoTool /> }.into_any(),
        "atlas" => view! { <atlas::AtlasTool /> }.into_any(),
        "demeter" => view! { <demeter::DemeterTool /> }.into_any(),
        "chronos" => view! { <chronos::ChronosTool /> }.into_any(),
        "hermes" => view! { <hermes::HermesTool /> }.into_any(),
        "iris" => view! { <iris::IrisTool /> }.into_any(),
        "hestia" => view! { <hestia::HestiaTool /> }.into_any(),
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
        "artemis" => view! { <artemis::ArtemisTool /> }.into_any(),
        "metis" => view! { <metis::MetisPage /> }.into_any(),
        "psyche" => view! { <psyche::PsychePage /> }.into_any(),
        other => view! { <NotFound slug=other.to_string() /> }.into_any(),
    }
}
