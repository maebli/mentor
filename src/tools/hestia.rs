use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;

const DEFAULT: &str = "\
08:30-10:00 | Design the data model
10:15-11:00 | Review the migration plan
11:00-11:30 | Reply to the support queue
13:00-14:30 | Prototype the import flow
";

#[derive(Clone, PartialEq)]
struct Block {
    start: u16,
    end: u16,
    title: String,
    line: usize,
}

fn minute(clock: &str) -> Option<u16> {
    let (h, m) = clock.trim().split_once(':')?;
    if h.len() != 2 || m.len() != 2 || !h.bytes().chain(m.bytes()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    let hour = h.parse::<u16>().ok()?;
    let minute = m.parse::<u16>().ok()?;
    (minute < 60 && (hour < 24 || (hour == 24 && minute == 0))).then(|| hour * 60 + minute)
}

fn day(source: &str) -> Result<Vec<Block>, Vec<ParseIssue>> {
    let mut blocks = Vec::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        let entry = line.content.split_once('|').and_then(|(range, title)| {
            let (start, end) = range.split_once('-')?;
            let (start, end) = (minute(start)?, minute(end)?);
            (start < end && !title.trim().is_empty()).then(|| Block {
                start,
                end,
                title: title.trim().into(),
                line: line.number,
            })
        });
        match entry {
            Some(block) => blocks.push(block),
            None => issues.push(ParseIssue::error(
                line.number,
                "Enter HH:MM-HH:MM | activity within one day. End after the start.",
            )),
        }
    }
    blocks.sort_by_key(|block| block.start);
    let mut busy_until = 0;
    for block in &blocks {
        if block.start < busy_until {
            issues.push(ParseIssue::error(
                block.line,
                format!("“{}” overlaps an earlier block.", block.title),
            ));
        }
        busy_until = busy_until.max(block.end);
    }
    if issues.is_empty() {
        Ok(blocks)
    } else {
        Err(issues)
    }
}

fn clock(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

fn agenda(blocks: Vec<Block>, _: RwSignal<String>) -> AnyView {
    let Some(first) = blocks.first() else {
        return view! { <p class="canvas-empty">"Enter a time range and an activity to plan your day."</p> }.into_any();
    };
    let mut previous_end = first.start;
    let total: u16 = blocks.iter().map(|b| b.end - b.start).sum();
    let rows = blocks.into_iter().map(|block| {
        let gap = block.start - previous_end;
        previous_end = block.end;
        view! {
            <li>
                {(gap > 0).then(|| view! { <p class="agenda-gap">{format!("{gap} min available")}</p> })}
                <div class="agenda-entry">
                    <time>{clock(block.start)}</time><strong>{block.title}</strong>
                    <span>{format!("{}–{} · {} min", clock(block.start), clock(block.end), block.end-block.start)}</span>
                </div>
            </li>
        }
    }).collect_view();
    view! { <section class="schedule-sheet"><h2>"Daily plan"</h2>
    <p>{format!("{} h {} min scheduled", total / 60, total % 60)}</p><ol>{rows}</ol></section> }
    .into_any()
}

#[component]
pub fn HestiaTool() -> impl IntoView {
    source_program(
        "hestia",
        DEFAULT,
        "One block per line. Enter HH:MM-HH:MM | activity in 24-hour time.",
        "https://calnewport.com/text-file-time-blocking/",
        &[],
        day,
        agenda,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_blocks_and_accepts_touching_times_and_midnight_end() {
        let blocks = day("23:30-24:00 | Close\n22:15-23:30 | Read").ok().unwrap();
        assert_eq!(blocks[0].title, "Read");
        assert_eq!(blocks[1].end, 1440);
    }

    #[test]
    fn flags_overlaps_and_bad_times() {
        let issues = day("09:00-12:00 | A\n10:00-10:30 | B\n11:00-11:30 | C")
            .err()
            .unwrap();
        assert_eq!(issues.iter().map(|i| i.line).collect::<Vec<_>>(), [2, 3]);
        assert!(day("10:70-12:00 | Invalid").is_err());
    }
}
