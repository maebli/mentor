//! Herakles — 8D report (eight disciplines of problem solving).
//!
//! Each discipline is written on one line in `dN: <content>` form. D0 is an
//! optional planning step; D1 through D8 make up the report itself.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Eight_disciplines_problem_solving";

const HINT: &str = "d1: build team · d2: problem · d3: containment · d4: root cause · d5: corrective · d6: implement · d7: prevent · d8: close";

const DEFAULT: &str = "\
d1: Assemble the incident commander, platform engineer, database owner, and customer support lead.
d2: Checkout requests returned HTTP 503 for 42 minutes after the 14:00 UTC release.
d3: Roll back the release, disable the new query path, and redirect traffic to healthy instances.
d4: An unbounded database query exhausted the connection pool, and load testing did not include production-scale order histories.
d5: Add a bounded query with pagination and enforce per-request database timeouts.
d6: Deploy the fix to a canary, validate latency and pool saturation under production-scale load, then roll it out globally.
d7: Add representative load-test fixtures, a connection-pool alert, and a release checklist gate for query plans.
d8: Thank the response team, publish the incident review, and close all actions after owners confirm completion.
";

const DISCIPLINE_NAMES: [&str; 9] = [
    "Plan",
    "Build the team",
    "Describe the problem",
    "Interim containment action",
    "Root cause & escape point",
    "Permanent corrective action",
    "Implement & validate",
    "Prevent recurrence",
    "Recognize the team & close",
];

#[derive(Clone, PartialEq)]
pub struct EightD {
    pub items: [Option<String>; 9],
}

/// Parse an 8D report, collecting syntax and completeness issues by line.
pub fn parse(input: &str) -> (EightD, Vec<ParseIssue>) {
    let mut model = EightD {
        items: Default::default(),
    };
    let mut issues = Vec::new();

    for line in scan(input) {
        let (key, rest) = line.colon();
        let key = key.to_ascii_lowercase();
        let discipline = key
            .strip_prefix('d')
            .filter(|digits| digits.len() == 1)
            .and_then(|digits| digits.parse::<usize>().ok())
            .filter(|number| *number <= 8);

        let Some(number) = discipline else {
            issues.push(ParseIssue::error(line.number, "expected `d1:` .. `d8:`"));
            continue;
        };

        if model.items[number].is_some() {
            issues.push(ParseIssue::warn(
                line.number,
                format!("d{number} given more than once (last wins)"),
            ));
        }
        if rest.is_empty() {
            issues.push(ParseIssue::warn(line.number, format!("d{number} is empty")));
        }
        model.items[number] = Some(rest.to_string());
    }

    for number in 1..=8 {
        if model.items[number].is_none() {
            issues.push(ParseIssue::warn(
                1,
                format!("d{number} ({}) is missing", DISCIPLINE_NAMES[number]),
            ));
        }
    }

    (model, issues)
}

fn render(model: EightD) -> AnyView {
    if model.items.iter().all(Option::is_none) {
        return view! {
            <p class="canvas-empty">"Fill in the disciplines on the left (d1: …)."</p>
        }
        .into_any();
    }

    let cards = model
        .items
        .into_iter()
        .enumerate()
        .filter_map(|(number, content)| {
            if number == 0 && content.is_none() {
                return None;
            }

            let missing = content.is_none();
            let class = if missing {
                "d8-card d8-missing"
            } else {
                "d8-card"
            };
            let body = content.unwrap_or_else(|| "—".to_string());

            Some(view! {
                <div class=class>
                    <div class="d8-badge">{format!("D{number}")}</div>
                    <div class="d8-main">
                        <div class="d8-name">{DISCIPLINE_NAMES[number]}</div>
                        <div class="d8-body">{body}</div>
                    </div>
                </div>
            })
        })
        .collect_view();

    view! { <div class="d8-report">{cards}</div> }.into_any()
}

#[component]
pub fn HeraklesTool() -> impl IntoView {
    let meta = registry::find("herakles").expect("herakles registered");
    let text = crate::ui::use_persisted("herakles", DEFAULT);
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

    view! { <ToolShell meta=meta reference=REFERENCE left=left right=right /> }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Severity;

    #[test]
    fn default_fills_all_eight_disciplines() {
        let (model, _) = parse(DEFAULT);
        assert!(model.items[1..=8].iter().all(Option::is_some));
    }

    #[test]
    fn errors_on_unknown_discipline() {
        let (_, issues) = parse("d9: retrospective\n");
        assert!(issues.iter().any(|issue| issue.severity == Severity::Error));
    }

    #[test]
    fn warns_when_a_discipline_is_missing() {
        let (_, issues) = parse("d1: Assemble the response team\n");
        assert!(issues
            .iter()
            .any(|issue| { issue.severity == Severity::Warning && issue.message.contains("d2") }));
    }
}
