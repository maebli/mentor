//! Artemis — SMART goals: split a goal into its five criteria and grade each
//! one with small, explainable heuristics.
//!
//! The grading is deliberately dumb: word counts, numbers, dates and a few word
//! lists. It can't know whether a goal is *true*, only whether it is written in
//! a way you could later check. Every verdict says which rule fired, so it can
//! be argued with.

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/SMART_criteria";

const HINT: &str =
    "goal <Title>  ·  (indent) specific | measurable | achievable | relevant | time: <text>";

const DEFAULT: &str = "\
# One goal per block. The five lines are graded by heuristics on the right.

goal Cut checkout drop-off
  specific: Rebuild the checkout form as a single page with inline validation
  measurable: Drop-off between cart and payment falls from 34% to under 20%
  achievable: 2 engineers for 6 weeks, reusing the existing payment API
  relevant: Checkout is the largest single loss in the funnel, so that revenue per session rises
  time: behind a flag by 2026-09-30, measured over the 4 weeks after launch

goal Improve the platform
  specific: Make the platform better and more modern
  measurable: Users are happier
  achievable: The team will work hard
  relevant: It is important
  time: soon
";

// ---------------------------------------------------------------- model

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Criterion {
    Specific,
    Measurable,
    Achievable,
    Relevant,
    TimeBound,
}

impl Criterion {
    pub const ALL: [Criterion; 5] = [
        Criterion::Specific,
        Criterion::Measurable,
        Criterion::Achievable,
        Criterion::Relevant,
        Criterion::TimeBound,
    ];

    fn letter(self) -> &'static str {
        match self {
            Criterion::Specific => "S",
            Criterion::Measurable => "M",
            Criterion::Achievable => "A",
            Criterion::Relevant => "R",
            Criterion::TimeBound => "T",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Criterion::Specific => "Specific",
            Criterion::Measurable => "Measurable",
            Criterion::Achievable => "Achievable",
            Criterion::Relevant => "Relevant",
            Criterion::TimeBound => "Time-bound",
        }
    }

    /// The question the criterion is really asking.
    fn prompt(self) -> &'static str {
        match self {
            Criterion::Specific => "what exactly changes?",
            Criterion::Measurable => "how will you know?",
            Criterion::Achievable => "can it be done with what you have?",
            Criterion::Relevant => "why does it matter?",
            Criterion::TimeBound => "by when?",
        }
    }

    /// Accepted keys in the text format, so nobody has to remember one spelling.
    fn from_key(key: &str) -> Option<Criterion> {
        let key = key.trim().to_ascii_lowercase();
        match key.as_str() {
            "s" | "specific" | "what" => Some(Criterion::Specific),
            "m" | "measurable" | "measure" | "metric" => Some(Criterion::Measurable),
            "a" | "achievable" | "attainable" | "realistic" => Some(Criterion::Achievable),
            "r" | "relevant" | "why" => Some(Criterion::Relevant),
            "t" | "time" | "timebound" | "time-bound" | "timely" | "deadline" | "by" => {
                Some(Criterion::TimeBound)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Grade {
    Missing,
    Weak,
    Strong,
}

impl Grade {
    fn points(self) -> u32 {
        match self {
            Grade::Missing => 0,
            Grade::Weak => 1,
            Grade::Strong => 2,
        }
    }

    fn class(self) -> &'static str {
        match self {
            Grade::Missing => "sm-missing",
            Grade::Weak => "sm-weak",
            Grade::Strong => "sm-strong",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Goal {
    pub title: String,
    pub line: usize,
    /// Criterion text in the order it was written; at most one entry each.
    pub parts: Vec<(Criterion, String)>,
}

impl Goal {
    fn get(&self, criterion: Criterion) -> Option<&str> {
        self.parts
            .iter()
            .find(|(c, _)| *c == criterion)
            .map(|(_, text)| text.as_str())
    }
}

#[derive(Clone, PartialEq)]
pub struct Plan {
    pub goals: Vec<Goal>,
}

/// Parse `goal` blocks with indented `criterion: text` lines.
pub fn parse(input: &str) -> (Plan, Vec<ParseIssue>) {
    let mut plan = Plan { goals: Vec::new() };
    let mut issues = Vec::new();
    let mut current: Option<usize> = None;

    for line in scan(input) {
        match line.indent {
            0 => {
                current = None;
                let (keyword, rest) = line.keyword();
                if !keyword.eq_ignore_ascii_case("goal") {
                    issues.push(ParseIssue::error(line.number, "expected `goal <Title>`"));
                    continue;
                }
                if rest.is_empty() {
                    issues.push(ParseIssue::error(line.number, "a goal needs a title"));
                    continue;
                }
                plan.goals.push(Goal {
                    title: rest.to_string(),
                    line: line.number,
                    parts: Vec::new(),
                });
                current = Some(plan.goals.len() - 1);
            }
            1 => {
                if !line.content.contains(':') {
                    issues.push(ParseIssue::error(
                        line.number,
                        "expected `<criterion>: <text>`",
                    ));
                    continue;
                }
                let (key, value) = line.colon();
                let Some(goal_index) = current else {
                    issues.push(ParseIssue::error(
                        line.number,
                        "a criterion must sit under a goal",
                    ));
                    continue;
                };
                let Some(criterion) = Criterion::from_key(key) else {
                    issues.push(ParseIssue::warn(
                        line.number,
                        format!("“{key}” is not one of specific / measurable / achievable / relevant / time"),
                    ));
                    continue;
                };
                if value.is_empty() {
                    issues.push(ParseIssue::error(
                        line.number,
                        format!("{} needs some text — {}", criterion.label(), criterion.prompt()),
                    ));
                    continue;
                }

                let parts = &mut plan.goals[goal_index].parts;
                if let Some((_, existing)) = parts.iter_mut().find(|(c, _)| *c == criterion) {
                    issues.push(ParseIssue::warn(
                        line.number,
                        format!("duplicate {} (last wins)", criterion.label()),
                    ));
                    *existing = value.to_string();
                } else {
                    parts.push((criterion, value.to_string()));
                }
            }
            _ => issues.push(ParseIssue::error(
                line.number,
                "expected `<criterion>: <text>` at one level of indent",
            )),
        }
    }

    for goal in &plan.goals {
        for criterion in Criterion::ALL {
            if goal.get(criterion).is_none() {
                issues.push(ParseIssue::warn(
                    goal.line,
                    format!(
                        "“{}” has no {} — {}",
                        goal.title,
                        criterion.label(),
                        criterion.prompt()
                    ),
                ));
            }
        }
    }

    (plan, issues)
}

// ------------------------------------------------------------ heuristics

/// Words that promise something without saying what.
const VAGUE: &[&str] = &[
    "improve", "improved", "better", "best", "optimise", "optimize", "enhance", "robust",
    "scalable", "efficient", "quality", "modern", "modernise", "modernize", "seamless",
    "streamline", "leverage", "synergy", "world-class", "some", "various", "several", "stuff",
    "things", "nice", "good", "great", "faster", "cleaner", "asap", "soon", "etc",
];

/// Claims with no margin in them — usually a sign nobody costed the goal.
const ABSOLUTES: &[&str] = &[
    "all", "every", "everyone", "everything", "always", "never", "zero", "none", "perfect",
    "fully", "completely", "100", "guaranteed", "any",
];

/// Evidence that the goal is grounded in something that already exists.
const PRECEDENT: &[&str] = &[
    "existing", "already", "reuse", "reusing", "reused", "proven", "precedent", "in place",
    "last time", "spare", "slack", "budgeted", "approved", "funded",
];

/// Words that tie a goal to something bigger than itself.
const LINK: &[&str] = &[
    "because", "so that", "in order to", "unblocks", "unblock", "enables", "enable", "supports",
    "drives", "aligns", "target", "goal", "okr", "strategy", "revenue", "churn", "retention",
    "cost", "costs", "customer", "customers", "user", "users", "risk", "compliance", "required",
    "mandate", "funnel", "growth", "reliability", "safety", "deadline",
];

const DURATIONS: &[&str] = &[
    "day", "days", "week", "weeks", "month", "months", "quarter", "quarters", "sprint", "sprints",
    "year", "years",
];

/// Month names. "May" is deliberately absent — as a token it is far more often
/// the hedge than the month.
const MONTHS: &[&str] = &[
    "january", "february", "march", "april", "june", "july", "august", "september", "october",
    "november", "december", "jan", "feb", "mar", "apr", "jun", "jul", "aug", "sep", "sept", "oct",
    "nov", "dec",
];

/// Lowercased alphanumeric tokens, so `100%` matches `100` and `handsome` never
/// matches `some`.
fn tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Which of `needles` appear in `text`: single words match whole tokens,
/// multi-word needles match as substrings.
fn hits<'a>(text: &str, needles: &[&'a str]) -> Vec<&'a str> {
    let toks = tokens(text);
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .copied()
        .filter(|needle| {
            if needle.contains(' ') || needle.contains('-') {
                lower.contains(needle)
            } else {
                toks.iter().any(|t| t == needle)
            }
        })
        .collect()
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().filter(|w| !w.is_empty()).count()
}

fn has_number(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_digit())
}

/// A four-digit token that looks like a calendar year.
fn has_year(text: &str) -> bool {
    tokens(text).iter().any(|t| {
        t.len() == 4 && t.chars().all(|c| c.is_ascii_digit()) && (t.starts_with('1') || t.starts_with('2'))
    })
}

fn has_quarter(text: &str) -> bool {
    tokens(text).iter().any(|t| {
        t.len() == 2 && t.starts_with('q') && matches!(t.as_bytes()[1], b'1'..=b'4')
    })
}

#[derive(Clone, PartialEq)]
pub struct Check {
    pub criterion: Criterion,
    pub text: String,
    pub grade: Grade,
    /// Why it got that grade — always names the rule that fired.
    pub note: String,
}

fn join(list: &[&str]) -> String {
    list.iter()
        .map(|w| format!("“{w}”"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn grade(criterion: Criterion, text: &str) -> (Grade, String) {
    match criterion {
        Criterion::Specific => {
            let vague = hits(text, VAGUE);
            if word_count(text) < 4 {
                (
                    Grade::Weak,
                    "too short to pin anything down — name what changes, and where".into(),
                )
            } else if !vague.is_empty() {
                (
                    Grade::Weak,
                    format!("vague wording: {} — say what will actually be different", join(&vague)),
                )
            } else {
                (Grade::Strong, "concrete: a reader could picture the change".into())
            }
        }
        Criterion::Measurable => {
            if !has_number(text) {
                (
                    Grade::Weak,
                    "no number — what value, count or rate would prove it?".into(),
                )
            } else if !hits(text, &["from", "today", "currently", "baseline", "now"]).is_empty() {
                (Grade::Strong, "a number with a baseline to move from".into())
            } else {
                (Grade::Strong, "a number you can check against".into())
            }
        }
        Criterion::Achievable => {
            let absolutes = hits(text, ABSOLUTES);
            if !absolutes.is_empty() {
                (
                    Grade::Weak,
                    format!("absolute claim: {} — leave yourself a margin", join(&absolutes)),
                )
            } else if !has_number(text) && hits(text, PRECEDENT).is_empty() {
                (
                    Grade::Weak,
                    "no capacity named — how many people, for how long, on top of what?".into(),
                )
            } else {
                (Grade::Strong, "grounded in real capacity or precedent".into())
            }
        }
        Criterion::Relevant => {
            let link = hits(text, LINK);
            if word_count(text) < 4 {
                (
                    Grade::Weak,
                    "too thin — finish the sentence “…so that …”".into(),
                )
            } else if link.is_empty() {
                (
                    Grade::Weak,
                    "no link to anything bigger — what does it unblock or protect?".into(),
                )
            } else {
                (
                    Grade::Strong,
                    format!("tied to something that matters ({})", join(&link[..1])),
                )
            }
        }
        Criterion::TimeBound => {
            if has_year(text) || has_quarter(text) || !hits(text, MONTHS).is_empty() {
                (Grade::Strong, "anchored to a date on the calendar".into())
            } else if has_number(text) && !hits(text, DURATIONS).is_empty() {
                (
                    Grade::Weak,
                    "a duration, not a date — from when? anchor it to a calendar date".into(),
                )
            } else {
                (Grade::Weak, "no date — when exactly is it due?".into())
            }
        }
    }
}

/// Grade all five criteria of a goal, in SMART order.
pub fn check(goal: &Goal) -> Vec<Check> {
    Criterion::ALL
        .iter()
        .map(|criterion| match goal.get(*criterion) {
            Some(text) => {
                let (grade, note) = grade(*criterion, text);
                Check {
                    criterion: *criterion,
                    text: text.to_string(),
                    grade,
                    note,
                }
            }
            None => Check {
                criterion: *criterion,
                text: String::new(),
                grade: Grade::Missing,
                note: format!("missing — {}", criterion.prompt()),
            },
        })
        .collect()
}

/// 0–10: two points per criterion that holds up, one for a weak one.
pub fn score(checks: &[Check]) -> u32 {
    checks.iter().map(|c| c.grade.points()).sum()
}

fn verdict(score: u32) -> &'static str {
    match score {
        9..=10 => "sharp",
        6..=8 => "workable",
        3..=5 => "fuzzy",
        _ => "a wish",
    }
}

// ---------------------------------------------------------------- render

fn goal_card(goal: &Goal) -> AnyView {
    let checks = check(goal);
    let total = score(&checks);

    let strip = checks
        .iter()
        .map(|c| {
            let class = format!("sm-letter {}", c.grade.class());
            let title = format!("{}: {}", c.criterion.label(), c.note);
            view! { <span class=class title=title>{c.criterion.letter()}</span> }
        })
        .collect_view();

    let rows = checks
        .iter()
        .map(|c| {
            let class = format!("sm-row {}", c.grade.class());
            let text = if c.text.is_empty() {
                "—".to_string()
            } else {
                c.text.clone()
            };
            view! {
                <li class=class>
                    <span class="sm-key">{c.criterion.label()}</span>
                    <span class="sm-val">{text}</span>
                    <span class="sm-note">{c.note.clone()}</span>
                </li>
            }
        })
        .collect_view();

    view! {
        <article class="sm-card">
            <header class="sm-head">
                <h3 class="sm-title">{goal.title.clone()}</h3>
                <span class="sm-strip">{strip}</span>
                <span class="sm-score">
                    {format!("{total}/10")}
                    <span class="sm-verdict">{verdict(total)}</span>
                </span>
            </header>
            <ul class="sm-rows">{rows}</ul>
        </article>
    }
    .into_any()
}

fn render(plan: Plan) -> AnyView {
    if plan.goals.is_empty() {
        return view! {
            <p class="canvas-empty">"Write a `goal` on the left, then its five SMART lines."</p>
        }
        .into_any();
    }

    let cards = plan.goals.iter().map(goal_card).collect_view();

    view! {
        <div class="sm-board">
            {cards}
            <p class="sm-foot">
                "Heuristics only: this checks how the goal is "
                <em>"written"</em>
                " — numbers, dates, hedges — not whether it is the right goal."
            </p>
        </div>
    }
    .into_any()
}

#[component]
pub fn ArtemisTool() -> impl IntoView {
    let meta = registry::find("artemis").expect("artemis registered");
    let text = crate::ui::use_persisted("artemis", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane
            text=text
            issues=issues
            syntax_hint=HINT
            keywords=&["goal"]
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

    fn goal_of(text: &str) -> Goal {
        let (plan, _) = parse(text);
        plan.goals.into_iter().next().expect("one goal")
    }

    #[test]
    fn default_parses_without_issues() {
        let (plan, issues) = parse(DEFAULT);
        assert_eq!(plan.goals.len(), 2);
        assert!(issues.is_empty(), "default should be structurally clean");
    }

    #[test]
    fn default_scores_a_sharp_goal_and_a_fuzzy_one() {
        let (plan, _) = parse(DEFAULT);
        assert_eq!(score(&check(&plan.goals[0])), 10);
        assert_eq!(score(&check(&plan.goals[1])), 5);
        assert_eq!(verdict(10), "sharp");
        assert_eq!(verdict(5), "fuzzy");
    }

    #[test]
    fn missing_criterion_warns_and_scores_zero_for_it() {
        let (_, issues) = parse("goal Ship it\n  specific: Move the parser behind a trait\n");
        assert!(issues
            .iter()
            .any(|i| i.message.contains("has no Measurable")));

        let goal = goal_of("goal Ship it\n  specific: Move the parser behind a trait\n");
        let checks = check(&goal);
        assert_eq!(checks[1].grade, Grade::Missing);
    }

    #[test]
    fn unknown_key_warns() {
        let (_, issues) = parse("goal Ship it\n  sparkly: yes\n");
        assert!(issues.iter().any(|i| i.message.contains("is not one of")));
    }

    #[test]
    fn aliases_map_to_the_same_criteria() {
        let goal = goal_of("goal G\n  m: 40% fewer pages\n  by: 2027-01-15\n");
        assert_eq!(goal.parts[0].0, Criterion::Measurable);
        assert_eq!(goal.parts[1].0, Criterion::TimeBound);
    }

    #[test]
    fn measurable_needs_a_number() {
        assert_eq!(grade(Criterion::Measurable, "users are happier").0, Grade::Weak);
        assert_eq!(grade(Criterion::Measurable, "p95 latency under 200ms").0, Grade::Strong);
    }

    #[test]
    fn time_wants_a_date_not_a_duration() {
        assert_eq!(grade(Criterion::TimeBound, "by 2026-09-30").0, Grade::Strong);
        assert_eq!(grade(Criterion::TimeBound, "end of Q3").0, Grade::Strong);
        assert_eq!(grade(Criterion::TimeBound, "in about 6 weeks").0, Grade::Weak);
        assert_eq!(grade(Criterion::TimeBound, "soon").0, Grade::Weak);
    }

    #[test]
    fn vague_words_only_match_whole_words() {
        // "handsome" must not trip the "some" rule.
        assert_eq!(
            grade(Criterion::Specific, "Rewrite the handsome onboarding flow in Rust").0,
            Grade::Strong
        );
        assert_eq!(
            grade(Criterion::Specific, "Make the onboarding flow better for users").0,
            Grade::Weak
        );
    }

    #[test]
    fn achievable_rejects_absolutes_and_hand_waving() {
        assert_eq!(grade(Criterion::Achievable, "we will migrate every service").0, Grade::Weak);
        assert_eq!(grade(Criterion::Achievable, "the team will work hard").0, Grade::Weak);
        assert_eq!(
            grade(Criterion::Achievable, "2 engineers for 6 weeks on the existing API").0,
            Grade::Strong
        );
    }

    #[test]
    fn relevant_wants_a_connection() {
        assert_eq!(grade(Criterion::Relevant, "it is important").0, Grade::Weak);
        assert_eq!(
            grade(Criterion::Relevant, "it is the last blocker, so that the launch can ship").0,
            Grade::Strong
        );
    }
}
