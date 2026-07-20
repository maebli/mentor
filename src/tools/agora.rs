//! Agora — a CRC (Class · Responsibility · Collaborator) card editor.
//!
//! This is the reference implementation every other editor tool follows:
//!   1. a `parse` function: text -> (model, issues), pure and unit-tested;
//!   2. a `render` function: model -> visual view;
//!   3. an `*Tool` component wiring text -> parse -> {editor, visual}.
//!
//! Text format (2-space indentation):
//! ```text
//! card OrderService
//!   responsibility Validate incoming orders
//!   collaborator PaymentGateway
//! ```

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;
use std::collections::HashSet;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Class-responsibility-collaboration_card";

const HINT: &str = "card <Name>  ·  (indent) responsibility <text>  ·  (indent) collaborator <Name>";

const DEFAULT: &str = "\
card OrderService
  responsibility Validate incoming orders
  responsibility Calculate order totals
  collaborator PaymentGateway
  collaborator InventoryService

card PaymentGateway
  responsibility Authorize payment
  responsibility Record transactions
  collaborator BankAdapter

card InventoryService
  responsibility Reserve stock for an order
  collaborator WarehouseClient
";

#[derive(Clone, PartialEq)]
pub struct Card {
    pub name: String,
    pub responsibilities: Vec<String>,
    pub collaborators: Vec<String>,
}

/// Parse CRC text into cards, collecting syntax and reference issues.
pub fn parse(input: &str) -> (Vec<Card>, Vec<ParseIssue>) {
    let mut cards: Vec<Card> = Vec::new();
    let mut issues: Vec<ParseIssue> = Vec::new();
    // (line, collaborator name) pairs, checked once all names are known.
    let mut references: Vec<(usize, String)> = Vec::new();

    for line in scan(input) {
        match line.indent {
            0 => {
                let (kw, rest) = line.keyword();
                if kw == "card" {
                    if rest.is_empty() {
                        issues.push(ParseIssue::error(line.number, "`card` needs a name"));
                        continue;
                    }
                    if cards.iter().any(|c| c.name == rest) {
                        issues.push(ParseIssue::warn(
                            line.number,
                            format!("duplicate card “{rest}”"),
                        ));
                    }
                    cards.push(Card {
                        name: rest.to_string(),
                        responsibilities: Vec::new(),
                        collaborators: Vec::new(),
                    });
                } else {
                    issues.push(ParseIssue::error(
                        line.number,
                        format!("expected `card <Name>`, found “{kw}”"),
                    ));
                }
            }
            1 => {
                let Some(card) = cards.last_mut() else {
                    issues.push(ParseIssue::error(
                        line.number,
                        "this line must sit inside a `card`",
                    ));
                    continue;
                };
                let (kw, rest) = line.keyword();
                match kw {
                    "responsibility" | "resp" => {
                        if rest.is_empty() {
                            issues.push(ParseIssue::warn(line.number, "empty responsibility"));
                        } else {
                            card.responsibilities.push(rest.to_string());
                        }
                    }
                    "collaborator" | "collab" => {
                        if rest.is_empty() {
                            issues.push(ParseIssue::warn(line.number, "empty collaborator"));
                        } else {
                            card.collaborators.push(rest.to_string());
                            references.push((line.number, rest.to_string()));
                        }
                    }
                    _ => issues.push(ParseIssue::error(
                        line.number,
                        format!("expected `responsibility` or `collaborator`, found “{kw}”"),
                    )),
                }
            }
            _ => issues.push(ParseIssue::error(
                line.number,
                "indented too deeply — use one level under a card",
            )),
        }
    }

    let names: HashSet<&str> = cards.iter().map(|c| c.name.as_str()).collect();
    for (line, name) in references {
        if !names.contains(name.as_str()) {
            issues.push(ParseIssue::warn(
                line,
                format!("collaborator “{name}” has no card of its own"),
            ));
        }
    }

    (cards, issues)
}

fn render(cards: Vec<Card>) -> AnyView {
    if cards.is_empty() {
        return view! {
            <p class="canvas-empty">"Describe a card on the left to see it here."</p>
        }
        .into_any();
    }
    let names: HashSet<String> = cards.iter().map(|c| c.name.clone()).collect();
    let views = cards
        .into_iter()
        .map(|c| {
            let resp = c
                .responsibilities
                .into_iter()
                .map(|r| view! { <li>{r}</li> })
                .collect_view();
            let names = names.clone();
            let collab = c
                .collaborators
                .into_iter()
                .map(move |name| {
                    let known = names.contains(&name);
                    let cls = if known { "collab known" } else { "collab unknown" };
                    view! { <li class=cls>{name}</li> }
                })
                .collect_view();
            view! {
                <article class="crc-card">
                    <header class="crc-name">{c.name}</header>
                    <div class="crc-body">
                        <div class="crc-col">
                            <h4>"Responsibilities"</h4>
                            <ul>{resp}</ul>
                        </div>
                        <div class="crc-col">
                            <h4>"Collaborators"</h4>
                            <ul>{collab}</ul>
                        </div>
                    </div>
                </article>
            }
        })
        .collect_view();
    view! { <div class="crc-grid">{views}</div> }.into_any()
}

#[component]
pub fn AgoraTool() -> impl IntoView {
    let meta = registry::find("agora").expect("agora registered");
    let text = crate::ui::use_persisted("agora", DEFAULT);
    let parsed = Memo::new(move |_| parse(&text.get()));
    let issues = Signal::derive(move || parsed.get().1);

    let left = view! {
        <EditorPane text=text issues=issues syntax_hint=HINT />
    }
    .into_any();
    let right = view! {
        <div class="canvas">{move || render(parsed.get().0)}</div>
    }
    .into_any();

    view! { <ToolShell meta=meta reference=REFERENCE left=left right=right /> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cards_and_members() {
        let (cards, issues) = parse(DEFAULT);
        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].name, "OrderService");
        assert_eq!(cards[0].responsibilities.len(), 2);
        assert_eq!(cards[0].collaborators, vec!["PaymentGateway", "InventoryService"]);
        assert!(issues.iter().all(|i| i.severity == crate::dsl::Severity::Warning
            || i.message.is_empty()));
    }

    #[test]
    fn warns_on_unknown_collaborator() {
        let (_, issues) = parse("card A\n  collaborator Ghost\n");
        assert!(issues.iter().any(|i| i.message.contains("Ghost")));
    }

    #[test]
    fn errors_on_member_without_card() {
        let (_, issues) = parse("  responsibility orphan\n");
        assert!(issues.iter().any(|i| i.message.contains("inside a `card`")));
    }
}
