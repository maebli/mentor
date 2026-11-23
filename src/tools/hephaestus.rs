use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;

const SAMPLE: &str = "Prepare | 2 | 6\nAssemble | 1 | 8\nInspect | 1 | 5";

#[derive(Clone, PartialEq)]
struct Stage {
    name: String,
    stations: u32,
    minutes: u32,
}

impl Stage {
    fn hourly(&self) -> f64 {
        60.0 * f64::from(self.stations) / f64::from(self.minutes)
    }
}

fn stages(source: &str) -> Result<Vec<Stage>, Vec<ParseIssue>> {
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        let fields: Vec<_> = line.content.split('|').map(str::trim).collect();
        if let [name, stations, minutes] = fields.as_slice() {
            if let (Ok(stations), Ok(minutes)) = (stations.parse(), minutes.parse()) {
                if !name.is_empty() && stations > 0 && minutes > 0 {
                    rows.push(Stage {
                        name: (*name).into(),
                        stations,
                        minutes,
                    });
                    continue;
                }
            }
        }
        issues.push(ParseIssue::error(line.number, "Enter stage | station count | minutes per item. Both numbers must be positive whole numbers."));
    }
    if issues.is_empty() {
        Ok(rows)
    } else {
        Err(issues)
    }
}

fn capacity_order(stages: &[Stage]) -> Vec<usize> {
    let mut order: Vec<_> = (0..stages.len()).collect();
    order.sort_by(|&a, &b| {
        let a = &stages[a];
        let b = &stages[b];
        (u64::from(a.stations) * u64::from(b.minutes))
            .cmp(&(u64::from(b.stations) * u64::from(a.minutes)))
    });
    order
}

fn with_extra_station(stages: &[Stage], order: &[usize], index: usize) -> f64 {
    let stage = &stages[index];
    let upgraded = 60.0 * (f64::from(stage.stations) + 1.0) / f64::from(stage.minutes);
    let other = if index == order[0] {
        order.get(1)
    } else {
        order.first()
    };
    other.map_or(upgraded, |&i| upgraded.min(stages[i].hourly()))
}

fn capacity(stages: Vec<Stage>, _: RwSignal<String>) -> AnyView {
    let order = capacity_order(&stages);
    let Some(&first) = order.first() else {
        return view! { <p class="canvas-empty">"Add a stage to estimate line capacity."</p> }
            .into_any();
    };
    let slowest = &stages[first];
    let rows = stages.iter().enumerate().map(|(index, stage)| {
        let limiting = u64::from(stage.stations) * u64::from(slowest.minutes)
            == u64::from(slowest.stations) * u64::from(stage.minutes);
        view! {
            <li class="quorum-row">
                <div class="schedule-label"><h3>{stage.name.clone()}</h3>
                    {limiting.then(|| view! { <strong>"Bottleneck"</strong> })}</div>
                <p>{format!("{:.2} items/hour at this stage", stage.hourly())}</p>
                <div>{format!("Line capacity with one more station here: {:.2} items/hour", with_extra_station(&stages, &order, index))}</div>
            </li>
        }
    }).collect_view();
    view! {
        <section class="schedule-sheet"><h2>{format!("{:.2} items/hour", slowest.hourly())}</h2>
            <p>"Steady-state capacity assumes every item visits every stage once, with identical parallel stations, steady demand, and no downtime."</p>
            <ul>{rows}</ul>
        </section>
    }.into_any()
}

#[component]
pub fn HephaestusTool() -> impl IntoView {
    source_program(
        "hephaestus",
        SAMPLE,
        "One stage per line: stage | parallel stations | whole minutes per item at one station.",
        "https://www.tocinstitute.org/theory-of-constraints.html",
        &[],
        stages,
        capacity,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_upgrade_can_shift_the_constraint() {
        let stages = stages(SAMPLE).ok().unwrap();
        let order = capacity_order(&stages);
        assert_eq!(order, [1, 2, 0]);
        assert_eq!(with_extra_station(&stages, &order, 1), 12.0);
        assert_eq!(with_extra_station(&stages, &order, 0), 7.5);
    }

    #[test]
    fn tied_bottlenecks_remain_and_single_stages_can_double() {
        let rows = stages("A | 1 | 3\nB | 2 | 6").ok().unwrap();
        assert_eq!(with_extra_station(&rows, &capacity_order(&rows), 0), 20.0);
        assert_eq!(with_extra_station(&rows[..1], &[0], 0), 40.0);
        assert!(stages("A | 0 | 1").is_err());
    }
}
