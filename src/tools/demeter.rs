use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;

const SAMPLE: &str = "January | 100 | 65, 52, 41\nFebruary | 140 | 98, 80\nMarch | 160 | 120\n";

#[derive(Clone, PartialEq)]
struct Cohort {
    name: String,
    size: u32,
    active: Vec<u32>,
}

fn cohort(line: &str) -> Option<Cohort> {
    let mut fields = line.split('|').map(str::trim);
    let name = fields.next()?;
    let size = fields.next()?.parse::<u32>().ok()?;
    let mut active = Vec::new();
    if let Some(counts) = fields.next().filter(|counts| !counts.is_empty()) {
        for count in counts.split(',') {
            let count = count.trim().parse().ok()?;
            if count > size {
                return None;
            }
            active.push(count);
        }
    }
    if name.is_empty() || size == 0 || fields.next().is_some() {
        return None;
    }
    Some(Cohort {
        name: name.to_owned(),
        size,
        active,
    })
}

fn cohorts(source: &str) -> Result<Vec<Cohort>, Vec<ParseIssue>> {
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        match cohort(&line.content) {
            Some(row) => rows.push(row),
            None => issues.push(ParseIssue::error(line.number, "Enter cohort | positive starting size | active counts. Each count must be between zero and the starting size.")),
        }
    }
    if issues.is_empty() {
        Ok(rows)
    } else {
        Err(issues)
    }
}

fn observed_totals(rows: &[Cohort]) -> Vec<(u64, u64)> {
    let periods = rows
        .iter()
        .map(|row| row.active.len())
        .max()
        .unwrap_or_default();
    let mut totals = vec![(0, 0); periods];
    for row in rows {
        for ((active, eligible), &count) in totals.iter_mut().zip(&row.active) {
            *active += u64::from(count);
            *eligible += u64::from(row.size);
        }
    }
    totals
}

fn rate(active: u64, eligible: u64) -> impl IntoView {
    let percent = 100.0 * active as f64 / eligible as f64;
    view! { <td title=format!("{active} of {eligible} active")
        style:background-color=format!("rgb(88 141 110 / {:.2})", percent / 190.0)>
        {format!("{percent:.1}%")}
    </td> }
}

fn matrix(rows: Vec<Cohort>, _: RwSignal<String>) -> AnyView {
    if rows.is_empty() {
        return view! { <p class="canvas-empty">"Enter a cohort to compare retention."</p> }
            .into_any();
    }
    let totals = observed_totals(&rows);
    let headers = (1..=totals.len())
        .map(|i| view! { <th scope="col">{format!("Period {i}")}</th> })
        .collect_view();
    let body = rows
        .into_iter()
        .map(|row| {
            let cells = row
                .active
                .into_iter()
                .map(|n| rate(n.into(), row.size.into()))
                .collect_view();
            view! { <tr><th scope="row">{row.name}</th><td>{row.size}</td>{cells}</tr> }
        })
        .collect_view();
    let weighted = totals
        .into_iter()
        .map(|(active, eligible)| rate(active, eligible))
        .collect_view();
    view! { <section class="schedule-sheet cohort-sheet"><h2>"Retention by cohort age"</h2>
        <p>"Each period uses the original cohort size. The pooled row includes only cohorts with an observation for that period."</p>
        <table><thead><tr><th scope="col">"Cohort"</th><th scope="col">"Joined"</th>{headers}</tr></thead>
        <tbody>{body}</tbody><tfoot><tr><th scope="row" colspan="2">"Pooled retention"</th>{weighted}</tr></tfoot></table>
    </section> }.into_any()
}

#[component]
pub fn DemeterTool() -> impl IntoView {
    source_program("demeter", SAMPLE, "Use cohort | starting size | active in periods 1, 2, … . Count unique members in equal-length periods. Omit future periods.",
        "https://www.startuplessonslearned.com/2009/01/lessons-learned-on-mashable-today.html", &[], cohorts, matrix)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weights_only_observed_cohorts_including_zero_returns() {
        let rows = cohorts("A | 10 | 0, 2\nB | 30 | 20\nC | 8").ok().unwrap();
        assert_eq!(observed_totals(&rows), [(20, 40), (2, 10)]);
    }
    #[test]
    fn rejects_impossible_observations() {
        for row in ["A | 0", "A | 2 | 3", "A | 2 | 1, ,2"] {
            assert!(cohort(row).is_none());
        }
    }
}
