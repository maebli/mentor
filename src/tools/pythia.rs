//! Pythia — reference-class forecasting from comparable past projects.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Reference_class_forecasting";

const HINT: &str = "question: <what>  ·  unit: <weeks>  ·  estimate: <n>  ·  case: <label> <n>";

const DEFAULT: &str = "\
question: How long will the billing platform migration take?
unit: weeks
estimate: 8
case: Identity service migration 13
case: Payments API rewrite 16
case: Customer data migration 11
case: Reporting pipeline replacement 19
case: Order service split 14
case: Auth platform consolidation 17
";

#[derive(Clone, PartialEq)]
struct Case {
    label: String,
    value: f64,
}

#[derive(Clone, PartialEq)]
struct Forecast {
    question: String,
    unit: String,
    estimate: Option<f64>,
    cases: Vec<Case>,
}

/// Parse an estimate and its reference class, collecting issues by source line.
fn parse(input: &str) -> (Forecast, Vec<ParseIssue>) {
    let mut forecast = Forecast {
        question: String::new(),
        unit: String::new(),
        estimate: None,
        cases: Vec::new(),
    };
    let mut issues = Vec::new();

    for line in scan(input) {
        let (keyword, value) = line.colon();
        match keyword {
            "question" => forecast.question = value.to_string(),
            "unit" => forecast.unit = value.to_string(),
            "estimate" => match value.trim().parse::<f64>() {
                Ok(estimate) => forecast.estimate = Some(estimate),
                Err(_) => issues.push(ParseIssue::error(line.number, "estimate must be a number")),
            },
            "case" => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                let parsed_value = parts.last().and_then(|part| part.parse::<f64>().ok());
                let Some(case_value) = parsed_value else {
                    issues.push(ParseIssue::error(
                        line.number,
                        "a case needs a number, e.g. `case: Payments API 14`",
                    ));
                    continue;
                };

                let label = parts[..parts.len() - 1].join(" ");
                if label.is_empty() {
                    issues.push(ParseIssue::warn(line.number, "case label is empty"));
                }
                forecast.cases.push(Case {
                    label,
                    value: case_value,
                });
            }
            _ => issues.push(ParseIssue::error(
                line.number,
                "expected question, unit, estimate, or case",
            )),
        }
    }

    match forecast.cases.len() {
        0 => issues.push(ParseIssue::warn(1, "add at least one reference case")),
        1..=4 => issues.push(ParseIssue::warn(
            1,
            "small reference class (n<5) — treat the base rate with caution",
        )),
        _ => {}
    }

    if let Some(estimate) = forecast.estimate {
        if !forecast.cases.is_empty() && forecast.cases.iter().all(|case| estimate < case.value) {
            issues.push(ParseIssue::warn(
                1,
                "your estimate is below every comparable case — likely optimistic",
            ));
        }
    }

    (forecast, issues)
}

fn format_number(value: f64) -> String {
    let value = if value.abs() < 0.05 { 0.0 } else { value };
    let formatted = format!("{value:.1}");
    formatted
        .strip_suffix(".0")
        .unwrap_or(&formatted)
        .to_string()
}

fn render(forecast: Forecast) -> AnyView {
    if forecast.cases.is_empty() {
        return view! {
            <p class="canvas-empty">
                "Add your estimate and some comparable past projects on the left."
            </p>
        }
        .into_any();
    }

    let mut values: Vec<f64> = forecast.cases.iter().map(|case| case.value).collect();
    values.sort_by(f64::total_cmp);
    let min = values[0];
    let max = values[values.len() - 1];
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let median = if values.len() % 2 == 0 {
        let upper = values.len() / 2;
        (values[upper - 1] + values[upper]) / 2.0
    } else {
        values[values.len() / 2]
    };

    let mut lo = min;
    let mut hi = max;
    if let Some(estimate) = forecast.estimate {
        lo = lo.min(estimate);
        hi = hi.max(estimate);
    }
    let span = hi - lo;
    let padding = if span.abs() < f64::EPSILON {
        hi.abs().max(1.0) * 0.08
    } else {
        span * 0.08
    };
    lo -= padding;
    hi += padding;

    let w = 900.0_f64;
    let h = 430.0_f64;
    let plot_left = 70.0_f64;
    let plot_right = w - 40.0;
    let plot_top = 88.0_f64;
    let plot_bottom = h - 58.0;
    let axis_y = h * 0.53;
    let value_x = |value: f64| {
        let clamped = value.max(lo).min(hi);
        plot_left + (clamped - lo) / (hi - lo) * (plot_right - plot_left)
    };
    let unit = forecast.unit.trim();
    let unit_suffix = if unit.is_empty() {
        String::new()
    } else {
        format!(" {unit}")
    };
    let estimate_summary = forecast
        .estimate
        .map(|estimate| format!("{}{}", format_number(estimate), unit_suffix))
        .unwrap_or_else(|| "—".to_string());
    let summary = format!(
        "Reference median {}{} · mean {} · range {}–{} · your estimate {}",
        format_number(median),
        unit_suffix,
        format_number(mean),
        format_number(min),
        format_number(max),
        estimate_summary,
    );

    let mut els: Vec<AnyView> = Vec::new();
    els.push(
        view! {
            <text class="fc-summary" x=w / 2.0 y=30.0 text-anchor="middle">
                {summary}
            </text>
        }
        .into_any(),
    );
    els.push(
        view! {
            <line
                class="fc-axis"
                x1=plot_left
                y1=axis_y
                x2=plot_right
                y2=axis_y
            />
        }
        .into_any(),
    );

    for i in 0..=4 {
        let value = lo + (hi - lo) * i as f64 / 4.0;
        els.push(
            view! {
                <text
                    class="fc-tick"
                    x=value_x(value)
                    y=axis_y + 22.0
                    text-anchor="middle"
                >
                    {format_number(value)}
                </text>
            }
            .into_any(),
        );
    }

    let median_x = value_x(median);
    els.push(
        view! {
            <line
                class="fc-median"
                x1=median_x
                y1=plot_top
                x2=median_x
                y2=plot_bottom
            />
        }
        .into_any(),
    );
    els.push(
        view! {
            <text
                class="fc-median-label"
                x=median_x
                y=plot_bottom + 23.0
                text-anchor="middle"
            >
                {format!("median {}", format_number(median))}
            </text>
        }
        .into_any(),
    );

    // Only the (short) value sits at each dot; full names go in the legend below
    // the chart, so nothing overlaps or gets clipped at the edges.
    for case in forecast.cases.iter() {
        let x = value_x(case.value);
        els.push(view! { <circle class="fc-case" cx=x cy=axis_y r=6.0 /> }.into_any());
        els.push(
            view! {
                <text class="fc-case-label" x=x y=axis_y - 13.0 text-anchor="middle">
                    {format_number(case.value)}
                </text>
            }
            .into_any(),
        );
    }

    if let Some(estimate) = forecast.estimate {
        let estimate_x = value_x(estimate);
        let marker_y = plot_top;
        let points = format!(
            "{},{} {},{} {},{}",
            estimate_x - 8.0,
            marker_y - 13.0,
            estimate_x + 8.0,
            marker_y - 13.0,
            estimate_x,
            marker_y,
        );
        els.push(
            view! {
                <line
                    class="fc-estimate"
                    x1=estimate_x
                    y1=plot_top
                    x2=estimate_x
                    y2=plot_bottom
                />
            }
            .into_any(),
        );
        els.push(view! { <polygon class="fc-estimate" points=points /> }.into_any());
        els.push(
            view! {
                <text
                    class="fc-estimate-label"
                    x=estimate_x
                    y=plot_top - 23.0
                    text-anchor="middle"
                >
                    {format!("your estimate {}", format_number(estimate))}
                </text>
            }
            .into_any(),
        );
    }

    let legend = forecast
        .cases
        .iter()
        .map(|case| {
            let label = if case.label.is_empty() {
                format_number(case.value)
            } else {
                format!("{} · {}{}", case.label, format_number(case.value), unit_suffix)
            };
            view! { <span class="fc-legend-item">{label}</span> }
        })
        .collect_view();

    let viewbox = format!("0 0 {w} {h}");
    view! {
        <div class="fc-wrap">
            <svg class="fc-svg" viewBox=viewbox preserveAspectRatio="xMidYMid meet">
                {els.into_iter().collect_view()}
            </svg>
            <div class="fc-legend">{legend}</div>
        </div>
    }
    .into_any()
}

#[component]
pub fn PythiaTool() -> impl IntoView {
    let meta = registry::find("pythia").expect("pythia registered");
    let text = crate::ui::use_persisted("pythia", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane
            text=text
            issues=issues
            syntax_hint=HINT
            keywords=&["question", "unit", "estimate", "case"]
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
    fn default_parses_question_estimate_and_cases() {
        let (forecast, issues) = parse(DEFAULT);
        assert_eq!(
            forecast.question,
            "How long will the billing platform migration take?"
        );
        assert_eq!(forecast.estimate, Some(8.0));
        assert_eq!(forecast.cases.len(), 6);
        assert_eq!(
            forecast
                .cases
                .iter()
                .map(|case| case.value)
                .collect::<Vec<_>>(),
            vec![13.0, 16.0, 11.0, 19.0, 14.0, 17.0]
        );
        assert!(issues.iter().all(|issue| issue.severity != Severity::Error));
    }

    #[test]
    fn case_without_a_number_errors() {
        let (_, issues) = parse("case: Payments API\n");
        assert!(issues.iter().any(|issue| {
            issue.severity == Severity::Error && issue.message.contains("case needs a number")
        }));
    }

    #[test]
    fn default_warns_that_the_estimate_is_optimistic() {
        let (_, issues) = parse(DEFAULT);
        assert!(issues.iter().any(|issue| {
            issue.severity == Severity::Warning
                && issue.message
                    == "your estimate is below every comparable case — likely optimistic"
        }));
    }
}
