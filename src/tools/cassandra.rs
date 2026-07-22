//! Cassandra — a premortem risk grid.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Premortem";

const HINT: &str = "outcome: <failure>  ·  cause: <why>  ·  (indent) likelihood/impact: high|medium|low  ·  mitigation: <fix>";

const DEFAULT: &str = "\
outcome: The checkout redesign launched three weeks late and increased failed payments
cause: The legacy payment migration took longer than estimated
  likelihood: high
  impact: high
  mitigation: Prototype the migration and time it against production-scale data
cause: The payment provider sandbox behaved differently from production
  likelihood: medium
  impact: high
  mitigation: Run a small production canary with automatic rollback
cause: Missing funnel telemetry hid regressions until customer reports arrived
  likelihood: medium
  impact: medium
  mitigation: Define launch dashboards and alerts before implementation starts
cause: Support agents were not trained on the new recovery flow
  likelihood: low
  impact: low
  mitigation: Rehearse failure scenarios with support before launch
cause: Late feature requests displaced reliability work
  likelihood: high
  impact: low
  mitigation: Freeze scope two weeks before launch and require an explicit trade-off
";

#[derive(Clone, Copy, PartialEq)]
pub enum Level {
    High,
    Medium,
    Low,
}

impl Level {
    fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "high" => Some(Self::High),
            "medium" | "med" => Some(Self::Medium),
            "low" => Some(Self::Low),
            _ => None,
        }
    }

    fn rank(self) -> u8 {
        match self {
            Self::High => 3,
            Self::Medium => 2,
            Self::Low => 1,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Cause {
    pub text: String,
    pub likelihood: Option<Level>,
    pub impact: Option<Level>,
    pub mitigation: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct Premortem {
    pub outcome: String,
    pub causes: Vec<Cause>,
}

/// Parse premortem text into an imagined outcome and rated causes.
pub fn parse(input: &str) -> (Premortem, Vec<ParseIssue>) {
    let mut model = Premortem {
        outcome: String::new(),
        causes: Vec::new(),
    };
    let mut issues = Vec::new();
    let mut outcome_set = false;
    let mut current_cause = None;

    for line in scan(input) {
        if line.indent >= 2 {
            issues.push(ParseIssue::warn(line.number, "indented too deeply"));
            continue;
        }

        let (keyword, value) = line.colon();
        match line.indent {
            0 => {
                current_cause = None;
                match keyword {
                    "outcome" => {
                        if outcome_set {
                            issues.push(ParseIssue::warn(
                                line.number,
                                "only the first outcome is used",
                            ));
                        } else {
                            outcome_set = true;
                            model.outcome = value.to_string();
                        }
                        if value.is_empty() {
                            issues.push(ParseIssue::warn(line.number, "empty outcome"));
                        }
                    }
                    "cause" => {
                        if value.is_empty() {
                            issues.push(ParseIssue::warn(line.number, "empty cause"));
                        }
                        model.causes.push(Cause {
                            text: value.to_string(),
                            likelihood: None,
                            impact: None,
                            mitigation: None,
                        });
                        current_cause = Some(model.causes.len() - 1);
                    }
                    _ => issues.push(ParseIssue::error(
                        line.number,
                        "expected `outcome:` or `cause:`",
                    )),
                }
            }
            1 => match keyword {
                "likelihood" => {
                    let Some(index) = current_cause else {
                        issues.push(ParseIssue::error(
                            line.number,
                            "likelihood must sit under a cause",
                        ));
                        continue;
                    };
                    match Level::parse(value) {
                        Some(level) => model.causes[index].likelihood = Some(level),
                        None => issues.push(ParseIssue::error(
                            line.number,
                            "likelihood must be high, medium, or low",
                        )),
                    }
                }
                "impact" => {
                    let Some(index) = current_cause else {
                        issues.push(ParseIssue::error(
                            line.number,
                            "impact must sit under a cause",
                        ));
                        continue;
                    };
                    match Level::parse(value) {
                        Some(level) => model.causes[index].impact = Some(level),
                        None => issues.push(ParseIssue::error(
                            line.number,
                            "impact must be high, medium, or low",
                        )),
                    }
                }
                "mitigation" => {
                    let Some(index) = current_cause else {
                        issues.push(ParseIssue::error(
                            line.number,
                            "mitigation must sit under a cause",
                        ));
                        continue;
                    };
                    model.causes[index].mitigation = Some(value.to_string());
                }
                _ => issues.push(ParseIssue::error(
                    line.number,
                    "expected likelihood, impact, or mitigation",
                )),
            },
            _ => unreachable!(),
        }
    }

    for cause in &model.causes {
        if cause.likelihood.is_none() || cause.impact.is_none() {
            issues.push(ParseIssue::warn(
                1,
                "a cause is missing likelihood or impact",
            ));
        }
    }

    (model, issues)
}

fn render(model: Premortem) -> AnyView {
    if model.causes.is_empty() {
        return view! {
            <p class="canvas-empty">"Describe the failure and its causes on the left."</p>
        }
        .into_any();
    }

    let outcome = if model.outcome.is_empty() {
        "(no outcome yet)".to_string()
    } else {
        model.outcome.clone()
    };
    let impacts = [Level::Low, Level::Medium, Level::High];
    let likelihoods = [Level::High, Level::Medium, Level::Low];
    let mut grid: Vec<AnyView> = Vec::new();

    grid.push(view! { <div class="pm-corner"></div> }.into_any());
    for impact in impacts {
        grid.push(view! { <div class="pm-axis-x">{impact.label()}</div> }.into_any());
    }

    for likelihood in likelihoods {
        grid.push(view! { <div class="pm-axis-y">{likelihood.label()}</div> }.into_any());
        for impact in impacts {
            let severity = match likelihood.rank() + impact.rank() {
                5.. => "pm-cell pm-sev-high",
                3..=4 => "pm-cell pm-sev-med",
                _ => "pm-cell pm-sev-low",
            };
            let chips = model
                .causes
                .iter()
                .filter(|cause| {
                    cause.likelihood == Some(likelihood) && cause.impact == Some(impact)
                })
                .map(|cause| {
                    let text = cause.text.clone();
                    if let Some(mitigation) = cause.mitigation.clone() {
                        view! { <span class="pm-chip" title=mitigation>{text}</span> }.into_any()
                    } else {
                        view! { <span class="pm-chip">{text}</span> }.into_any()
                    }
                })
                .collect_view();
            grid.push(view! { <div class=severity>{chips}</div> }.into_any());
        }
    }

    let unrated = model
        .causes
        .iter()
        .filter(|cause| cause.likelihood.is_none() || cause.impact.is_none())
        .map(|cause| cause.text.as_str())
        .collect::<Vec<_>>();
    let unrated_view = if unrated.is_empty() {
        None
    } else {
        Some(view! { <div class="pm-unrated">{format!("Unrated: {}", unrated.join(", "))}</div> })
    };

    view! {
        <div class="pm-board">
            <div class="pm-outcome">{outcome}</div>
            <div class="pm-grid">{grid.into_iter().collect_view()}</div>
            {unrated_view}
        </div>
    }
    .into_any()
}

#[component]
pub fn CassandraTool() -> impl IntoView {
    let meta = registry::find("cassandra").expect("cassandra registered");
    let text = crate::ui::use_persisted("cassandra", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane
            text=text
            issues=issues
            syntax_hint=HINT
            keywords=&["outcome", "cause", "likelihood", "impact", "mitigation"]
        />
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
    fn default_parses_outcome_causes_and_levels() {
        let (model, issues) = parse(DEFAULT);
        assert!(!model.outcome.is_empty());
        assert_eq!(model.causes.len(), 5);
        assert!(model.causes[0].likelihood == Some(Level::High));
        assert!(model.causes[1].impact == Some(Level::High));
        assert!(model.causes[2].likelihood == Some(Level::Medium));
        assert!(issues.iter().all(|issue| issue.severity != Severity::Error));
    }

    #[test]
    fn invalid_likelihood_errors() {
        let (_, issues) =
            parse("cause: The launch slipped\n  likelihood: likely\n  impact: high\n");
        assert!(issues.iter().any(|issue| {
            issue.severity == Severity::Error
                && issue.message == "likelihood must be high, medium, or low"
        }));
    }

    #[test]
    fn missing_level_warns() {
        let (_, issues) = parse("cause: The launch slipped\n  likelihood: med\n");
        assert!(issues.iter().any(|issue| {
            issue.severity == Severity::Warning
                && issue.message == "a cause is missing likelihood or impact"
        }));
    }
}
