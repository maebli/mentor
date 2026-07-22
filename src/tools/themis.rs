//! Themis — a weighted decision matrix for comparing scored options.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Decision-matrix_method";

const HINT: &str = "criterion <Name> <weight>  ·  option <Name>  ·  (indent) <Criterion>: <score>";

const DEFAULT: &str = "\
criterion Query flexibility 5
criterion Operational simplicity 4
criterion Scalability 3
criterion Cost efficiency 2

option PostgreSQL
  Query flexibility: 5
  Operational simplicity: 4
  Scalability: 4
  Cost efficiency: 4

option DynamoDB
  Query flexibility: 3
  Operational simplicity: 2
  Scalability: 5
  Cost efficiency: 3

option SQLite
  Query flexibility: 3
  Operational simplicity: 5
  Scalability: 1
  Cost efficiency: 5
";

#[derive(Clone, PartialEq)]
pub struct Criterion {
    pub name: String,
    pub weight: f64,
}

#[derive(Clone, PartialEq)]
pub struct Opt {
    pub name: String,
    pub scores: Vec<(usize, f64)>,
}

#[derive(Clone, PartialEq)]
pub struct Matrix {
    pub criteria: Vec<Criterion>,
    pub options: Vec<Opt>,
}

/// Parse criteria, options, and their indented scores into a decision matrix.
pub fn parse(input: &str) -> (Matrix, Vec<ParseIssue>) {
    let mut matrix = Matrix {
        criteria: Vec::new(),
        options: Vec::new(),
    };
    let mut issues = Vec::new();
    let mut current_option = None;

    for line in scan(input) {
        match line.indent {
            0 => {
                current_option = None;
                let (keyword, rest) = line.keyword();
                match keyword {
                    "criterion" => {
                        let parts: Vec<&str> = rest.split_whitespace().collect();
                        let Some(weight) = parts.last().and_then(|token| token.parse::<f64>().ok())
                        else {
                            issues.push(ParseIssue::error(
                                line.number,
                                "a criterion needs a weight, e.g. `criterion Cost 5`",
                            ));
                            continue;
                        };
                        let name = parts[..parts.len() - 1].join(" ");
                        if name.is_empty() {
                            issues.push(ParseIssue::error(line.number, "a criterion needs a name"));
                            continue;
                        }
                        if weight <= 0.0 {
                            issues.push(ParseIssue::warn(line.number, "weight should be positive"));
                        }
                        matrix.criteria.push(Criterion { name, weight });
                    }
                    "option" => {
                        if rest.is_empty() {
                            issues.push(ParseIssue::error(line.number, "an option needs a name"));
                            continue;
                        }
                        matrix.options.push(Opt {
                            name: rest.to_string(),
                            scores: Vec::new(),
                        });
                        current_option = Some(matrix.options.len() - 1);
                    }
                    _ => issues.push(ParseIssue::error(
                        line.number,
                        "expected `criterion` or `option`",
                    )),
                }
            }
            1 => {
                if !line.content.contains(':') {
                    issues.push(ParseIssue::error(
                        line.number,
                        "expected `<Criterion>: <score>`",
                    ));
                    continue;
                }
                let Some(option_index) = current_option else {
                    issues.push(ParseIssue::error(
                        line.number,
                        "a score must sit under an option",
                    ));
                    continue;
                };
                let (key, value) = line.colon();
                let Some(criterion_index) = matrix
                    .criteria
                    .iter()
                    .position(|criterion| criterion.name.eq_ignore_ascii_case(key))
                else {
                    issues.push(ParseIssue::warn(
                        line.number,
                        format!("no criterion named “{key}”"),
                    ));
                    continue;
                };
                let Ok(score) = value.parse::<f64>() else {
                    issues.push(ParseIssue::error(line.number, "score must be a number"));
                    continue;
                };

                let scores = &mut matrix.options[option_index].scores;
                if let Some((_, old_score)) = scores
                    .iter_mut()
                    .find(|(index, _)| *index == criterion_index)
                {
                    issues.push(ParseIssue::warn(
                        line.number,
                        format!("duplicate score for “{key}” (last wins)"),
                    ));
                    *old_score = score;
                } else {
                    scores.push((criterion_index, score));
                }
            }
            _ => issues.push(ParseIssue::error(
                line.number,
                "expected `<Criterion>: <score>`",
            )),
        }
    }

    for option in &matrix.options {
        for (criterion_index, criterion) in matrix.criteria.iter().enumerate() {
            if !option
                .scores
                .iter()
                .any(|(index, _)| *index == criterion_index)
            {
                issues.push(ParseIssue::warn(
                    1,
                    format!("“{}” has no score for “{}”", option.name, criterion.name),
                ));
            }
        }
    }

    (matrix, issues)
}

fn weighted_total(matrix: &Matrix, option: &Opt) -> f64 {
    matrix
        .criteria
        .iter()
        .enumerate()
        .map(|(index, criterion)| {
            let score = option
                .scores
                .iter()
                .find(|(criterion_index, _)| *criterion_index == index)
                .map(|(_, score)| *score)
                .unwrap_or(0.0);
            criterion.weight * score
        })
        .sum()
}

fn winner_index(totals: &[f64]) -> Option<usize> {
    totals
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
}

fn format_number(value: f64) -> String {
    let formatted = format!("{value:.1}");
    formatted
        .strip_suffix(".0")
        .unwrap_or(&formatted)
        .to_string()
}

fn render(matrix: Matrix) -> AnyView {
    if matrix.criteria.is_empty() || matrix.options.is_empty() {
        return view! {
            <p class="canvas-empty">
                "Add criteria (with weights) and options with scores on the left."
            </p>
        }
        .into_any();
    }

    let totals: Vec<f64> = matrix
        .options
        .iter()
        .map(|option| weighted_total(&matrix, option))
        .collect();
    let winner = winner_index(&totals);
    let headers = matrix
        .criteria
        .iter()
        .map(|criterion| {
            view! {
                <th class="dm-th">
                    {criterion.name.clone()}
                    <span class="dm-weight">{format!("×{}", format_number(criterion.weight))}</span>
                </th>
            }
        })
        .collect_view();
    let rows = matrix
        .options
        .iter()
        .enumerate()
        .map(|(option_index, option)| {
            let class = if winner == Some(option_index) {
                "dm-row dm-winner"
            } else {
                "dm-row"
            };
            let scores = matrix
                .criteria
                .iter()
                .enumerate()
                .map(|(criterion_index, _)| {
                    let value = option
                        .scores
                        .iter()
                        .find(|(index, _)| *index == criterion_index)
                        .map(|(_, score)| format_number(*score))
                        .unwrap_or_else(|| "—".to_string());
                    view! { <td class="dm-score">{value}</td> }
                })
                .collect_view();
            view! {
                <tr class=class>
                    <td class="dm-opt">{option.name.clone()}</td>
                    {scores}
                    <td class="dm-total">{format_number(totals[option_index])}</td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <table class="dm-table">
            <thead>
                <tr>
                    <th class="dm-th dm-opt-h">"Option"</th>
                    {headers}
                    <th class="dm-th dm-total-h">"Total"</th>
                </tr>
            </thead>
            <tbody>{rows}</tbody>
        </table>
    }
    .into_any()
}

#[component]
pub fn ThemisTool() -> impl IntoView {
    let meta = registry::find("themis").expect("themis registered");
    let text = crate::ui::use_persisted("themis", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane
            text=text
            issues=issues
            syntax_hint=HINT
            keywords=&["criterion", "option"]
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
    fn default_parses_and_highest_total_wins() {
        let (matrix, issues) = parse(DEFAULT);
        assert_eq!(matrix.criteria.len(), 4);
        assert_eq!(matrix.options.len(), 3);
        assert!(issues.is_empty());

        let totals: Vec<f64> = matrix
            .options
            .iter()
            .map(|option| weighted_total(&matrix, option))
            .collect();
        assert_eq!(totals, vec![61.0, 44.0, 48.0]);
        assert_eq!(winner_index(&totals), Some(0));
    }

    #[test]
    fn criterion_without_weight_errors() {
        let (_, issues) = parse("criterion Cost\n");
        assert!(issues.iter().any(|issue| {
            issue.severity == Severity::Error
                && issue.message == "a criterion needs a weight, e.g. `criterion Cost 5`"
        }));
    }

    #[test]
    fn unknown_criterion_under_option_warns() {
        let (_, issues) = parse("criterion Cost 5\noption Postgres\n  Speed: 3\n");
        assert!(issues.iter().any(|issue| {
            issue.severity == Severity::Warning && issue.message == "no criterion named “Speed”"
        }));
    }
}
