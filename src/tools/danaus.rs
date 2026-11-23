use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;

const SAMPLE: &str =
    "start | 20\nMonday | 12 | 8\nTuesday | 6 | 16\nWednesday | 4 | 24\nThursday | 10 | 4";

#[derive(Clone, PartialEq)]
struct Period {
    name: String,
    inflow: u32,
    released: u64,
    unmet: u64,
    closing: u64,
}

#[derive(Clone, PartialEq)]
struct Ledger {
    initial: u32,
    periods: Vec<Period>,
}

fn simulate(source: &str) -> Result<Ledger, Vec<ParseIssue>> {
    let mut lines = scan(source).into_iter();
    let Some(first) = lines.next() else {
        return Ok(Ledger {
            initial: 0,
            periods: Vec::new(),
        });
    };
    let initial = first
        .content
        .split_once('|')
        .filter(|(key, _)| key.trim() == "start")
        .and_then(|(_, value)| value.trim().parse::<u32>().ok())
        .ok_or_else(|| {
            vec![ParseIssue::error(
                first.number,
                "Start with start | opening quantity.",
            )]
        })?;
    let mut periods = Vec::new();
    let mut stock = u64::from(initial);
    let mut issues = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.content.split('|').map(str::trim).collect();
        if let [name, inflow, demand] = fields.as_slice() {
            if let (Ok(inflow), Ok(demand)) = (inflow.parse::<u32>(), demand.parse::<u32>()) {
                if !name.is_empty() {
                    let available = stock + u64::from(inflow);
                    let released = available.min(u64::from(demand));
                    stock = available - released;
                    periods.push(Period {
                        name: (*name).into(),
                        inflow,
                        released,
                        unmet: u64::from(demand) - released,
                        closing: stock,
                    });
                    continue;
                }
            }
        }
        issues.push(ParseIssue::error(line.number, "Use period | incoming units | requested outgoing units. Enter nonnegative whole numbers."));
    }
    if issues.is_empty() {
        Ok(Ledger { initial, periods })
    } else {
        Err(issues)
    }
}

fn balances(ledger: Ledger, _: RwSignal<String>) -> AnyView {
    let Some(last) = ledger.periods.last() else {
        return view! { <p class="canvas-empty">"Add a period after the opening quantity to simulate the stock."</p> }.into_any();
    };
    let final_stock = last.closing;
    let max = ledger
        .periods
        .iter()
        .map(|p| p.closing)
        .max()
        .unwrap_or(0)
        .max(u64::from(ledger.initial))
        .max(1);
    let rows = ledger.periods.into_iter().map(|period| {
        let width = period.closing as f64 * 100.0 / max as f64;
        view! {
            <li class="schedule-row"><div class="schedule-label"><strong>{period.name}</strong><span>{format!("{} remaining", period.closing)}</span></div>
                <div class="schedule-track" aria-hidden="true"><span style:width=format!("{width}%")></span></div>
                <div>{format!("{} received · {} released", period.inflow, period.released)}
                    {(period.unmet > 0).then(|| format!(" · {} unmet", period.unmet))}
                </div>
            </li>
        }
    }).collect_view();
    view! { <section class="schedule-sheet"><h2>{format!("{final_stock} units remaining")}</h2>
        <p>{format!("Opening stock: {}. Inflow arrives first each period. Unfilled demand expires at the end of that period.", ledger.initial)}</p>
        <ol>{rows}</ol></section> }.into_any()
}

#[component]
pub fn DanausTool() -> impl IntoView {
    source_program("danaus", SAMPLE, "Start with start | quantity. Then enter period | incoming units | requested outgoing units in chronological order.",
        "https://donellameadows.org/wp-content/userfiles/bathtubs101.pdf", &[], simulate, balances)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_releases_and_carries_remaining_stock_forward() {
        let ledger = simulate("start | 3\nA | 2 | 9\nB | 7 | 1").ok().unwrap();
        assert_eq!(
            (
                ledger.periods[0].released,
                ledger.periods[0].unmet,
                ledger.periods[0].closing
            ),
            (5, 4, 0)
        );
        assert_eq!(ledger.periods[1].closing, 6);
    }

    #[test]
    fn rejects_missing_opening_and_negative_flow() {
        assert!(simulate("Monday | 2 | 1").is_err());
        assert!(simulate("start | 2\nMonday | -1 | 0").is_err());
    }
}
