//! Socrates — 5 Whys.
//!
//! Ask "why" repeatedly to peel symptoms away from the underlying cause. Each
//! answer becomes the cause-of-the-line-above until you reach something
//! actionable. The text format is line-based and indentation-free:
//! ```text
//! problem: <the problem statement>
//! why: <a cause of the line above>
//! why: <a cause of that cause>
//! ```
//! Colon form is primary; the `why <text>` keyword form also works, and a
//! leading `why:` may be repeated on a single line.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Five_whys";

const HINT: &str = "problem: <statement>   ·   why: <cause>   (repeat why for each level)";

const DEFAULT: &str = "\
problem: Deployments fail intermittently in production
why: The migration step times out before completing
why: A long-running ALTER TABLE locks the orders table
why: The table has grown to 50M rows and the change rewrites every row
why: We never partitioned or archived historical data
why: No data retention policy was ever defined
";

#[derive(Clone, PartialEq)]
pub struct FiveWhys {
    pub problem: String,
    pub whys: Vec<String>,
}

/// Parse 5-Whys text into a model, collecting syntax issues by line.
pub fn parse(input: &str) -> (FiveWhys, Vec<ParseIssue>) {
    let mut model = FiveWhys { problem: String::new(), whys: Vec::new() };
    let mut issues: Vec<ParseIssue> = Vec::new();
    let mut problem_set = false;
    let mut last_why_line: Option<usize> = None;

    let lines = scan(input);

    for line in &lines {
        // Colon form is primary; fall back to whitespace-keyword form so
        // `why <text>` works too. Match agora's split-then-dispatch style.
        let (kw, rest) = if line.content.contains(':') {
            line.colon()
        } else {
            line.keyword()
        };
        match kw {
            "problem" => {
                if !problem_set {
                    problem_set = true;
                    if rest.is_empty() {
                        issues.push(ParseIssue::warn(line.number, "empty problem statement"));
                    } else {
                        model.problem = rest.to_string();
                    }
                } else {
                    issues.push(ParseIssue::warn(line.number, "only the first problem is used"));
                }
            }
            "why" => {
                let answer = strip_leading_why(rest);
                if answer.is_empty() {
                    issues.push(ParseIssue::warn(line.number, "empty `why:` — add a cause"));
                }
                model.whys.push(answer.to_string());
                last_why_line = Some(line.number);
            }
            _ => {
                issues.push(ParseIssue::error(
                    line.number,
                    "expected `problem:` or `why:`",
                ));
            }
        }
    }

    if !problem_set {
        let ln = lines.first().map(|l| l.number).unwrap_or(1);
        issues.push(ParseIssue::error(ln, "first meaningful line must be `problem:`"));
    }

    if model.whys.len() >= 1 && model.whys.len() < 5 {
        if let Some(ln) = last_why_line {
            issues.push(ParseIssue::warn(ln, "fewer than 5 whys — keep asking"));
        }
    }

    (model, issues)
}

/// Strip any number of repeated leading `why:` / `why ` markers from a cause
/// string, so a line like `why: why: because X` counts as a single answer.
fn strip_leading_why(mut s: &str) -> &str {
    loop {
        let trimmed = s.trim();
        let stripped = trimmed
            .strip_prefix("why:")
            .or_else(|| trimmed.strip_prefix("why "));
        match stripped {
            Some(rest) => s = rest,
            None => return trimmed,
        }
    }
}

fn render(model: FiveWhys) -> AnyView {
    if model.problem.is_empty() {
        return view! {
            <p class="canvas-empty">"Describe a problem on the left."</p>
        }
        .into_any();
    }

    let FiveWhys { problem, whys } = model;
    let n_whys = whys.len();

    let mut nodes: Vec<AnyView> = Vec::new();
    nodes.push(
        view! {
            <div class="chain-node node-problem">
                <span class="node-label">"problem"</span>
                <div class="node-body">{problem}</div>
            </div>
        }
        .into_any(),
    );

    for (i, why) in whys.into_iter().enumerate() {
        let is_root = i + 1 == n_whys;
        let cls = if is_root {
            "chain-node node-why node-root"
        } else {
            "chain-node node-why"
        };
        nodes.push(view! { <div class="why-arrow">"why?"</div> }.into_any());
        if is_root {
            nodes.push(
                view! {
                    <div class=cls>
                        <span class="node-label">"why"</span>
                        <div class="node-body">{why}</div>
                        <span class="root-badge">"root cause"</span>
                    </div>
                }
                .into_any(),
            );
        } else {
            nodes.push(
                view! {
                    <div class=cls>
                        <span class="node-label">"why"</span>
                        <div class="node-body">{why}</div>
                    </div>
                }
                .into_any(),
            );
        }
    }

    view! {
        <div class="chain">{nodes.into_iter().collect_view()}</div>
    }
    .into_any()
}

#[component]
pub fn SocratesTool() -> impl IntoView {
    let meta = registry::find("socrates").expect("socrates registered");
    let text = crate::ui::use_persisted("socrates", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane text=text issues=issues syntax_hint=HINT />
    }
    .into_any();
    let right = view! {
        <div class="canvas">{move || render(parsed.get().0)}</div>
    }
    .into_any();

    view! { <ToolShell meta=meta reference=REFERENCE text=text left=left right=right /> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Severity;

    #[test]
    fn parses_default_into_problem_and_five_whys() {
        let (model, issues) = parse(DEFAULT);
        assert_eq!(model.problem, "Deployments fail intermittently in production");
        assert_eq!(model.whys.len(), 5);
        assert_eq!(model.whys.last().unwrap(), "No data retention policy was ever defined");
        assert!(issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn errors_when_problem_missing() {
        let (_, issues) = parse("why: nothing to anchor\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("problem")));
    }

    #[test]
    fn errors_on_unknown_keyword() {
        let (_, issues) = parse("problem: something is wrong\nfoo: bar\n");
        assert!(issues
            .iter()
            .any(|i| i.message.contains("expected") && i.message.contains("why")));
    }

    #[test]
    fn warns_when_fewer_than_five_whys() {
        let (_, issues) = parse("problem: x\nwhy: a\nwhy: b\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Warning && i.message.contains("fewer than 5")));
    }
}
