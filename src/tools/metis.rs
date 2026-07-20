//! Metis — a searchable, read-only reference of design and thinking heuristics.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Heuristic";

const CATEGORIES: [&str; 5] = [
    "Mental Models",
    "Uncertainty & Risk",
    "Design & Code",
    "Systems & Process",
    "Cognition & Decisions",
];

struct Heuristic {
    name: &'static str,
    tldr: &'static str,
    category: &'static str,
}

static HEURISTICS: &[Heuristic] = &[
    Heuristic {
        name: "Occam's Razor",
        tldr: "Prefer the simplest explanation or solution that fits the facts.",
        category: "Design & Code",
    },
    Heuristic {
        name: "KISS",
        tldr: "Simpler designs are easier to build, understand, and change.",
        category: "Design & Code",
    },
    Heuristic {
        name: "YAGNI",
        tldr: "Don't build a feature until it's actually needed.",
        category: "Design & Code",
    },
    Heuristic {
        name: "DRY",
        tldr: "Each piece of knowledge should have one authoritative representation.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Rule of Three",
        tldr: "Wait until you've seen something three times before abstracting it.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Principle of Least Astonishment",
        tldr: "A component should behave the way its users expect.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Single Responsibility",
        tldr: "A module should have exactly one reason to change.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Separation of Concerns",
        tldr: "Keep distinct aspects of a system in distinct parts.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Law of Demeter",
        tldr: "Talk only to your immediate collaborators; don't reach through them.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Composition over Inheritance",
        tldr: "Prefer assembling behavior over deep class hierarchies.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Fail Fast",
        tldr: "Surface errors as early and loudly as possible.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Premature Optimization",
        tldr: "Don't optimize before you've measured where the cost is.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Worse is Better",
        tldr: "A simpler, less complete solution often beats a perfect complex one.",
        category: "Design & Code",
    },
    Heuristic {
        name: "Conway's Law",
        tldr: "Systems mirror the communication structure of the organization that builds them.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Gall's Law",
        tldr: "A working complex system evolves from a working simple system.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Brooks's Law",
        tldr: "Adding people to a late software project makes it later.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Hofstadter's Law",
        tldr: "It always takes longer than you expect, even when you account for Hofstadter's Law.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Parkinson's Law",
        tldr: "Work expands to fill the time available for it.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Ninety-Ninety Rule",
        tldr: "The first 90% takes 90% of the time; the last 10% takes the other 90%.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Postel's Law",
        tldr: "Be conservative in what you send, liberal in what you accept.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Chesterton's Fence",
        tldr: "Don't remove something until you understand why it was put there.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Goodhart's Law",
        tldr: "When a measure becomes a target, it stops being a good measure.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Second-System Effect",
        tldr: "The second system is the most dangerous — bloated by over-ambition.",
        category: "Systems & Process",
    },
    Heuristic {
        name: "Hanlon's Razor",
        tldr: "Never attribute to malice what is adequately explained by carelessness.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Sunk Cost Fallacy",
        tldr: "Past investment shouldn't drive future decisions.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Dunning-Kruger Effect",
        tldr: "The less competent you are, the more you may overestimate your competence.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Pareto Principle (80/20)",
        tldr: "Roughly 80% of effects come from 20% of causes.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Murphy's Law",
        tldr: "Anything that can go wrong, will — so design for it.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Confirmation Bias",
        tldr: "We seek and favour evidence that confirms what we already believe.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Availability Heuristic",
        tldr: "We judge how likely something is by how easily examples come to mind.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Loss Aversion",
        tldr: "A loss hurts about twice as much as an equal gain feels good.",
        category: "Cognition & Decisions",
    },
    Heuristic {
        name: "Anchoring",
        tldr: "The first number you hear drags every later estimate toward it.",
        category: "Cognition & Decisions",
    },
    // --- Mental Models: the general-purpose thinking toolkit ---
    Heuristic {
        name: "First Principles",
        tldr: "Reason up from what must be true, not by analogy to what already exists.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Inversion",
        tldr: "Solve it backwards — ask how you'd guarantee failure, then avoid exactly that.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Second-Order Thinking",
        tldr: "Ask \"and then what?\" — trace the consequences of the consequences.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Circle of Competence",
        tldr: "Know the edge of what you actually understand, and operate inside it.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Opportunity Cost",
        tldr: "The real cost of anything is the best alternative you gave up for it.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Margin of Safety",
        tldr: "Leave slack between what you expect and what you can actually survive.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Map Is Not the Territory",
        tldr: "A model is a useful simplification, never the reality it stands in for.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Regression to the Mean",
        tldr: "Extreme results tend to be followed by more ordinary ones.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Base Rates",
        tldr: "Anchor on how often something happens in general before trusting this case.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Compounding",
        tldr: "Small advantages, repeated over time, produce wildly outsized results.",
        category: "Mental Models",
    },
    Heuristic {
        name: "Incentives",
        tldr: "Show me the incentive and I'll show you the outcome.",
        category: "Mental Models",
    },
    // --- Uncertainty & Risk: Taleb's toolkit for a world of fat tails ---
    Heuristic {
        name: "Antifragility",
        tldr: "Some things gain from disorder — build systems that get stronger under stress.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Lindy Effect",
        tldr: "For ideas and tech, what has survived long is likely to survive longer still.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Black Swan",
        tldr: "Rare, unpredictable, high-impact events dominate history — plan for the unforeseeable.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Barbell Strategy",
        tldr: "Pair extreme safety with small capped bets on huge upside; avoid the fragile middle.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Skin in the Game",
        tldr: "Don't trust advice from anyone who bears no cost when they're wrong.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Via Negativa",
        tldr: "Improve by removing — subtracting harm usually beats adding cleverness.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Optionality",
        tldr: "Prefer choices that cap your downside but leave the upside open.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Trial and Error",
        tldr: "Tinkering with cheap, survivable mistakes beats grand top-down prediction.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Ludic Fallacy",
        tldr: "The tidy odds of games don't model the messy, open risks of real life.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Narrative Fallacy",
        tldr: "We invent tidy stories for random events, then mistake them for understanding.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Turkey Problem",
        tldr: "A long run of good days is no proof of safety — until the day it isn't.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Iatrogenics",
        tldr: "The harm caused by the intervention itself — sometimes doing nothing is best.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Silent Evidence",
        tldr: "Winners write history; the graveyard of failures is invisible but just as real.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Precautionary Principle",
        tldr: "Against risks of ruin, the burden of proof is on safety — never bet the irreplaceable.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Seneca's Asymmetry",
        tldr: "Gains arrive slowly, ruin arrives fast — fragility is a steep one-way cliff.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Convexity",
        tldr: "Favour payoffs where, for the same shock, the upside outweighs the downside.",
        category: "Uncertainty & Risk",
    },
    Heuristic {
        name: "Minority Rule",
        tldr: "A small, intransigent minority can bend the whole system to its preference.",
        category: "Uncertainty & Risk",
    },
];

fn card(heuristic: &'static Heuristic) -> impl IntoView {
    view! {
        <div class="metis-card">
            <span class="metis-name">{heuristic.name}</span>
            <span class="metis-tldr">{heuristic.tldr}</span>
        </div>
    }
}

#[component]
pub fn MetisPage() -> impl IntoView {
    let query = RwSignal::new(String::new());

    let results = move || {
        let q = query.get();
        let q = q.trim();
        if q.is_empty() {
            CATEGORIES
                .iter()
                .map(|category| {
                    let cards = HEURISTICS
                        .iter()
                        .filter(|heuristic| heuristic.category == *category)
                        .map(card)
                        .collect_view();
                    view! {
                        <section class="metis-group">
                            <h2 class="metis-group-title">{*category}</h2>
                            <div class="metis-grid">{cards}</div>
                        </section>
                    }
                })
                .collect_view()
                .into_any()
        } else {
            let matcher = SkimMatcherV2::default();
            let mut scored: Vec<(i64, &'static Heuristic)> = HEURISTICS
                .iter()
                .filter_map(|heuristic| {
                    let haystack = format!(
                        "{} {} {}",
                        heuristic.name, heuristic.tldr, heuristic.category
                    );
                    matcher
                        .fuzzy_match(&haystack, q)
                        .map(|score| (score, heuristic))
                })
                .collect();
            scored.sort_by(|a, b| b.0.cmp(&a.0));

            if scored.is_empty() {
                view! { <p class="empty">"No heuristics match “" {q.to_string()} "”."</p> }
                    .into_any()
            } else {
                let cards = scored
                    .into_iter()
                    .map(|(_, heuristic)| card(heuristic))
                    .collect_view();
                view! {
                    <section class="metis-group">
                        <div class="metis-grid">{cards}</div>
                    </section>
                }
                .into_any()
            }
        }
    };

    view! {
        <div class="metis-page">
            <header>
                <a class="back" href="#/">"← Mentor"</a>
                <h1>"🦉 Metis"</h1>
                <p>"Rules of thumb for design and thinking."</p>
                <a class="reference" href=REFERENCE target="_blank" rel="noreferrer">
                    "reference ↗"
                </a>
                <input
                    class="search"
                    type="search"
                    placeholder="Search heuristics…"
                    autocomplete="off"
                    prop:value=move || query.get()
                    on:input=move |ev| query.set(event_target_value(&ev))
                />
            </header>
            {results}
        </div>
    }
}
