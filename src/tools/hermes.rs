use super::source_program;
use crate::dsl::{scan, ParseIssue};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Element, PointerEvent};

const DEFAULT: &str = "\
column Ready
  card Review launch copy
  card Prepare demo

column In progress | 2
  card Test signup flow

column Done
  card Agree on release scope
";

#[derive(Clone, PartialEq)]
struct Column {
    name: String,
    header: usize,
    limit: Option<usize>,
    cards: Vec<(usize, String)>,
}

impl Column {
    fn full(&self) -> bool {
        self.limit.is_some_and(|limit| self.cards.len() >= limit)
    }

    fn insert_after(&self) -> usize {
        self.cards.last().map_or(self.header, |card| card.0)
    }
}

fn board(source: &str) -> Result<Vec<Column>, Vec<ParseIssue>> {
    let mut columns = Vec::<Column>::new();
    let mut issues = Vec::new();
    for line in scan(source) {
        let (keyword, value) = line.keyword();
        match (line.indent, keyword) {
            (0, "column") => {
                let (name, limit) = value
                    .split_once('|')
                    .map_or((value, None), |(name, count)| {
                        (name.trim(), Some(count.trim()))
                    });
                let limit = match limit.map(str::parse::<usize>).transpose() {
                    Ok(limit) => limit,
                    Err(_) => {
                        issues.push(ParseIssue::error(
                            line.number,
                            "Use a whole number for the column limit.",
                        ));
                        continue;
                    }
                };
                if name.is_empty() {
                    issues.push(ParseIssue::error(line.number, "Give the column a name."));
                }
                columns.push(Column {
                    name: name.into(),
                    header: line.number,
                    limit,
                    cards: Vec::new(),
                });
            }
            (1, "card") if !value.is_empty() && !columns.is_empty() => {
                if let Some(column) = columns.last_mut() {
                    column.cards.push((line.number, value.into()));
                }
            }
            _ => issues.push(ParseIssue::error(
                line.number,
                "Use column <name> and indent card <title> under its column.",
            )),
        }
    }
    if issues.is_empty() {
        Ok(columns)
    } else {
        Err(issues)
    }
}

fn move_card(source: &str, from_line: usize, destination: &Column) -> String {
    let separator = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut lines: Vec<_> = source.lines().collect();
    let card = lines.remove(from_line - 1);
    let anchor = destination.insert_after();
    lines.insert(anchor - usize::from(from_line <= anchor), card);
    let mut result = lines.join(separator);
    if source.ends_with('\n') {
        result.push_str(separator);
    }
    result
}

fn transfer(text: RwSignal<String>, line: usize, title: &str, destination: usize) {
    let source = text.get_untracked();
    let Ok(columns) = board(&source) else { return };
    let Some(target) = columns
        .iter()
        .find(|c| c.header == destination && !c.full())
    else {
        return;
    };
    let belongs_elsewhere = columns
        .iter()
        .filter(|c| c.header != destination)
        .flat_map(|c| &c.cards)
        .any(|(number, name)| *number == line && name == title);
    if belongs_elsewhere {
        text.set(move_card(&source, line, target));
    }
}

struct Drag {
    pointer: i32,
    line: usize,
    title: String,
    x: i32,
    y: i32,
}

fn lane_at(event: &PointerEvent) -> Option<usize> {
    document()
        .element_from_point(event.client_x() as f32, event.client_y() as f32)?
        .closest(".kanban-lane")
        .ok()??
        .get_attribute("data-lane")?
        .parse()
        .ok()
}

fn move_button(
    label: &str,
    line: usize,
    target: Option<&Column>,
    symbol: &'static str,
    text: RwSignal<String>,
) -> AnyView {
    let Some(target) = target else {
        return ().into_any();
    };
    let destination = target.header;
    let expected_title = label.to_owned();
    let name = format!("Move {label} to {}", target.name);
    let blocked = target.full();
    view! {
        <button disabled=blocked aria-label=name.clone() title=name on:click=move |_| {
            transfer(text, line, &expected_title, destination);
        }>{symbol}</button>
    }
    .into_any()
}

fn render(columns: Vec<Column>, text: RwSignal<String>) -> AnyView {
    if columns.is_empty() {
        return view! { <p class="canvas-empty">"Add a column to start a board."</p> }.into_any();
    }
    let drag = RwSignal::new(None::<Drag>);
    let offset = RwSignal::new((0, 0));
    let hovered = RwSignal::new(None::<usize>);
    let clear = move || {
        drag.set(None);
        hovered.set(None);
        offset.set((0, 0));
    };
    let motion = move |event: PointerEvent| {
        drag.with_untracked(|active| {
            if let Some(active) = active.as_ref().filter(|a| a.pointer == event.pointer_id()) {
                offset.set((event.client_x() - active.x, event.client_y() - active.y));
                hovered.set(lane_at(&event));
            }
        });
    };
    let drop_card = move |event: PointerEvent| {
        let moved = drag.with_untracked(|active| {
            active
                .as_ref()
                .filter(|a| a.pointer == event.pointer_id())
                .map(|a| (a.line, a.title.clone()))
        });
        if let Some((line, title)) = moved {
            let destination = lane_at(&event);
            clear();
            if let Some(destination) = destination {
                transfer(text, line, &title, destination);
            }
        }
    };
    let lanes = columns.iter().enumerate().map(|(index, column)| {
        let count = column.cards.len();
        let capacity = match (column.limit, count) {
            (Some(limit), _) => format!("{count} / {limit}"),
            (None, 1) => "1 card".into(),
            (None, _) => format!("{count} cards"),
        };
        let cards = column.cards.iter().map(|(line, title)| {
            let back = move_button(title, *line, index.checked_sub(1).map(|i| &columns[i]), "←", text);
            let forward = move_button(title, *line, columns.get(index + 1), "→", text);
            let line = *line;
            let label = title.clone();
            let dragging = move || drag.with(|active| active.as_ref().is_some_and(|a| a.line == line));
            let start = move |event: PointerEvent| {
                if !event.is_primary() || event.button() != 0 { return }
                let Some(hit) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else { return };
                if hit.closest("button").ok().flatten().is_some() { return }
                let Some(card) = event.current_target().and_then(|t| t.dyn_into::<Element>().ok()) else { return };
                if card.set_pointer_capture(event.pointer_id()).is_err() { return }
                event.prevent_default();
                drag.set(Some(Drag { pointer: event.pointer_id(), line, title: label.clone(), x: event.client_x(), y: event.client_y() }));
            };
            view! {
                <li class:dragging=dragging on:pointerdown=start
                    style:translate=move || if dragging() {
                        let (x, y) = offset.get();
                        format!("{x}px {y}px")
                    } else { String::new() }>
                    <strong>{title.clone()}</strong><div class="kanban-moves">{back}{forward}</div>
                </li>
            }
        }).collect_view();
        let header = column.header;
        let full = column.full();
        view! {
            <section class="kanban-lane" data-lane=header
                class:drop-ready=move || !full && hovered.get() == Some(header)
                class:drop-full=move || full && hovered.get() == Some(header)>
                <h2>{column.name.clone()}</h2><p>{capacity}</p><ul>{cards}</ul>
            </section>
        }
    }).collect_view();
    view! { <div class="kanban-board" on:pointermove=motion on:pointerup=drop_card
        on:lostpointercapture=move |ev: PointerEvent| {
            if drag.with_untracked(|a| a.as_ref().is_some_and(|a| a.pointer == ev.pointer_id())) { clear(); }
        }>{lanes}</div> }.into_any()
}

#[component]
pub fn HermesTool() -> impl IntoView {
    source_program("hermes", DEFAULT, "Use column <name> and indent card <title> by two spaces. Add | 2 to a column to limit it to two cards.", "https://kanban.university/kanban-guide/", &["column", "card"], board, render)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_one_duplicate_title_and_preserves_notes_and_newlines() {
        let source = "column Ready\r\n  card A\r\n# note\r\n  card A\r\ncolumn Done\r\n";
        let columns = board(source).ok().unwrap();
        let moved = move_card(source, 4, &columns[1]);
        assert_eq!(
            moved,
            "column Ready\r\n  card A\r\n# note\r\ncolumn Done\r\n  card A\r\n"
        );
        let columns = board(&moved).ok().unwrap();
        assert_eq!(
            move_card(&moved, 5, &columns[0]),
            "column Ready\r\n  card A\r\n  card A\r\n# note\r\ncolumn Done\r\n"
        );
    }

    #[test]
    fn recognizes_full_columns_and_invalid_cards() {
        assert!(board("column Doing | 1\n  card A").ok().unwrap()[0].full());
        assert!(board("  card Missing column").is_err());
        assert!(board("column Doing | many").is_err());
    }
}
