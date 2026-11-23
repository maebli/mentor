use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;
use std::collections::{HashMap, VecDeque};

const DEFAULT: &str = "\
Scope release | 2
Build feature | 5 | Scope release
Draft guide | 3 | Scope release
Test feature | 2 | Build feature
Publish | 1 | Test feature, Draft guide
";

#[derive(Clone, PartialEq)]
struct Task {
    name: String,
    days: u32,
    after: Vec<String>,
    line: usize,
}

#[derive(Clone, PartialEq)]
struct Activity {
    task: Task,
    start: u64,
    finish: u64,
    slack: u64,
}

#[derive(Clone, Default, PartialEq)]
struct Schedule {
    days: u64,
    activities: Vec<Activity>,
}

fn parse_task(line: &str, number: usize) -> Result<Task, ParseIssue> {
    let fields: Vec<_> = line.split('|').map(str::trim).collect();
    if !(2..=3).contains(&fields.len()) || fields[0].is_empty() || fields[0].contains(',') {
        return Err(ParseIssue::error(
            number,
            "Use task name | days | optional predecessors. Use commas only between predecessors.",
        ));
    }
    let days = fields[1].parse::<u32>().map_err(|_| {
        ParseIssue::error(number, "Enter a duration in whole days, zero or greater.")
    })?;
    let mut after = Vec::new();
    if let Some(list) = fields.get(2).filter(|list| !list.is_empty()) {
        for name in list.split(',').map(str::trim) {
            if name.is_empty() {
                return Err(ParseIssue::error(number, "A predecessor name is missing."));
            }
            after.push(name.to_owned());
        }
    }
    Ok(Task {
        name: fields[0].into(),
        days,
        after,
        line: number,
    })
}

fn schedule(source: &str) -> Result<Schedule, Vec<ParseIssue>> {
    let mut tasks = Vec::new();
    let mut indices = HashMap::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        match parse_task(&line.content, line.number) {
            Ok(task) if indices.contains_key(&task.name) => issues.push(ParseIssue::error(
                line.number,
                format!("The task “{}” already exists.", task.name),
            )),
            Ok(task) => {
                indices.insert(task.name.clone(), tasks.len());
                tasks.push(task);
            }
            Err(issue) => issues.push(issue),
        }
    }
    let mut successors = vec![Vec::new(); tasks.len()];
    let mut remaining = Vec::new();
    for (index, task) in tasks.iter().enumerate() {
        remaining.push(task.after.len());
        for name in &task.after {
            match indices.get(name) {
                Some(&before) => successors[before].push(index),
                None => issues.push(ParseIssue::error(
                    task.line,
                    format!("Unknown predecessor “{name}”."),
                )),
            }
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let mut ready: VecDeque<_> = (0..tasks.len()).filter(|&i| remaining[i] == 0).collect();
    let mut order = Vec::new();
    let mut start = vec![0; tasks.len()];
    let mut days = 0;
    while let Some(i) = ready.pop_front() {
        order.push(i);
        let finish = start[i] + u64::from(tasks[i].days);
        days = days.max(finish);
        for &next in &successors[i] {
            start[next] = start[next].max(finish);
            remaining[next] -= 1;
            if remaining[next] == 0 {
                ready.push_back(next);
            }
        }
    }
    if order.len() != tasks.len() {
        return Err(tasks
            .iter()
            .zip(remaining)
            .filter(|(_, pending)| *pending > 0)
            .map(|(task, _)| ParseIssue::error(task.line, "A dependency cycle blocks this task."))
            .collect());
    }
    let mut latest_start = vec![0; tasks.len()];
    for &i in order.iter().rev() {
        let latest_finish = successors[i]
            .iter()
            .map(|&next| latest_start[next])
            .min()
            .unwrap_or(days);
        latest_start[i] = latest_finish - u64::from(tasks[i].days);
    }
    let activities = order
        .into_iter()
        .map(|i| Activity {
            task: tasks[i].clone(),
            start: start[i],
            finish: start[i] + u64::from(tasks[i].days),
            slack: latest_start[i] - start[i],
        })
        .collect();
    Ok(Schedule { days, activities })
}

fn timeline(model: Schedule, _: RwSignal<String>) -> AnyView {
    if model.activities.is_empty() {
        return view! { <p class="canvas-empty">"Enter a task and its duration to start a schedule."</p> }.into_any();
    }
    let span = model.days.max(1) as f64;
    let rows = model.activities.into_iter().map(|activity| {
        let offset = activity.start as f64 / span * 100.0;
        let width = f64::from(activity.task.days) / span * 100.0;
        let status = if activity.slack == 0 { "Critical".into() } else { format!("{} d slack", activity.slack) };
        view! {
            <li class="schedule-row" class:critical=activity.slack == 0>
                <div class="schedule-label"><strong>{activity.task.name}</strong><span>{status}</span></div>
                <div class="schedule-track" aria-hidden="true">
                    <span style:left=format!("{offset}%") style:width=format!("{width}%")></span>
                </div>
                <div class="schedule-time">{format!("Day {} → {}", activity.start, activity.finish)}</div>
            </li>
        }
    }).collect_view();
    view! {
        <section class="schedule-sheet">
            <h2>{format!("Finish on day {}", model.days)}</h2>
            <p>"Tasks start as soon as their predecessors finish. Critical tasks have zero slack."</p>
            <ol>{rows}</ol>
        </section>
    }.into_any()
}

#[component]
pub fn ChronosTool() -> impl IntoView {
    source_program("chronos", DEFAULT, "Enter task name | whole days | predecessors separated by commas. Independent tasks need only a name and days.", "https://www.gao.gov/assets/680/674404.pdf", &[], schedule, timeline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_forward_references_and_parallel_paths() {
        let model = schedule("Ship | 0 | Build, Docs\nDocs | 2\nBuild | 4")
            .ok()
            .unwrap();
        assert_eq!(model.days, 4);
        assert_eq!(
            model
                .activities
                .iter()
                .find(|a| a.task.name == "Docs")
                .unwrap()
                .slack,
            2
        );
        assert_eq!(model.activities.last().unwrap().start, 4);
        let tied = schedule("Left | 2\nRight | 2").ok().unwrap();
        assert!(tied.activities.iter().all(|a| a.slack == 0));
    }

    #[test]
    fn rejects_cycles_unknown_names_and_duplicate_tasks() {
        for source in ["A | 1 | B\nB | 2 | A", "A | 1 | Lost", "A | 1\nA | 2"] {
            assert!(schedule(source).is_err());
        }
    }
}
