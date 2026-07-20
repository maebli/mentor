//! Morpheus — morphological analysis (Zwicky box).
//!
//! A parameter/option matrix for exploring a design space. Each parameter is a
//! row; click one option per row to compose a candidate configuration.
//!
//! Text format (2-space indentation):
//! ```text
//! param Data store
//!   option Postgres
//!   option DynamoDB
//! ```
//!
//! This follows the same shape as `agora.rs`: a pure `parse` (unit-tested), a
//! `render` that turns the model into a view, and a `*Tool` component wiring
//! them together. The twist is selection state: a `RwSignal<HashMap<..>>` holds
//! the chosen option index per parameter, read inside `render` so the matrix
//! re-renders on click.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;
use std::collections::HashMap;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Morphological_analysis_(problem-solving)";

const HINT: &str = "param <name>  ·  (indent) option <value>   — then click one option per row";

const DEFAULT: &str = "\
param Data store
  option Postgres
  option DynamoDB
  option SQLite

param API style
  option REST
  option GraphQL
  option gRPC

param Deploy target
  option Kubernetes
  option Serverless
  option VM

param Auth
  option JWT
  option Session
  option OAuth
";

#[derive(Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub options: Vec<String>,
}

/// Parse Zwicky-box text into parameters (each with its options), collecting
/// syntax issues by line. After the line-by-line pass, parameters with fewer
/// than two options are warned about — line 1 is used as the anchor since the
/// parameter itself spans several lines.
pub fn parse(input: &str) -> (Vec<Param>, Vec<ParseIssue>) {
    let mut params: Vec<Param> = Vec::new();
    let mut issues: Vec<ParseIssue> = Vec::new();

    for line in scan(input) {
        match line.indent {
            0 => {
                let (kw, rest) = line.keyword();
                if kw == "param" {
                    if rest.is_empty() {
                        issues.push(ParseIssue::error(line.number, "`param` needs a name"));
                        continue;
                    }
                    params.push(Param {
                        name: rest.to_string(),
                        options: Vec::new(),
                    });
                } else {
                    issues.push(ParseIssue::error(
                        line.number,
                        format!("expected `param <name>`, found “{kw}”"),
                    ));
                }
            }
            1 => {
                let (kw, rest) = line.keyword();
                if kw != "option" {
                    issues.push(ParseIssue::error(
                        line.number,
                        format!("expected `option <value>`, found “{kw}”"),
                    ));
                    continue;
                }
                let Some(param) = params.last_mut() else {
                    issues.push(ParseIssue::error(
                        line.number,
                        "option must sit under a param",
                    ));
                    continue;
                };
                if rest.is_empty() {
                    issues.push(ParseIssue::warn(line.number, "empty option"));
                } else {
                    param.options.push(rest.to_string());
                }
            }
            _ => issues.push(ParseIssue::error(
                line.number,
                "indented too deeply — use one level under a param",
            )),
        }
    }

    for param in &params {
        if param.options.len() < 2 {
            issues.push(ParseIssue::warn(
                1,
                format!("param “{}” has fewer than 2 options", param.name),
            ));
        }
    }

    (params, issues)
}

/// Render the Zwicky matrix. `selection` is read inside this function, so the
/// closure that calls `render` re-runs whenever the selection changes.
fn render(params: Vec<Param>, selection: RwSignal<HashMap<usize, usize>>) -> AnyView {
    if params.is_empty() {
        return view! {
            <p class="canvas-empty">"Add parameters and options on the left."</p>
        }
        .into_any();
    }

    let total = params.len();
    let snapshot = selection.get();
    let chosen = snapshot.len();

    let rows = params
        .iter()
        .enumerate()
        .map(|(pi, param)| {
            let chips = param
                .options
                .iter()
                .enumerate()
                .map(move |(oi, value)| {
                    let selected = selection.get().get(&pi) == Some(&oi);
                    let cls = if selected { "zbox-chip sel" } else { "zbox-chip" };
                    view! {
                        <button class=cls
                            on:click=move |_| {
                                selection.update(move |m| {
                                    m.insert(pi, oi);
                                });
                            }
                        >
                            {value.clone()}
                        </button>
                    }
                })
                .collect_view();
            view! {
                <div class="zbox-row">
                    <div class="zbox-param">{param.name.clone()}</div>
                    <div class="zbox-options">{chips}</div>
                </div>
            }
        })
        .collect_view();

    let config = if chosen == total {
        let parts: Vec<String> = params
            .iter()
            .enumerate()
            .filter_map(|(pi, p)| snapshot.get(&pi).map(|oi| p.options[*oi].clone()))
            .collect();
        let joined = parts.join(" · ");
        view! {
            <div class="zbox-config">
                <span class="zbox-config-label">"Configuration: "</span>
                <span class="zbox-config-value">{joined}</span>
            </div>
        }
        .into_any()
    } else {
        view! {
            <div class="zbox-config zbox-hint">
                {format!(
                    "Pick one option per row to compose a configuration ({chosen}/{total} chosen)"
                )}
            </div>
        }
        .into_any()
    };

    view! {
        <div class="zbox">
            {rows}
            {config}
        </div>
    }
    .into_any()
}

#[component]
pub fn MorpheusTool() -> impl IntoView {
    let meta = registry::find("morpheus").expect("morpheus registered");
    let text = crate::ui::use_persisted("morpheus", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);
    let selection = RwSignal::new(HashMap::<usize, usize>::new());

    let left = view! {
        <EditorPane text=text issues=issues syntax_hint=HINT />
    }
    .into_any();
    let right = view! {
        <div class="canvas">{move || render(parsed.get().0, selection)}</div>
    }
    .into_any();

    view! { <ToolShell meta=meta reference=REFERENCE left=left right=right /> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Severity;

    #[test]
    fn parses_default_into_params_with_options() {
        let (params, issues) = parse(DEFAULT);
        assert_eq!(params.len(), 4);
        assert_eq!(params[0].name, "Data store");
        assert_eq!(params[0].options, vec!["Postgres", "DynamoDB", "SQLite"]);
        assert_eq!(params[1].name, "API style");
        assert_eq!(params[1].options.len(), 3);
        assert_eq!(params[2].name, "Deploy target");
        assert_eq!(params[2].options.len(), 3);
        assert_eq!(params[3].name, "Auth");
        assert_eq!(params[3].options.len(), 3);
        assert!(issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn errors_on_option_without_param() {
        let (_, issues) = parse("  option lonely\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("under a param")));
    }

    #[test]
    fn warns_when_param_has_fewer_than_two_options() {
        let (_, issues) = parse("param lonely\n  option only\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("fewer than 2")));
    }
}
