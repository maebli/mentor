use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;

const SAMPLE: &str = "One team | 12\nTwo squads | 6, 6\nThree squads | 4, 4, 4\n";

fn pairs(people: u32) -> u64 {
    u64::from(people) * u64::from(people.saturating_sub(1)) / 2
}

#[derive(Clone, PartialEq)]
struct Staffing {
    name: String,
    people: u32,
    internal: u64,
    between: u64,
}

fn scenario(line: &str) -> Option<Staffing> {
    let (name, sizes) = line.split_once('|')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut people = 0u32;
    let mut groups = 0;
    let mut internal = 0;
    for size in sizes.split(',') {
        let size = size.trim().parse::<u32>().ok()?;
        if size == 0 {
            return None;
        }
        people = people.checked_add(size)?;
        internal += pairs(size);
        groups += 1;
    }
    Some(Staffing {
        name: name.into(),
        people,
        internal,
        between: pairs(groups),
    })
}

fn scenarios(source: &str) -> Result<Vec<Staffing>, Vec<ParseIssue>> {
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        match scenario(&line.content) {
            Some(layout) => rows.push(layout),
            None => issues.push(ParseIssue::error(line.number, "Use scenario name | group sizes separated by commas. Each group needs at least one person and the total must fit a 32-bit count.")),
        }
    }
    if issues.is_empty() {
        Ok(rows)
    } else {
        Err(issues)
    }
}

fn layouts(rows: Vec<Staffing>, _: RwSignal<String>) -> AnyView {
    if rows.is_empty() {
        return view! { <p class="canvas-empty">"Enter a staffing scenario to count its communication paths."</p> }.into_any();
    }
    let rows = rows.into_iter().map(|s| {
        let all = pairs(s.people);
        let grouped = s.internal + s.between;
        let width = grouped as f64 * 100.0 / all.max(1) as f64;
        view! { <li class="quorum-row"><h3>{s.name}</h3>
            <p>{format!("{} people · {all} possible pairs across the whole team", s.people)}</p>
            <strong>{format!("{grouped} paths with one delegate per group")}</strong>
            <div class="schedule-track" aria-hidden="true"><span style:width=format!("{width}%")></span></div>
            <div>{format!("{} within groups + {} between delegates", s.internal, s.between)}</div>
        </li> }
    }).collect_view();
    view! { <section class="schedule-sheet"><h2>"Communication paths"</h2>
        <p>"Brooks counts a path for each pair of people. This comparison also models groups where every member can talk internally and delegates coordinate between groups."</p>
        <ul>{rows}</ul></section> }.into_any()
}

#[component]
pub fn AtlasTool() -> impl IntoView {
    source_program("atlas", SAMPLE, "Use scenario | group sizes. Enter a single size for one team, or separate several sizes with commas.",
        "https://courses.cs.duke.edu/fall18/compsci408/readings/mythical_man_month.pdf", &[], scenarios, layouts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_internal_and_delegate_pairs_once() {
        let s = scenario("Two groups | 4, 4").unwrap();
        assert_eq!((pairs(s.people), s.internal, s.between), (28, 12, 1));
        assert_eq!(scenario("Solo | 1").unwrap().internal, 0);
    }
    #[test]
    fn rejects_empty_groups_and_overflow() {
        assert!(scenario("Missing | 2,").is_none());
        assert!(scenario("Zero | 0").is_none());
        assert!(scenario("Overflow | 4294967295,1").is_none());
    }
}
