//! Cetus — Ishikawa (fishbone) cause-and-effect diagram.
//!
//! The effect sits at the fish's head on the right; category "bones" branch off
//! a horizontal spine, and causes hang off each bone. Follows the house pattern:
//! a pure `parse` (unit-tested), a `render` that emits an inline SVG, and a
//! `*Tool` component wiring them together.
//!
//! Text format (2-space indentation):
//! ```text
//! effect: Checkout page loads slowly
//! category Code
//!   cause: N+1 database queries
//!     cause: missing eager loading
//!   cause: unbounded response payloads
//! ```

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Ishikawa_diagram";

const HINT: &str =
    "effect: <problem>  ·  category <Name>  ·  (indent) cause: <text>  ·  (indent×2) cause: <sub-cause>";

const DEFAULT: &str = "\
effect: Checkout page loads slowly
category Code
  cause: N+1 database queries
    cause: missing eager loading
  cause: unbounded response payloads
category Infrastructure
  cause: undersized app instances
  cause: no CDN for static assets
category Data
  cause: missing index on orders table
  cause: stale cache invalidation
category Process
  cause: no performance budget in CI
  cause: load testing skipped before release
";

#[derive(Clone, PartialEq)]
pub struct Cause {
    pub text: String,
    pub sub: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub struct Category {
    pub name: String,
    pub causes: Vec<Cause>,
}

#[derive(Clone, PartialEq)]
pub struct Fishbone {
    pub effect: String,
    pub categories: Vec<Category>,
}

/// Parse fishbone text into an effect + categories + causes, reporting issues.
pub fn parse(input: &str) -> (Fishbone, Vec<ParseIssue>) {
    let mut fb = Fishbone { effect: String::new(), categories: Vec::new() };
    let mut issues: Vec<ParseIssue> = Vec::new();
    let mut effect_set = false;

    for line in scan(input) {
        // Normalise the leading token: `effect:` / `cause:` -> `effect` / `cause`.
        let (raw_kw, _) = line.keyword();
        let kw = raw_kw.trim_end_matches(':');

        match line.indent {
            0 => match kw {
                "effect" => {
                    let (_, rest) = line.colon();
                    if effect_set {
                        issues.push(ParseIssue::warn(line.number, "only the first effect is used"));
                    } else {
                        effect_set = true;
                        if rest.is_empty() {
                            issues.push(ParseIssue::error(
                                line.number,
                                "add an `effect:` — the problem at the fish head",
                            ));
                        } else {
                            fb.effect = rest.to_string();
                        }
                    }
                }
                "category" => {
                    let (_, name) = line.keyword();
                    if name.is_empty() {
                        issues.push(ParseIssue::error(line.number, "`category` needs a name"));
                    } else {
                        fb.categories.push(Category { name: name.to_string(), causes: Vec::new() });
                    }
                }
                _ => issues.push(ParseIssue::error(
                    line.number,
                    "expected `effect:` or `category <Name>`",
                )),
            },
            1 => {
                if kw != "cause" {
                    issues.push(ParseIssue::error(line.number, "expected `cause: <text>`"));
                    continue;
                }
                let (_, text) = line.colon();
                let Some(cat) = fb.categories.last_mut() else {
                    issues.push(ParseIssue::error(line.number, "cause must sit under a category"));
                    continue;
                };
                if text.is_empty() {
                    issues.push(ParseIssue::warn(line.number, "empty cause"));
                } else {
                    cat.causes.push(Cause { text: text.to_string(), sub: Vec::new() });
                }
            }
            2 => {
                if kw != "cause" {
                    issues.push(ParseIssue::error(line.number, "expected `cause: <text>`"));
                    continue;
                }
                let (_, text) = line.colon();
                let cause = fb.categories.last_mut().and_then(|c| c.causes.last_mut());
                let Some(cause) = cause else {
                    issues.push(ParseIssue::error(line.number, "sub-cause has no parent cause"));
                    continue;
                };
                if !text.is_empty() {
                    cause.sub.push(text.to_string());
                }
            }
            _ => issues.push(ParseIssue::warn(
                line.number,
                "sub-causes deeper than one level are ignored",
            )),
        }
    }

    if !effect_set {
        issues.push(ParseIssue::error(1, "add an `effect:` — the problem at the fish head"));
    }

    (fb, issues)
}

/// Wrap `text` to at most `cols` characters per line, at most `max` lines, with
/// an ellipsis if it overflows. Word-based so it doesn't cut mid-word.
fn wrap(text: &str, cols: usize, max: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.len() + 1 + word.len() <= cols {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
            if lines.len() == max {
                break;
            }
        }
    }
    if lines.len() < max && !cur.is_empty() {
        lines.push(cur);
    }
    if lines.len() == max {
        if let Some(last) = lines.last_mut() {
            if last.len() > cols.saturating_sub(1) {
                last.truncate(cols.saturating_sub(1));
                last.push('…');
            }
        }
    }
    lines
}

fn render(fb: Fishbone) -> AnyView {
    if fb.effect.is_empty() && fb.categories.is_empty() {
        return view! {
            <p class="canvas-empty">"Add an effect and some categories on the left."</p>
        }
        .into_any();
    }

    // Canvas geometry.
    let w = 940.0_f64;
    let spine_y = 250.0_f64;
    let h = 500.0_f64;
    let spine_start = 40.0_f64;
    let spine_end = 720.0_f64;
    let ex = 730.0_f64;
    let ew = 195.0_f64;
    let eh = 90.0_f64;

    let mut els: Vec<AnyView> = Vec::new();

    // Spine + arrowhead into the effect box.
    els.push(
        view! {
            <line class="fb-spine" x1=spine_start y1=spine_y x2=spine_end y2=spine_y />
        }
        .into_any(),
    );
    let ah = format!(
        "{},{} {},{} {},{}",
        spine_end,
        spine_y - 9.0,
        spine_end + 16.0,
        spine_y,
        spine_end,
        spine_y + 9.0
    );
    els.push(view! { <polygon class="fb-arrow" points=ah /> }.into_any());

    // Effect box at the head.
    els.push(
        view! {
            <rect class="fb-effect" x=ex y=spine_y - eh / 2.0 width=ew height=eh rx="10" />
        }
        .into_any(),
    );
    let effect_lines = wrap(&fb.effect, 22, 3);
    let n_lines = effect_lines.len().max(1);
    for (i, l) in effect_lines.into_iter().enumerate() {
        let ty = spine_y - (n_lines as f64 - 1.0) * 9.0 + i as f64 * 18.0 + 5.0;
        els.push(
            view! {
                <text class="fb-effect-text" x=ex + ew / 2.0 y=ty text-anchor="middle">
                    {l}
                </text>
            }
            .into_any(),
        );
    }

    // Category bones, alternating above / below the spine.
    let n = fb.categories.len().max(1);
    let x_first = 180.0_f64;
    let x_last = 660.0_f64;
    let step = if n > 1 { (x_last - x_first) / (n as f64 - 1.0) } else { 0.0 };
    let bone_dx = 130.0_f64;
    let bone_dy = 175.0_f64;

    for (i, cat) in fb.categories.iter().enumerate() {
        let above = i % 2 == 0;
        let dir = if above { -1.0 } else { 1.0 };
        let anchor_x = if n > 1 { x_first + i as f64 * step } else { (x_first + x_last) / 2.0 };
        let end_x = anchor_x - bone_dx;
        let end_y = spine_y + dir * bone_dy;

        // The bone.
        els.push(
            view! {
                <line class="fb-bone" x1=anchor_x y1=spine_y x2=end_x y2=end_y />
            }
            .into_any(),
        );

        // Category label box at the outer end of the bone.
        let label_w = (cat.name.len() as f64 * 7.4 + 16.0).max(46.0);
        let label_h = 24.0;
        let lx = end_x - label_w / 2.0;
        let ly = end_y + dir * 4.0 - label_h / 2.0 + dir * 6.0;
        els.push(
            view! {
                <rect class="fb-cat" x=lx y=ly width=label_w height=label_h rx="6" />
            }
            .into_any(),
        );
        els.push(
            view! {
                <text
                    class="fb-cat-text"
                    x=end_x
                    y=ly + label_h / 2.0 + 4.0
                    text-anchor="middle"
                >
                    {cat.name.clone()}
                </text>
            }
            .into_any(),
        );

        // Causes stepping along the bone from the spine outward.
        let m = cat.causes.len().max(1);
        for (k, cause) in cat.causes.iter().enumerate() {
            let t = (k as f64 + 1.0) / (m as f64 + 1.0);
            let px = anchor_x + t * (end_x - anchor_x);
            let py = spine_y + t * (end_y - spine_y);
            let ty = py + dir * -6.0;
            els.push(
                view! {
                    <text class="fb-cause" x=px + 10.0 y=ty text-anchor="start">
                        {cause.text.clone()}
                    </text>
                }
                .into_any(),
            );
            for (s, sub) in cause.sub.iter().enumerate() {
                let sy = ty + (s as f64 + 1.0) * 13.0 * if above { 1.0 } else { 1.0 };
                els.push(
                    view! {
                        <text class="fb-sub" x=px + 22.0 y=sy text-anchor="start">
                            {format!("– {sub}")}
                        </text>
                    }
                    .into_any(),
                );
            }
        }
    }

    let viewbox = format!("0 0 {w} {h}");
    view! {
        <svg class="fishbone" viewBox=viewbox preserveAspectRatio="xMidYMid meet">
            {els.into_iter().collect_view()}
        </svg>
    }
    .into_any()
}

#[component]
pub fn CetusTool() -> impl IntoView {
    let meta = registry::find("cetus").expect("cetus registered");
    let text = crate::ui::use_persisted("cetus", DEFAULT);
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
    fn parses_default_effect_and_categories() {
        let (fb, issues) = parse(DEFAULT);
        assert_eq!(fb.effect, "Checkout page loads slowly");
        assert_eq!(fb.categories.len(), 4);
        assert_eq!(fb.categories[0].name, "Code");
        assert_eq!(fb.categories[0].causes[0].sub, vec!["missing eager loading"]);
        assert!(issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn errors_when_effect_missing() {
        let (_, issues) = parse("category Code\n  cause: x\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("effect")));
    }

    #[test]
    fn errors_on_cause_without_category() {
        let (_, issues) = parse("effect: slow\n  cause: orphan\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("under a category")));
    }
}
