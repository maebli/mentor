//! Tyche — probabilistic estimation by convolving each task's uncertainty.
//!
//! A port of the maths behind [guessy](https://github.com/maebli/guessy) (itself
//! after Estigator): every line is one uncertain quantity expressed as a small
//! discrete distribution, and the total is their **convolution** — the exact
//! distribution of their sum. From it we read the mean, the median, and a P5–P95
//! confidence interval, so an estimate carries its own risk.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Convolution_of_probability_distributions";

const HINT: &str =
    "unit: days · verb: take · 4-7 range · up to 5 (or ~5) · 1 or 2 @ 25% · 3 fixed";

const DEFAULT: &str = "\
unit: days
verb: take
4-7 Sketching the UI
2-3 Wiring up the frontend
1 or 2 @ 25% Stakeholder review
3-6 Schema and migrations
up to 5 End-to-end testing
2 or 14 @ 25% Privacy review
~2 CI and release setup
";

/// Units and verbs accepted by the `unit:` / `verb:` directives. Also the words
/// the editor highlights after those keywords.
const VALID_UNITS: &[&str] = &[
    "days", "hours", "weeks", "months", "dollars", "euros", "gbp", "points", "items", "tickets",
];
const VALID_VERBS: &[&str] = &["take", "cost", "score", "yield", "produce", "weigh"];

// ── Model ───────────────────────────────────────────────────────────────────

/// One uncertain quantity, as a discrete distribution over non-negative integers.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Component {
    /// Uniform over every integer in `[min, max]`.
    Between { min: u32, max: u32 },
    /// Uniform over every integer in `[0, max]`.
    UpTo { max: u32 },
    /// `either` with probability `1 - prob_or`, `or` with probability `prob_or`.
    EitherOr { either: u32, or: u32, prob_or: f64 },
}

impl Component {
    /// Raw (un-normalised) probability mass function, index = value.
    fn raw_pmf(&self) -> Vec<f64> {
        match *self {
            Component::Between { min, max } => {
                let (lo, hi) = (min.min(max), min.max(max));
                let mut pmf = vec![0.0; lo as usize];
                pmf.resize((hi + 1) as usize, 1.0);
                pmf
            }
            Component::UpTo { max } => vec![1.0; (max + 1) as usize],
            Component::EitherOr { either, or, prob_or } => {
                let hi = either.max(or) as usize;
                let mut pmf = vec![0.0; hi + 1];
                pmf[either as usize] += 1.0 - prob_or;
                pmf[or as usize] += prob_or;
                pmf
            }
        }
    }

    /// The smallest and largest values this component can take.
    fn range(&self) -> (u32, u32) {
        match *self {
            Component::Between { min, max } => (min.min(max), min.max(max)),
            Component::UpTo { max } => (0, max),
            Component::EitherOr { either, or, .. } => (either.min(or), either.max(or)),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Entry {
    description: String,
    component: Component,
}

#[derive(Clone, PartialEq)]
struct Estimate {
    unit: String,
    verb: String,
    entries: Vec<Entry>,
}

/// Summary statistics of the convolved total.
#[derive(Clone, PartialEq)]
struct Analysis {
    mean: f64,
    std_dev: f64,
    p5: u32,
    p50: u32,
    p95: u32,
    min: u32,
    max: u32,
    pmf: Vec<f64>,
    cdf: Vec<f64>,
}

// ── Engine ──────────────────────────────────────────────────────────────────

fn normalise(pmf: &mut [f64]) {
    let sum: f64 = pmf.iter().sum();
    if sum > 0.0 {
        for p in pmf.iter_mut() {
            *p /= sum;
        }
    }
}

/// The distribution of `a + b`: `result[k] = Σⱼ a[j]·b[k−j]`.
fn convolve(a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; a.len() + b.len() - 1];
    for (i, &ai) in a.iter().enumerate() {
        if ai == 0.0 {
            continue;
        }
        for (j, &bj) in b.iter().enumerate() {
            out[i + j] += ai * bj;
        }
    }
    normalise(&mut out);
    out
}

/// Smallest value `v` with `P(X ≤ v) ≥ p` (a tight percentile from the CDF).
fn percentile(cdf: &[f64], p: f64) -> u32 {
    let eps = 1e-6;
    for (i, &c) in cdf.iter().enumerate() {
        if c >= p - eps {
            return i as u32;
        }
    }
    (cdf.len().saturating_sub(1)) as u32
}

fn analyze(estimate: &Estimate) -> Analysis {
    // Sum of independent components = convolution of their PMFs.
    let mut pmf = vec![1.0]; // delta at 0 — the empty sum
    for entry in &estimate.entries {
        let mut c = entry.component.raw_pmf();
        normalise(&mut c);
        pmf = convolve(&pmf, &c);
    }

    let mut cdf = Vec::with_capacity(pmf.len());
    let mut acc = 0.0;
    for &p in &pmf {
        acc += p;
        cdf.push(acc);
    }

    let mean: f64 = pmf.iter().enumerate().map(|(i, &p)| i as f64 * p).sum();
    let variance: f64 = pmf
        .iter()
        .enumerate()
        .map(|(i, &p)| p * (i as f64 - mean).powi(2))
        .sum();
    let min = pmf.iter().position(|&p| p > 0.0).unwrap_or(0) as u32;
    let max = pmf.iter().rposition(|&p| p > 0.0).unwrap_or(0) as u32;

    Analysis {
        mean,
        std_dev: variance.sqrt(),
        p5: percentile(&cdf, 0.05),
        p50: percentile(&cdf, 0.50),
        p95: percentile(&cdf, 0.95),
        min,
        max,
        pmf,
        cdf,
    }
}

// ── Parsing ─────────────────────────────────────────────────────────────────

/// Everything before an inline `#` comment.
fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(i) => &line[..i],
        None => line,
    }
}

/// Split a leading run of ASCII digits from the rest: `"12 rest"` → `("12", " rest")`.
fn split_number_prefix(s: &str) -> (&str, &str) {
    let end = s
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_digit())
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    (&s[..end], &s[end..])
}

/// The byte index of a `-` that sits between two numbers (spaces allowed).
fn find_range_dash(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'-' {
            let prev = line[..i].chars().rev().find(|c| !c.is_whitespace());
            let next = line[i + 1..].chars().find(|c| !c.is_whitespace());
            if matches!(prev, Some(d) if d.is_ascii_digit())
                && matches!(next, Some(d) if d.is_ascii_digit())
            {
                return Some(i);
            }
        }
    }
    None
}

/// Parse a probability written as `0.25` or `25%`.
fn parse_prob(s: &str) -> Result<f64, String> {
    if let Some(pct) = s.strip_suffix('%') {
        let v: f64 = pct.parse().map_err(|_| format!("invalid percentage '{s}'"))?;
        Ok(v / 100.0)
    } else {
        let v: f64 = s
            .parse()
            .map_err(|_| format!("invalid probability '{s}' (use 0.25 or 25%)"))?;
        if !(0.0..=1.0).contains(&v) {
            return Err(format!("probability {v} is out of range (0 to 1)"));
        }
        Ok(v)
    }
}

/// A number followed by a description: `"5 desc"` → `(5, "desc")`.
fn number_then_desc(s: &str, what: &str) -> Result<(u32, String), String> {
    let (num, rest) = split_number_prefix(s.trim_start());
    if num.is_empty() {
        return Err(format!("expected {what}, got '{}'", s.trim()));
    }
    let value: u32 = num.parse().map_err(|_| format!("'{num}' is not a whole number"))?;
    Ok((value, rest.trim().to_string()))
}

fn parse_component_line(line: &str) -> Result<(Component, String), String> {
    let lower = line.to_lowercase();

    // up to N / upto N / ~N
    for prefix in ["up to ", "upto ", "~"] {
        if lower.starts_with(prefix) {
            let (max, desc) = number_then_desc(&line[prefix.len()..], "a number after 'up to'")?;
            return Ok((Component::UpTo { max }, desc));
        }
    }

    // N or M [@ p] — checked before ranges ("1 or 2" has no dash)
    if let Some(or_pos) = lower.find(" or ") {
        let (either, _) = number_then_desc(&line[..or_pos], "a number before 'or'")?;
        let rest = line[or_pos + 4..].trim_start();
        let (or, after) = number_then_desc(rest, "a number after 'or'")?;
        let (prob_or, desc) = if let Some(tail) = after.trim_start().strip_prefix('@') {
            let tail = tail.trim_start();
            let end = tail
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_digit() || *c == '.' || *c == '%')
                .map(|(i, c)| i + c.len_utf8())
                .last()
                .unwrap_or(0);
            if end == 0 {
                return Err("expected a probability after '@'".into());
            }
            (parse_prob(&tail[..end])?, tail[end..].trim().to_string())
        } else {
            (0.5, after)
        };
        return Ok((Component::EitherOr { either, or, prob_or }, desc));
    }

    // N-M range
    if let Some(dash) = find_range_dash(line) {
        let min: u32 = line[..dash]
            .trim()
            .parse()
            .map_err(|_| format!("expected a number before '-', got '{}'", line[..dash].trim()))?;
        let (max, desc) = number_then_desc(&line[dash + 1..], "a number after '-'")?;
        if min > max {
            return Err(format!("min ({min}) is greater than max ({max})"));
        }
        return Ok((Component::Between { min, max }, desc));
    }

    // Bare N — a fixed (deterministic) value.
    let (value, desc) = number_then_desc(line, "a number or pattern (N-M, up to N, N or M, or N)")?;
    Ok((Component::Between { min: value, max: value }, desc))
}

fn parse(input: &str) -> (Estimate, Vec<ParseIssue>) {
    let mut estimate = Estimate {
        unit: "units".to_string(),
        verb: "take".to_string(),
        entries: Vec::new(),
    };
    let mut issues = Vec::new();

    for line in scan(input) {
        let content = strip_comment(&line.content).trim().to_string();
        if content.is_empty() {
            continue;
        }
        let lower = content.to_lowercase();

        if let Some(val) = lower.strip_prefix("unit:") {
            let val = val.trim();
            if VALID_UNITS.contains(&val) {
                estimate.unit = val.to_string();
            } else {
                issues.push(ParseIssue::error(
                    line.number,
                    format!("unknown unit '{val}'. Try one of: {}", VALID_UNITS.join(", ")),
                ));
            }
            continue;
        }
        if let Some(val) = lower.strip_prefix("verb:") {
            let val = val.trim();
            if VALID_VERBS.contains(&val) {
                estimate.verb = val.to_string();
            } else {
                issues.push(ParseIssue::error(
                    line.number,
                    format!("unknown verb '{val}'. Try one of: {}", VALID_VERBS.join(", ")),
                ));
            }
            continue;
        }

        match parse_component_line(&content) {
            Ok((component, description)) => estimate.entries.push(Entry { description, component }),
            Err(message) => issues.push(ParseIssue::error(line.number, message)),
        }
    }

    if estimate.entries.is_empty() {
        issues.push(ParseIssue::warn(1, "add at least one task, e.g. `4-7 Design`"));
    }

    (estimate, issues)
}

// ── Rendering ───────────────────────────────────────────────────────────────

fn fmt1(value: f64) -> String {
    let value = if value.abs() < 0.05 { 0.0 } else { value };
    let s = format!("{value:.1}");
    s.strip_suffix(".0").unwrap_or(&s).to_string()
}

fn render(estimate: Estimate) -> AnyView {
    if estimate.entries.is_empty() {
        return view! {
            <p class="canvas-empty">
                "Describe each task's uncertainty on the left — a range like "
                <code>"4-7"</code>", " <code>"up to 5"</code>", or " <code>"1 or 2 @ 25%"</code>
                " — and Tyche sums them into one distribution."
            </p>
        }
        .into_any();
    }

    let a = analyze(&estimate);
    let unit = estimate.unit.trim().to_string();
    let unit_suffix = if unit.is_empty() { String::new() } else { format!(" {unit}") };

    let summary = format!(
        "Most likely to {} {}{} · 90% confident {}–{}{} · mean {} ± {}",
        estimate.verb,
        a.p50,
        unit_suffix,
        a.p5,
        a.p95,
        unit_suffix,
        fmt1(a.mean),
        fmt1(a.std_dev),
    );

    // Trim leading/trailing zero-probability tails for a tight chart window.
    let first = a.pmf.iter().position(|&p| p > 1e-9).unwrap_or(0);
    let last = a.pmf.iter().rposition(|&p| p > 1e-9).unwrap_or(a.pmf.len() - 1);
    let start = first.saturating_sub(1);
    let end = (last + 1).min(a.pmf.len() - 1);
    let span = (end - start + 1).max(1);

    let (w, h) = (900.0_f64, 430.0_f64);
    let pl = 56.0_f64;
    let pr = 28.0_f64;
    let pt = 96.0_f64;
    let pb = 66.0_f64;
    let pw = w - pl - pr;
    let ph = h - pt - pb;
    let baseline = pt + ph;

    let max_pmf = a.pmf.iter().cloned().fold(0.0_f64, f64::max).max(1e-9);
    let bar_w = pw / span as f64;
    let gap = (bar_w * 0.2).clamp(1.0, 6.0).min(bar_w * 0.5);
    let center = |i: usize| pl + (i - start) as f64 * bar_w + bar_w / 2.0;

    let mut els: Vec<AnyView> = Vec::new();

    // Summary headline.
    els.push(
        view! {
            <text class="ty-summary" x=w / 2.0 y=34.0 text-anchor="middle">{summary}</text>
        }
        .into_any(),
    );

    // Confidence band (P5–P95) behind the bars.
    if a.p95 >= a.p5 {
        let bx = pl + (a.p5.max(start as u32) as usize - start) as f64 * bar_w;
        let bx2 = pl + (a.p95.min(end as u32) as usize - start + 1) as f64 * bar_w;
        els.push(
            view! {
                <rect class="ty-band" x=bx y=pt width=(bx2 - bx).max(0.0) height=ph />
            }
            .into_any(),
        );
        els.push(
            view! {
                <text class="ty-band-label" x=(bx + bx2) / 2.0 y=pt - 8.0 text-anchor="middle">
                    "90% interval"
                </text>
            }
            .into_any(),
        );
    }

    // Bars: P(total = value).
    for i in start..=end {
        let p = a.pmf[i];
        if p < 1e-12 {
            continue;
        }
        let bh = (p / max_pmf) * ph;
        let x = pl + (i - start) as f64 * bar_w + gap / 2.0;
        els.push(
            view! {
                <rect class="ty-bar" x=x y=baseline - bh width=(bar_w - gap).max(0.5) height=bh rx=2.0 />
            }
            .into_any(),
        );
    }

    // CDF line, P(total ≤ value), scaled to the plot height.
    let cdf_points = (start..=end)
        .map(|i| format!("{:.1},{:.1}", center(i), baseline - a.cdf[i] * ph))
        .collect::<Vec<_>>()
        .join(" ");
    els.push(view! { <polyline class="ty-cdf" points=cdf_points /> }.into_any());

    // Baseline axis.
    els.push(
        view! { <line class="ty-axis" x1=pl y1=baseline x2=w - pr y2=baseline /> }.into_any(),
    );

    // P50 marker.
    let p50_x = center(a.p50 as usize);
    els.push(
        view! { <line class="ty-p50" x1=p50_x y1=pt x2=p50_x y2=baseline /> }.into_any(),
    );
    els.push(
        view! {
            <text class="ty-p50-label" x=p50_x y=pt - 8.0 text-anchor="middle">
                {format!("median {}", a.p50)}
            </text>
        }
        .into_any(),
    );

    // X-axis value labels (~8 ticks).
    let step = (span as f64 / 8.0).ceil().max(1.0) as usize;
    for i in (start..=end).step_by(step) {
        els.push(
            view! {
                <text class="ty-tick" x=center(i) y=baseline + 22.0 text-anchor="middle">
                    {i.to_string()}
                </text>
            }
            .into_any(),
        );
    }
    let axis_title = if unit.is_empty() {
        format!("total ({})", estimate.verb)
    } else {
        format!("total {unit}")
    };
    els.push(
        view! {
            <text class="ty-axis-title" x=w / 2.0 y=baseline + 46.0 text-anchor="middle">
                {axis_title}
            </text>
        }
        .into_any(),
    );

    // Component legend: each task and its range.
    let legend = estimate
        .entries
        .iter()
        .map(|entry| {
            let (lo, hi) = entry.component.range();
            let range = match entry.component {
                Component::EitherOr { either, or, prob_or } => {
                    format!("{either} or {or} @ {:.0}%", prob_or * 100.0)
                }
                _ if lo == hi => lo.to_string(),
                _ => format!("{lo}–{hi}"),
            };
            let label = if entry.description.is_empty() {
                range
            } else {
                format!("{} · {range}", entry.description)
            };
            view! { <span class="ty-legend-item">{label}</span> }
        })
        .collect_view();

    let viewbox = format!("0 0 {w} {h}");
    view! {
        <div class="ty-wrap">
            <svg class="ty-svg" viewBox=viewbox preserveAspectRatio="xMidYMid meet">
                {els.into_iter().collect_view()}
            </svg>
            <div class="ty-legend">{legend}</div>
        </div>
    }
    .into_any()
}

#[component]
pub fn TycheTool() -> impl IntoView {
    let meta = registry::find("tyche").expect("tyche registered");
    let text = crate::ui::use_persisted("tyche", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane
            text=text
            issues=issues
            syntax_hint=HINT
            keywords=&["unit", "verb", "up"]
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

    fn no_errors(issues: &[ParseIssue]) -> bool {
        issues.iter().all(|i| i.severity != Severity::Error)
    }

    #[test]
    fn default_parses_cleanly() {
        let (estimate, issues) = parse(DEFAULT);
        assert!(no_errors(&issues), "unexpected errors parsing the default");
        assert_eq!(estimate.unit, "days");
        assert_eq!(estimate.verb, "take");
        assert_eq!(estimate.entries.len(), 7);
    }

    #[test]
    fn parses_each_component_form() {
        let (e, issues) = parse("4-7 range\nup to 5 upto\n~3 tilde\n1 or 2 either\n2 or 5 @ 25% probbed\n9 fixed\n");
        assert!(no_errors(&issues));
        assert_eq!(e.entries[0].component, Component::Between { min: 4, max: 7 });
        assert_eq!(e.entries[1].component, Component::UpTo { max: 5 });
        assert_eq!(e.entries[2].component, Component::UpTo { max: 3 });
        assert_eq!(e.entries[3].component, Component::EitherOr { either: 1, or: 2, prob_or: 0.5 });
        assert_eq!(e.entries[4].component, Component::EitherOr { either: 2, or: 5, prob_or: 0.25 });
        assert_eq!(e.entries[5].component, Component::Between { min: 9, max: 9 });
        assert_eq!(e.entries[0].description, "range");
    }

    #[test]
    fn inline_comment_is_stripped_from_description() {
        let (e, _) = parse("4-7 Design work  # rough guess\n");
        assert_eq!(e.entries[0].description, "Design work");
    }

    #[test]
    fn bad_lines_report_errors_but_keep_valid_ones() {
        let (e, issues) = parse("4-7 ok\nnonsense here\n3 also ok\n");
        assert_eq!(e.entries.len(), 2);
        assert!(issues.iter().any(|i| i.severity == Severity::Error && i.line == 2));
    }

    #[test]
    fn min_greater_than_max_errors() {
        let (_, issues) = parse("7-4 backwards\n");
        assert!(issues.iter().any(|i| i.message.contains("greater than max")));
    }

    #[test]
    fn unknown_unit_errors_but_keeps_default() {
        let (e, issues) = parse("unit: bananas\n3 Task\n");
        assert!(issues.iter().any(|i| i.message.contains("unknown unit")));
        assert_eq!(e.unit, "units");
        assert_eq!(e.entries.len(), 1);
    }

    #[test]
    fn empty_input_warns() {
        let (_, issues) = parse("");
        assert!(issues.iter().any(|i| i.severity == Severity::Warning));
    }

    #[test]
    fn convolution_of_two_dice_is_triangular() {
        let e = Estimate {
            unit: "pts".into(),
            verb: "score".into(),
            entries: vec![
                Entry { description: String::new(), component: Component::Between { min: 1, max: 6 } },
                Entry { description: String::new(), component: Component::Between { min: 1, max: 6 } },
            ],
        };
        let a = analyze(&e);
        assert!((a.pmf[7] - 6.0 / 36.0).abs() < 1e-12);
        assert!((a.pmf[2] - 1.0 / 36.0).abs() < 1e-12);
        assert_eq!(a.min, 2);
        assert_eq!(a.max, 12);
        assert!((a.mean - 7.0).abs() < 1e-9);
    }

    #[test]
    fn sample_range_and_percentile_order() {
        let (e, _) = parse(DEFAULT);
        let a = analyze(&e);
        // min = 4+2+1+3+0+2+0 = 12, max = 7+3+2+6+5+14+2 = 39
        assert_eq!(a.min, 12);
        assert_eq!(a.max, 39);
        assert!(a.p5 <= a.p50 && a.p50 <= a.p95);
    }
}
