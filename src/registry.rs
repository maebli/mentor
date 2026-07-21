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
    /// Things to look up rather than fill in.
    Reference,
}

impl Category {
    pub const ORDER: [Category; 3] = [
        Category::Design,
        Category::ProblemSolving,
        Category::Reference,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Category::Design => "Design & Architecture",
            Category::ProblemSolving => "Problem Solving & Root Cause",
            Category::Reference => "Reference",
        }
    }
}

#[derive(Clone)]
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
    /// Extra search keywords beyond the visible text.
    pub tags: &'static [&'static str],
}

impl ToolMeta {
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
            greek: "Agora",
            title: "CRC Card Board",
            tagline: "Draggable Class · Responsibility · Collaborator cards for object design.",
            glyph: "🏛️",
            category: Category::Design,
            tags: &["crc", "class", "responsibility", "collaborator", "oop", "cards", "board"],
        },
        ToolMeta {
            slug: "morpheus",
            greek: "Morpheus",
            title: "Morphological Analysis",
            tagline: "Break a design into parameters and explore combinations.",
            glyph: "🔷",
            category: Category::Design,
            tags: &["morphological", "zwicky", "box", "matrix", "parameters", "options"],
        },
        ToolMeta {
            slug: "ariadne",
            greek: "Ariadne",
            title: "Issue Tree",
            tagline: "Decompose a question into a MECE tree of sub-issues.",
            glyph: "🧵",
            category: Category::ProblemSolving,
            tags: &["issue", "tree", "mece", "logic", "decomposition", "hypothesis"],
        },
        ToolMeta {
            slug: "socrates",
            greek: "Socrates",
            title: "5 Whys",
            tagline: "Ask 'why' until the root cause surfaces.",
            glyph: "❓",
            category: Category::ProblemSolving,
            tags: &["five", "whys", "why", "root cause", "toyota", "elenchus"],
        },
        ToolMeta {
            slug: "cetus",
            greek: "Cetus",
            title: "Ishikawa Diagram",
            tagline: "Fishbone cause-and-effect analysis across categories.",
            glyph: "🐟",
            category: Category::ProblemSolving,
            tags: &["ishikawa", "fishbone", "cause", "effect", "6m", "root cause"],
        },
        ToolMeta {
            slug: "herakles",
            greek: "Herakles",
            title: "8D Report",
            tagline: "The eight disciplines of structured problem solving.",
            glyph: "🦁",
            category: Category::ProblemSolving,
            tags: &["8d", "eight disciplines", "quality", "corrective action", "report"],
        },
        ToolMeta {
            slug: "metis",
            greek: "Metis",
            title: "Heuristics",
            tagline: "A visual reference of rules of thumb for design and thinking.",
            glyph: "🦉",
            category: Category::Reference,
            tags: &["heuristic", "rules of thumb", "principles", "biases", "wisdom"],
        },
    ]
}

pub fn find(slug: &str) -> Option<&'static ToolMeta> {
    tools().iter().find(|t| t.slug == slug)
}
