//! Metis — a searchable, read-only reference of design and thinking heuristics.

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Heuristic";

const CATEGORIES: [&str; 3] = [
    "Design & Code",
    "Systems & Process",
    "Cognition & Decisions",
];

struct Heuristic {
    name: &'static str,
    tldr: &'static str,
    category: &'static str,
}

static HEURISTICS: [Heuristic; 28] = [
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
