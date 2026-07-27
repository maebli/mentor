//! Psyche — the sixteen personality types on a spider chart, plus the
//! generational cohorts, plotted in the same space so they can be compared.
//!
//! Two honest caveats are built into the page rather than hidden in a footnote:
//!
//! * The four-letter type is a *vocabulary*, not a measurement. So the shape a
//!   type draws here is derived straight from its own letters (E/I, N/S, F/T,
//!   P/J) — no invented trait scores. Identity (Assertive–Turbulent) is not
//!   fixed by the four letters, so every type sits on the midline there.
//! * The generational profiles are indicative tendencies drawn from survey and
//!   cohort research, not personality measurements, and the spread inside a
//!   cohort dwarfs the gap between cohorts.

use crate::registry;
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Myers%E2%80%93Briggs_Type_Indicator";

/// How many profiles can be overlaid before the chart turns to spaghetti.
const MAX_SERIES: usize = 4;

const DEFAULT_SELECTION: &str = "INTJ,gen-z";

// ------------------------------------------------------------------ axes

/// One radar axis: the centre is `low`, the outer edge is `high`.
struct Axis {
    dimension: &'static str,
    low: &'static str,
    high: &'static str,
}

const AXES: [Axis; 5] = [
    Axis { dimension: "Mind", low: "Introverted", high: "Extraverted" },
    Axis { dimension: "Energy", low: "Observant", high: "Intuitive" },
    Axis { dimension: "Nature", low: "Thinking", high: "Feeling" },
    Axis { dimension: "Tactics", low: "Judging", high: "Prospecting" },
    Axis { dimension: "Identity", low: "Assertive", high: "Turbulent" },
];

// ------------------------------------------------------------- the types

struct Role {
    name: &'static str,
    letters: &'static str,
    gist: &'static str,
}

const ROLES: [Role; 4] = [
    Role {
        name: "Analysts",
        letters: "N + T",
        gist: "Drawn to systems, models and competence.",
    },
    Role {
        name: "Diplomats",
        letters: "N + F",
        gist: "Drawn to meaning, people and potential.",
    },
    Role {
        name: "Sentinels",
        letters: "S + J",
        gist: "Drawn to order, duty and continuity.",
    },
    Role {
        name: "Explorers",
        letters: "S + P",
        gist: "Drawn to action, craft and the present moment.",
    },
];

struct Type16 {
    code: &'static str,
    name: &'static str,
    role: &'static str,
    /// Estimated share of the US population (MBTI Manual / CAPT estimates).
    share: &'static str,
    tldr: &'static str,
    at_work: &'static str,
    watch: &'static str,
}

static TYPES: &[Type16] = &[
    // --- Analysts ---
    Type16 {
        code: "INTJ",
        name: "Architect",
        role: "Analysts",
        share: "~2.1%",
        tldr: "Builds a long-range plan, then quietly executes it.",
        at_work: "Wants the strategy coherent before work starts; will redesign a system rather than keep patching it.",
        watch: "Can file other people's objections as noise when they arrive without a model attached.",
    },
    Type16 {
        code: "INTP",
        name: "Logician",
        role: "Analysts",
        share: "~3.3%",
        tldr: "Takes ideas apart to find out what is actually true.",
        at_work: "Excellent at finding the flaw in a design; least interested in the last 20% of shipping it.",
        watch: "The analysis can quietly become the deliverable.",
    },
    Type16 {
        code: "ENTJ",
        name: "Commander",
        role: "Analysts",
        share: "~1.8%",
        tldr: "Organises people and resources around a goal, fast.",
        at_work: "Sets direction, clears blockers, expects a decision rather than a discussion.",
        watch: "Speed can flatten the dissent that was worth hearing.",
    },
    Type16 {
        code: "ENTP",
        name: "Debater",
        role: "Analysts",
        share: "~3.2%",
        tldr: "Generates options and stress-tests them out loud.",
        at_work: "Great at kicking things off and reframing them; needs someone beside them who closes.",
        watch: "Argues for sport, and may reopen decisions that were already made.",
    },
    // --- Diplomats ---
    Type16 {
        code: "INFJ",
        name: "Advocate",
        role: "Diplomats",
        share: "~1.5%",
        tldr: "Works from a quiet but strongly held sense of what things are for.",
        at_work: "Reads the room, defends the user, plans in private and then arrives with a view.",
        watch: "Holds concerns back until they come out all at once as a hard line.",
    },
    Type16 {
        code: "INFP",
        name: "Mediator",
        role: "Diplomats",
        share: "~4.4%",
        tldr: "Measures the work against personal values.",
        at_work: "Cares that the thing is good, not just done; usually writes and explains well.",
        watch: "Avoiding conflict can look, from outside, exactly like agreement.",
    },
    Type16 {
        code: "ENFJ",
        name: "Protagonist",
        role: "Diplomats",
        share: "~2.5%",
        tldr: "Brings people with them toward a shared goal.",
        at_work: "Natural facilitator and coach; keeps the team's morale inside the plan.",
        watch: "May carry everyone else's load until their own work slips.",
    },
    Type16 {
        code: "ENFP",
        name: "Campaigner",
        role: "Diplomats",
        share: "~8.1%",
        tldr: "Sees possibility in everything, and in everyone.",
        at_work: "Energises kick-offs and connects ideas nobody had put side by side.",
        watch: "Enthusiasm outruns follow-through once the novelty wears off.",
    },
    // --- Sentinels ---
    Type16 {
        code: "ISTJ",
        name: "Logistician",
        role: "Sentinels",
        share: "~11.6%",
        tldr: "Does what was agreed, exactly, on time.",
        at_work: "The spine of a team: keeps the records and notices the drift from the standard.",
        watch: "Treats “we have always done it this way” as though it were evidence.",
    },
    Type16 {
        code: "ISFJ",
        name: "Defender",
        role: "Sentinels",
        share: "~13.8%",
        tldr: "Looks after the people and the details everyone else forgets.",
        at_work: "Quiet continuity — onboarding, handovers, the unglamorous glue of delivery.",
        watch: "Absorbs far too much work without ever mentioning it.",
    },
    Type16 {
        code: "ESTJ",
        name: "Executive",
        role: "Sentinels",
        share: "~8.7%",
        tldr: "Puts structure on a messy situation and drives it to done.",
        at_work: "Clear ownership, clear dates, visible progress.",
        watch: "The process can outlive the problem that justified it.",
    },
    Type16 {
        code: "ESFJ",
        name: "Consul",
        role: "Sentinels",
        share: "~12.3%",
        tldr: "Keeps the group working, and keeps it looked after.",
        at_work: "Coordination and communication; the person who notices someone has gone quiet.",
        watch: "Harmony gets protected at the price of an honest disagreement.",
    },
    // --- Explorers ---
    Type16 {
        code: "ISTP",
        name: "Virtuoso",
        role: "Explorers",
        share: "~5.4%",
        tldr: "Understands a system by taking it apart.",
        at_work: "Calm in an incident; fixes the real thing, fast, with very little ceremony.",
        watch: "Skips the write-up that would have let anyone else fix it next time.",
    },
    Type16 {
        code: "ISFP",
        name: "Adventurer",
        role: "Explorers",
        share: "~8.8%",
        tldr: "Works quietly and concretely, with an eye for craft.",
        at_work: "Hands-on quality, especially anywhere the work is seen or touched.",
        watch: "Goes quiet instead of fighting for their own view.",
    },
    Type16 {
        code: "ESTP",
        name: "Entrepreneur",
        role: "Explorers",
        share: "~4.3%",
        tldr: "Acts now and adjusts on contact with reality.",
        at_work: "Unblocks stuck situations; thrives on urgency and negotiation.",
        watch: "Keeps shipping past the point where stopping to think was cheaper.",
    },
    Type16 {
        code: "ESFP",
        name: "Entertainer",
        role: "Explorers",
        share: "~8.5%",
        tldr: "Brings energy, and reads the room in real time.",
        at_work: "Makes demos, customers and teams come alive.",
        watch: "Detail and long horizons need scaffolding from someone else.",
    },
];

/// The shape a type draws: taken straight from its own letters, so nothing is
/// invented. Identity is not encoded in the four letters, so it sits at 50.
fn axes_of(code: &str) -> [f64; 5] {
    let letters: Vec<char> = code.chars().collect();
    let toward = |c: char, high: char| if c == high { 80.0 } else { 20.0 };
    [
        toward(letters[0], 'E'),
        toward(letters[1], 'N'),
        toward(letters[2], 'F'),
        toward(letters[3], 'P'),
        50.0,
    ]
}

fn find_type(code: &str) -> Option<&'static Type16> {
    TYPES.iter().find(|t| t.code.eq_ignore_ascii_case(code))
}

// ------------------------------------------------------------ generations

struct Generation {
    key: &'static str,
    label: &'static str,
    years: &'static str,
    /// What shaped the cohort.
    formative: &'static str,
    /// How it tends to show up at work.
    at_work: &'static str,
    /// The stereotype, and what is actually behind it.
    myth: &'static str,
    /// Indicative tendencies on the same five axes — see the caveat on the page.
    axes: [f64; 5],
}

static GENERATIONS: &[Generation] = &[
    Generation {
        key: "silent",
        label: "Silent Generation",
        years: "1928–1945",
        formative: "Depression-era childhoods and post-war reconstruction; institutions you joined once and stayed in.",
        at_work: "Almost entirely retired. Where still present: values hierarchy, discretion, and getting on with it without fuss.",
        myth: "“Rigid” mostly describes the incentives they were given — loyalty and process were what got rewarded for forty years.",
        axes: [45.0, 40.0, 45.0, 25.0, 50.0],
    },
    Generation {
        key: "boomers",
        label: "Baby Boomers",
        years: "1946–1964",
        formative: "Post-war growth, mass media, and careers built inside one or two organisations.",
        at_work: "Prefers the meeting or the phone call to the thread; reads title and tenure as real signals; long-horizon loyalty.",
        myth: "“Tech-averse” tracks exposure far better than ability — this cohort adopted every tool their jobs actually required.",
        axes: [55.0, 45.0, 45.0, 33.0, 55.0],
    },
    Generation {
        key: "gen-x",
        label: "Gen X",
        years: "1965–1980",
        formative: "Latchkey childhoods, recessions, the arrival of the PC and the early web — and employers who laid people off anyway.",
        at_work: "Self-reliant and sceptical of corporate messaging; wants autonomy and results over ceremony. The cohort holding most middle management.",
        myth: "“Cynical” reads better as having watched loyalty go unrewarded once already.",
        axes: [45.0, 55.0, 40.0, 55.0, 50.0],
    },
    Generation {
        key: "millennials",
        label: "Millennials (Gen Y)",
        years: "1981–1996",
        formative: "Grew up with the web; hit the 2008 crash and student debt on the way into a weak job market.",
        at_work: "Wants purpose, feedback and progression sooner than the ladder offers them; comfortable working in the open.",
        myth: "“Entitled” largely dissolves once you control for career stage — every cohort asks for more at 25 than at 55.",
        axes: [55.0, 60.0, 58.0, 55.0, 40.0],
    },
    Generation {
        key: "gen-z",
        label: "Gen Z",
        years: "1997–2012",
        formative: "Smartphones and social media from childhood, pandemic schooling, and visible climate and housing anxiety.",
        at_work: "Fluent with consumer software but not automatically with work tools; expects flexibility, pay transparency and quick feedback; changes jobs to change conditions.",
        myth: "The best-evidenced cohort difference is not a personality trait but sharply higher reported anxiety and stress.",
        axes: [40.0, 65.0, 60.0, 60.0, 25.0],
    },
    Generation {
        key: "gen-alpha",
        label: "Gen Alpha",
        years: "2013–2024",
        formative: "Tablets before literacy, AI assistants as a default tool, post-pandemic schooling.",
        at_work: "Not in the workforce yet — the oldest are still in school.",
        myth: "Anything specific said today about their working personality is marketing, not evidence.",
        axes: [45.0, 65.0, 55.0, 60.0, 30.0],
    },
];

fn find_generation(key: &str) -> Option<&'static Generation> {
    GENERATIONS.iter().find(|g| g.key == key)
}

// -------------------------------------------------------------- selection

#[derive(Clone, PartialEq)]
struct Series {
    label: String,
    sub: String,
    values: [f64; 5],
}

fn keys_of(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
        .collect()
}

/// Add or remove `key`, dropping the oldest selection once the chart is full.
fn toggled(mut keys: Vec<String>, key: &str) -> Vec<String> {
    match keys.iter().position(|k| k == key) {
        Some(index) => {
            keys.remove(index);
        }
        None => {
            keys.push(key.to_string());
            if keys.len() > MAX_SERIES {
                keys.remove(0);
            }
        }
    }
    keys
}

fn toggle(selection: RwSignal<String>, key: &str) {
    let keys = toggled(keys_of(&selection.get_untracked()), key);
    selection.set(keys.join(","));
}

fn series_of(keys: &[String]) -> Vec<Series> {
    keys.iter()
        .filter_map(|key| {
            if let Some(t) = find_type(key) {
                Some(Series {
                    label: t.code.to_string(),
                    sub: t.name.to_string(),
                    values: axes_of(t.code),
                })
            } else {
                find_generation(key).map(|g| Series {
                    label: g.label.to_string(),
                    sub: g.years.to_string(),
                    values: g.axes,
                })
            }
        })
        .collect()
}

// ----------------------------------------------------------- spider chart

const W: f64 = 620.0;
const H: f64 = 410.0;
const CX: f64 = 310.0;
const CY: f64 = 208.0;
const R: f64 = 148.0;

fn angle(index: usize) -> f64 {
    -std::f64::consts::FRAC_PI_2 + index as f64 * std::f64::consts::TAU / 5.0
}

fn point(index: usize, value: f64) -> (f64, f64) {
    let a = angle(index);
    let radius = R * (value / 100.0);
    (CX + radius * a.cos(), CY + radius * a.sin())
}

fn polygon(values: &[f64; 5]) -> String {
    (0..5)
        .map(|i| {
            let (x, y) = point(i, values[i]);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn radar(series: Vec<Series>) -> AnyView {
    let mut els: Vec<AnyView> = Vec::new();

    // Grid rings, outermost drawn a shade stronger.
    for level in [25.0, 50.0, 75.0, 100.0] {
        let class = if level == 100.0 {
            "psy-ring psy-ring-edge"
        } else {
            "psy-ring"
        };
        let points = polygon(&[level; 5]);
        els.push(view! { <polygon class=class points=points /> }.into_any());
    }

    // Spokes and axis labels.
    for (i, axis) in AXES.iter().enumerate() {
        let (x, y) = point(i, 100.0);
        els.push(
            view! { <line class="psy-spoke" x1=CX y1=CY x2=x y2=y /> }.into_any(),
        );

        let a = angle(i);
        let (lx, ly) = (CX + (R + 26.0) * a.cos(), CY + (R + 26.0) * a.sin());
        let anchor = if a.cos() > 0.2 {
            "start"
        } else if a.cos() < -0.2 {
            "end"
        } else {
            "middle"
        };
        els.push(
            view! {
                <text class="psy-axis" x=lx y=ly text-anchor=anchor>{axis.dimension}</text>
            }
            .into_any(),
        );
        els.push(
            view! {
                <text class="psy-pole" x=lx y=ly + 13.0 text-anchor=anchor>
                    {format!("→ {}", axis.high)}
                </text>
            }
            .into_any(),
        );
    }

    // One polygon (plus vertex dots) per selected profile.
    for (index, s) in series.iter().enumerate() {
        let points = polygon(&s.values);
        els.push(
            view! { <polygon class=format!("psy-poly psy-s{index}") points=points /> }.into_any(),
        );
        for i in 0..5 {
            let (x, y) = point(i, s.values[i]);
            els.push(
                view! { <circle class=format!("psy-dot psy-s{index}") cx=x cy=y r=3.5 /> }
                    .into_any(),
            );
        }
    }

    if series.is_empty() {
        els.push(
            view! {
                <text class="psy-hint" x=CX y=CY text-anchor="middle">
                    "pick a type or a generation below"
                </text>
            }
            .into_any(),
        );
    }

    let legend = series
        .iter()
        .enumerate()
        .map(|(index, s)| {
            view! {
                <span class=format!("psy-legend-item psy-s{index}")>
                    <span class="psy-swatch"></span>
                    <strong>{s.label.clone()}</strong>
                    <span class="psy-legend-sub">{s.sub.clone()}</span>
                </span>
            }
        })
        .collect_view();

    view! {
        <div class="psy-chart">
            <svg class="psy-svg" viewBox=format!("0 0 {W} {H}") preserveAspectRatio="xMidYMid meet">
                {els.into_iter().collect_view()}
            </svg>
            <div class="psy-legend">{legend}</div>
            <p class="psy-centre-note">
                "Centre of each axis is the opposite pole: "
                {AXES.iter().map(|a| a.low).collect::<Vec<_>>().join(" · ")}
            </p>
        </div>
    }
    .into_any()
}

// ------------------------------------------------------------------- page

#[component]
pub fn PsychePage() -> impl IntoView {
    let meta = registry::find("psyche").expect("psyche registered");
    let selection = crate::ui::use_persisted("psyche", DEFAULT_SELECTION);
    let keys = Memo::new(move |_| keys_of(&selection.get()));
    let is_on = move |key: &str| keys.get().iter().any(|k| k == key);

    let chart = move || radar(series_of(&keys.get()));

    let type_groups = ROLES
        .iter()
        .map(|role| {
            let chips = TYPES
                .iter()
                .filter(|t| t.role == role.name)
                .map(|t| {
                    let code = t.code;
                    view! {
                        <button
                            class=move || {
                                if is_on(code) { "psy-chip psy-chip-on" } else { "psy-chip" }
                            }
                            on:click=move |_| toggle(selection, code)
                        >
                            <span class="psy-code">{t.code}</span>
                            <span class="psy-name">{t.name}</span>
                            <span class="psy-share">{t.share}</span>
                        </button>
                    }
                })
                .collect_view();
            view! {
                <section class="psy-group">
                    <h3 class="psy-group-title">
                        {role.name}
                        <span class="psy-group-letters">{role.letters}</span>
                        <span class="psy-group-gist">{role.gist}</span>
                    </h3>
                    <div class="psy-chips">{chips}</div>
                </section>
            }
        })
        .collect_view();

    let details = move || {
        let cards = keys
            .get()
            .iter()
            .filter_map(|key| find_type(key))
            .map(|t| {
                view! {
                    <article class="psy-card">
                        <header>
                            <span class="psy-card-code">{t.code}</span>
                            <h3>{t.name}</h3>
                            <span class="psy-card-role">{t.role}" · "{t.share}</span>
                        </header>
                        <p class="psy-tldr">{t.tldr}</p>
                        <p><span class="psy-k">"At work"</span>{t.at_work}</p>
                        <p><span class="psy-k">"Watch for"</span>{t.watch}</p>
                    </article>
                }
            })
            .collect_view();
        view! { <div class="psy-cards">{cards}</div> }
    };

    let generation_cards = GENERATIONS
        .iter()
        .map(|g| {
            let key = g.key;
            view! {
                <article
                    class=move || {
                        if is_on(key) { "psy-gen psy-gen-on" } else { "psy-gen" }
                    }
                    on:click=move |_| toggle(selection, key)
                >
                    <header>
                        <h3>{g.label}</h3>
                        <span class="psy-years">{g.years}</span>
                        <span class="psy-plot">
                            {move || if is_on(key) { "plotted" } else { "plot →" }}
                        </span>
                    </header>
                    <p><span class="psy-k">"Formed by"</span>{g.formative}</p>
                    <p><span class="psy-k">"At work"</span>{g.at_work}</p>
                    <p class="psy-myth"><span class="psy-k">"The stereotype"</span>{g.myth}</p>
                </article>
            }
        })
        .collect_view();

    let copy_png = move |_| crate::ui::copy_png(".psy-chart");
    let save_png = move |_| crate::ui::download_png(".psy-chart", "psyche.png");

    view! {
        <div class="psyche-page">
            <header class="psy-head">
                <a class="back" href="#/">"← Mentor"</a>
                <h1>{meta.glyph}" "{meta.greek}</h1>
                <p class="psy-sub">{meta.tagline}</p>
                <a class="reference" href=REFERENCE target="_blank" rel="noreferrer">
                    "reference ↗"
                </a>
            </header>

            <p class="psy-caveat">
                <strong>"Read this first. "</strong>
                "Type is a vocabulary, not a measurement: the four-letter model has poor
                test–retest reliability, and its dichotomies are really continuous traits
                rather than boxes. The shapes below are drawn from the letters themselves —
                no trait scores are invented. Generational profiles are indicative tendencies
                from survey research, not personality data; the spread inside any cohort is
                far larger than the gap between cohorts, and “generation” is easily confused
                with simply being young."
            </p>

            <div class="psy-stage">
                <div class="export-bar">
                    <button class="exp" on:click=copy_png title="Copy the chart to the clipboard as a PNG image">"Copy PNG"</button>
                    <button class="exp" on:click=save_png title="Download the chart as a PNG image">"Save PNG"</button>
                    <span class="exp-sep"></span>
                    <span class="psy-count">
                        {move || format!("{}/{MAX_SERIES} plotted", keys.get().len())}
                    </span>
                </div>
                {chart}
            </div>

            <h2 class="psy-section">"The sixteen types"</h2>
            <p class="psy-section-sub">
                "Click to overlay (up to "{MAX_SERIES}" at a time). Percentages are the
                estimated share of the population, which is why the rarest types are the
                ones you hear most about."
            </p>
            {type_groups}
            {details}

            <h2 class="psy-section">"The generations"</h2>
            <p class="psy-section-sub">
                "Click a cohort to plot it against the types. Year ranges follow the Pew
                Research Center's definitions."
            </p>
            <div class="psy-gens">{generation_cards}</div>

            <p class="psy-note">
                "Identity (Assertive–Turbulent) is not encoded in the four letters, so every
                type sits on the midline there — it is the axis where the generational
                profiles carry most of their signal."
            </p>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_sixteen_types_four_per_role() {
        assert_eq!(TYPES.len(), 16);
        for role in ROLES {
            assert_eq!(
                TYPES.iter().filter(|t| t.role == role.name).count(),
                4,
                "{} should hold four types",
                role.name
            );
        }
    }

    #[test]
    fn axes_follow_the_letters() {
        assert_eq!(axes_of("INTJ"), [20.0, 80.0, 20.0, 20.0, 50.0]);
        assert_eq!(axes_of("ESFP"), [80.0, 20.0, 80.0, 80.0, 50.0]);
    }

    #[test]
    fn selection_toggles_and_caps() {
        let mut keys = Vec::new();
        for key in ["INTJ", "ENFP", "gen-x", "gen-z", "ISTJ"] {
            keys = toggled(keys, key);
        }
        assert_eq!(keys.len(), MAX_SERIES, "the oldest should be dropped");
        assert_eq!(keys[0], "ENFP");

        keys = toggled(keys, "ENFP");
        assert!(!keys.contains(&"ENFP".to_string()));
    }

    #[test]
    fn series_resolve_types_and_generations_and_ignore_junk() {
        let keys = keys_of("INTJ,gen-z,nonsense");
        let series = series_of(&keys);
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].label, "INTJ");
        assert_eq!(series[1].label, "Gen Z");
    }

    #[test]
    fn default_selection_resolves() {
        assert_eq!(series_of(&keys_of(DEFAULT_SELECTION)).len(), 2);
    }

    #[test]
    fn polygon_has_five_vertices() {
        assert_eq!(polygon(&[50.0; 5]).split(' ').count(), 5);
    }
}
