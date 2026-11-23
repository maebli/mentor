//! Central registry of every tool in the suite.
//!
//! Adding a new tool is a two-step change: add a `ToolMeta` entry here and add
//! a match arm in `tools::route`. The home page, navigation and fuzzy search
//! are all driven from this list, so nothing else needs to change.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Shaping the design: structure, components, combinations.
    Design,
    /// Getting to the root of a problem.
    ProblemSolving,
    /// Deciding and forecasting under uncertainty.
    Decision,
    Delivery,
    /// Things to look up rather than fill in.
    Reference,
}

impl Category {
    pub const ORDER: [Category; 5] = [
        Category::Design,
        Category::ProblemSolving,
        Category::Decision,
        Category::Delivery,
        Category::Reference,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Category::Design => "Design",
            Category::ProblemSolving => "Problem solving",
            Category::Decision => "Decisions",
            Category::Delivery => "Delivery",
            Category::Reference => "Reference",
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ToolMeta {
    /// URL slug (also the localStorage namespace). Lives after `#/`.
    pub slug: &'static str,
    /// The Greek name shown as the tool's title.
    pub greek: &'static str,
    /// Plain-language name of the technique.
    pub title: &'static str,
    /// One-line description of what it does.
    pub tagline: &'static str,
    /// Emoji glyph used on cards and nav.
    pub glyph: &'static str,
    pub category: Category,
    pub wikipedia_article: &'static str,
    /// Extra search keywords beyond the visible text.
    pub tags: &'static [&'static str],
}

impl ToolMeta {
    pub fn wikipedia_url(&self) -> String {
        format!("https://en.wikipedia.org/wiki/{}", self.wikipedia_article)
    }

    /// The `href` for an in-app link (hash routing keeps GitHub Pages happy).
    pub fn href(&self) -> String {
        format!("#/{}", self.slug)
    }

    /// Everything the fuzzy matcher is allowed to search over.
    pub fn haystack(&self) -> String {
        format!(
            "{} {} {} {}",
            self.greek,
            self.title,
            self.tagline,
            self.tags.join(" ")
        )
    }
}

/// The full catalogue, in a stable order. Grouping for display is done by
/// `Category::ORDER`, not by position here.
pub fn tools() -> &'static [ToolMeta] {
    &[
        ToolMeta {
            slug: "agora",
            wikipedia_article: "Class-responsibility-collaboration_card",
            greek: "Agora",
            title: "CRC Card Board",
            tagline: "Map classes, their responsibilities, and the collaborators they depend on.",
            glyph: "🏛️",
            category: Category::Design,
            tags: &["crc", "class", "responsibility", "collaborator", "oop", "cards", "board"],
        },
        ToolMeta {
            slug: "morpheus",
            wikipedia_article: "Morphological_analysis_(problem-solving)",
            greek: "Morpheus",
            title: "Morphological Analysis",
            tagline: "Break a design into parameters and explore combinations.",
            glyph: "🔷",
            category: Category::Design,
            tags: &["morphological", "zwicky", "box", "matrix", "parameters", "options"],
        },
        ToolMeta {
            slug: "ariadne",
            wikipedia_article: "Issue_tree",
            greek: "Ariadne",
            title: "Issue Tree",
            tagline: "Break a question into smaller questions and see how they fit together.",
            glyph: "🧵",
            category: Category::ProblemSolving,
            tags: &["issue", "tree", "mece", "logic", "decomposition", "hypothesis"],
        },
        ToolMeta {
            slug: "socrates",
            wikipedia_article: "Five_whys",
            greek: "Socrates",
            title: "5 Whys",
            tagline: "Ask 'why' until the root cause surfaces.",
            glyph: "❓",
            category: Category::ProblemSolving,
            tags: &["five", "whys", "why", "root cause", "toyota", "elenchus"],
        },
        ToolMeta {
            slug: "cetus",
            wikipedia_article: "Ishikawa_diagram",
            greek: "Cetus",
            title: "Ishikawa Diagram",
            tagline: "Fishbone cause-and-effect analysis across categories.",
            glyph: "🐟",
            category: Category::ProblemSolving,
            tags: &["ishikawa", "fishbone", "cause", "effect", "6m", "root cause"],
        },
        ToolMeta {
            slug: "herakles",
            wikipedia_article: "Eight_disciplines_problem_solving",
            greek: "Herakles",
            title: "8D Report",
            tagline: "The eight disciplines of structured problem solving.",
            glyph: "🦁",
            category: Category::ProblemSolving,
            tags: &["8d", "eight disciplines", "quality", "corrective action", "report"],
        },
        ToolMeta {
            slug: "cassandra",
            wikipedia_article: "Premortem",
            greek: "Cassandra",
            title: "Premortem",
            tagline: "Imagine the project failed, then list and rank why.",
            glyph: "🔮",
            category: Category::Decision,
            tags: &[
                "premortem", "prospective hindsight", "risk", "failure", "klein",
                "kahneman", "assumptions",
            ],
        },
        ToolMeta {
            slug: "pythia",
            wikipedia_article: "Reference_class_forecasting",
            greek: "Pythia",
            title: "Reference-Class Forecasting",
            tagline: "Check your estimate against how similar past projects actually went.",
            glyph: "📈",
            category: Category::Decision,
            tags: &[
                "reference class", "outside view", "base rate", "planning fallacy",
                "flyvbjerg", "estimate", "forecast",
            ],
        },
        ToolMeta {
            slug: "themis",
            wikipedia_article: "Decision-matrix_method",
            greek: "Themis",
            title: "Weighted Decision Matrix",
            tagline: "Score options against weighted criteria to compare them fairly.",
            glyph: "⚖️",
            category: Category::Decision,
            tags: &[
                "decision matrix", "weighted scoring", "mcda", "criteria", "options",
                "trade-off", "pugh",
            ],
        },
        ToolMeta {
            slug: "tyche",
            wikipedia_article: "Convolution_of_probability_distributions",
            greek: "Tyche",
            title: "Probabilistic Estimate",
            tagline: "Combine uncertain task estimates into a range for the whole project.",
            glyph: "🎲",
            category: Category::Decision,
            tags: &[
                "estimate", "estimation", "guesstimate", "monte carlo", "convolution",
                "probability", "distribution", "uncertainty", "risk", "pert", "confidence",
                "percentile", "range", "three point",
            ],
        },
        ToolMeta {
            slug: "artemis",
            wikipedia_article: "SMART_criteria",
            greek: "Artemis",
            title: "SMART Goals",
            tagline: "Write a goal, then have each of the five SMART criteria graded by heuristics.",
            glyph: "🏹",
            category: Category::Decision,
            tags: &[
                "smart", "goal", "goals", "objective", "okr", "specific", "measurable",
                "achievable", "relevant", "time-bound", "target", "planning",
            ],
        },
        ToolMeta {
            slug: "demeter",
            wikipedia_article: "Cohort_analysis",
            greek: "Demeter",
            title: "Cohort Retention Explorer",
            tagline: "Compare groups at the same age using cohort analysis from The Lean Startup.",
            glyph: "🌾",
            category: Category::Decision,
            tags: &["retention", "cohort", "ries", "analytics", "product", "startup"],
        },
        ToolMeta {
            slug: "atlas",
            wikipedia_article: "Brooks%27s_law",
            greek: "Atlas",
            title: "Team Communication Planner",
            tagline: "Compare team structures using the communication paths described in The Mythical Man-Month.",
            glyph: "🌎",
            category: Category::Delivery,
            tags: &["staffing", "coordination", "brooks", "team", "communication"],
        },
        ToolMeta {
            slug: "chronos",
            wikipedia_article: "Critical_path_method",
            greek: "Chronos",
            title: "Critical Path Planner",
            tagline: "Schedule dependent tasks and see which delays change the finish date.",
            glyph: "⌛",
            category: Category::Delivery,
            tags: &["cpm", "dependency", "schedule", "slack", "gantt"],
        },
        ToolMeta {
            slug: "hermes",
            wikipedia_article: "Kanban_board",
            greek: "Hermes",
            title: "Kanban Board",
            tagline: "Move cards through your workflow and limit work in progress.",
            glyph: "📋",
            category: Category::Delivery,
            tags: &["flow", "wip", "tasks", "columns"],
        },
        ToolMeta {
            slug: "iris",
            wikipedia_article: "Quorum_(distributed_computing)",
            greek: "Iris",
            title: "Replica Quorum Calculator",
            tagline: "Compare replica overlap and availability using the replication model in Designing Data-Intensive Applications.",
            glyph: "🌐",
            category: Category::Design,
            tags: &["ddia", "kleppmann", "read", "write", "database"],
        },
        ToolMeta {
            slug: "hestia",
            wikipedia_article: "Timeblocking",
            greek: "Hestia",
            title: "Time Block Planner",
            tagline: "Plan a day with the time-blocking method from Deep Work and spot gaps or conflicts.",
            glyph: "🕰",
            category: Category::Delivery,
            tags: &["cal newport", "agenda", "focus", "calendar"],
        },
        ToolMeta {
            slug: "hephaestus",
            wikipedia_article: "Theory_of_constraints",
            greek: "Hephaestus",
            title: "Bottleneck Planner",
            tagline: "Compare serial stages and see where an extra station could increase hourly output.",
            glyph: "⚒",
            category: Category::Delivery,
            tags: &["The Goal", "Goldratt", "constraint", "capacity", "throughput"],
        },
        ToolMeta {
            slug: "danaus",
            wikipedia_article: "Stock_and_flow",
            greek: "Danaus",
            title: "Stock and Flow Simulator",
            tagline: "Track what enters, what leaves, and how much remains at the end of each period.",
            glyph: "🏺",
            category: Category::Decision,
            tags: &["Thinking in Systems", "Meadows", "stock", "flow", "backlog", "inventory"],
        },
        ToolMeta {
            slug: "echo",
            wikipedia_article: "Huffman_coding",
            greek: "Echo",
            title: "Huffman Coding Explorer",
            tagline: "Turn a message into a prefix code and compare its bit count with a fixed-width code.",
            glyph: "🔣",
            category: Category::Design,
            tags: &["SICP", "compression", "algorithm", "encoding", "frequency"],
        },
        ToolMeta {
            slug: "metis",
            wikipedia_article: "Heuristic",
            greek: "Metis",
            title: "Heuristics",
            tagline: "A visual reference of rules of thumb for design and thinking.",
            glyph: "🦉",
            category: Category::Reference,
            tags: &["heuristic", "rules of thumb", "principles", "biases", "wisdom"],
        },
        ToolMeta {
            slug: "psyche",
            wikipedia_article: "Myers%E2%80%93Briggs_Type_Indicator",
            greek: "Psyche",
            title: "Personality & Generations",
            tagline: "The sixteen types on a spider chart — and the generational cohorts plotted beside them.",
            glyph: "🧠",
            category: Category::Reference,
            tags: &[
                "personality", "16 personalities", "mbti", "myers briggs", "type", "introvert",
                "extravert", "spider", "radar", "chart", "generation", "generations", "boomer",
                "gen x", "millennial", "gen z", "gen alpha", "cohort", "team",
            ],
        },
    ]
}

pub fn find(slug: &str) -> Option<&'static ToolMeta> {
    tools().iter().find(|t| t.slug == slug)
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_tool_has_a_wikipedia_article() {
        for tool in super::tools() {
            assert!(!tool.wikipedia_article.trim().is_empty(), "{}", tool.slug);
            assert!(!tool.wikipedia_article.contains("://"), "{}", tool.slug);
        }
    }
}
