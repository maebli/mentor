use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;

const DEFAULT: &str =
    "Balanced | 3 | 2 | 2\nFast reads | 3 | 1 | 3\nIndependent reads | 3 | 1 | 1\n";

#[derive(Clone, PartialEq)]
struct Quorum {
    name: String,
    replicas: u32,
    read: u32,
    write: u32,
}

impl Quorum {
    fn overlap(&self) -> u32 {
        self.read.saturating_sub(self.replicas - self.write)
    }
}

fn configurations(source: &str) -> Result<Vec<Quorum>, Vec<ParseIssue>> {
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        let fields: Vec<_> = line.content.split('|').map(str::trim).collect();
        if let [name, n, r, w] = fields.as_slice() {
            if let (Ok(replicas), Ok(read), Ok(write)) = (n.parse(), r.parse(), w.parse()) {
                if !name.is_empty()
                    && (1..=replicas).contains(&read)
                    && (1..=replicas).contains(&write)
                {
                    rows.push(Quorum {
                        name: name.to_string(),
                        replicas,
                        read,
                        write,
                    });
                    continue;
                }
            }
        }
        issues.push(ParseIssue::error(line.number, "Use name | replicas | read replies | write replies. Each reply count must be between 1 and the replica count."));
    }
    if issues.is_empty() {
        Ok(rows)
    } else {
        Err(issues)
    }
}

fn compare(rows: Vec<Quorum>, _: RwSignal<String>) -> AnyView {
    if rows.is_empty() {
        return view! { <p class="canvas-empty">"Enter a replica configuration to compare its quorums."</p> }.into_any();
    }
    let cards = rows.into_iter().map(|q| {
        let overlap = q.overlap();
        view! { <li class="quorum-row">
            <h3>{q.name}</h3>
            <p>{format!("{} replicas · {} read replies · {} write replies", q.replicas, q.read, q.write)}</p>
            <strong>{if overlap == 0 { "Read and write sets can be disjoint".into() }
                else { format!("Minimum shared replicas: {overlap}") }}</strong>
            <div>{format!("Offline replicas tolerated · reads {} · writes {}", q.replicas-q.read, q.replicas-q.write)}</div>
        </li> }
    }).collect_view();
    view! { <section class="schedule-sheet"><h2>"Quorum comparison"</h2>
        <p>"Assumes a fixed replica set for one key. Overlap describes the responding sets. Freshness also depends on version handling and concurrent writes."</p>
        <ul>{cards}</ul></section> }.into_any()
}

#[component]
pub fn IrisTool() -> impl IntoView {
    source_program("iris", DEFAULT, "One configuration per line. Enter name | replica count | required read replies | required write replies.",
        "https://www.oreilly.com/library/view/designing-data-intensive-applications/9781491903063/ch05.html",
        &[], configurations, compare)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_requires_strict_majority_of_combined_replies() {
        let rows = configurations("Boundary | 4 | 2 | 2\nOverlap | 5 | 4 | 3")
            .ok()
            .unwrap();
        assert_eq!(rows[0].overlap(), 0);
        assert_eq!(rows[1].overlap(), 2);
    }

    #[test]
    fn rejects_impossible_thresholds() {
        for source in ["X | 2 | 0 | 1", "X | 2 | 1 | 3", "X | 0 | 0 | 0"] {
            assert!(configurations(source).is_err());
        }
    }
}
