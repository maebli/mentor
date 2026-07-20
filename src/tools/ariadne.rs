//! Ariadne — issue tree (guided MECE decomposition).
//!
//! Each line of text is a node; its indentation gives the parent/child
//! relationship. Exactly one line at the left margin is the root question, and
//! every indented line is a sub-issue of the line above its parent. The text
//! format uses no keywords — the prose itself is the node label:
//! ```text
//! Can we cut cloud spend 30% this quarter?
//!   Reduce compute
//!     Right-size under-utilized instances
//!     Move batch jobs to spot instances
//!   Reduce storage
//!     Delete orphaned snapshots and backups
//! ```

use crate::dsl::{scan, ParseIssue};
use crate::registry;
use crate::ui::{EditorPane, ToolShell};
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Issue_tree";

const HINT: &str = "each line is a node · indent 2 spaces to nest a sub-issue under the line above";

const DEFAULT: &str = "\
Can we cut cloud spend 30% this quarter?
  Reduce compute
    Right-size under-utilized instances
    Move batch jobs to spot instances
  Reduce storage
    Delete orphaned snapshots and backups
    Lower replication factor on cold buckets
  Reduce data transfer
    Serve static assets from a CDN
    Compress API responses
";

#[derive(Clone, PartialEq)]
pub struct Node {
    pub label: String,
    pub children: Vec<Node>,
}

/// Parse an indentation outline into a tree, collecting issues by line.
/// Returns `None` for the root when the input is empty.
pub fn parse(input: &str) -> (Option<Node>, Vec<ParseIssue>) {
    let lines = scan(input);
    let mut issues = Vec::new();
    let mut root: Option<Node> = None;
    // `path` holds the child indices walked from the root down to the most
    // recently added node, so `path.len()` is the depth of that node (0 = root).
    let mut path: Vec<usize> = Vec::new();
    // Once a second root appears we drop everything that follows.
    let mut frozen = false;

    let has_root_line = lines.iter().any(|l| l.indent == 0);
    if !lines.is_empty() && !has_root_line {
        issues.push(ParseIssue::error(
            lines[0].number,
            "the tree needs a root question at the left margin",
        ));
    }

    for line in &lines {
        if frozen {
            continue;
        }
        let indent = line.indent;

        if indent == 0 {
            if root.is_none() {
                root = Some(Node { label: line.content.clone(), children: Vec::new() });
                path.clear();
            } else {
                issues.push(ParseIssue::warn(
                    line.number,
                    "extra root ignored — an issue tree has one root",
                ));
                frozen = true;
            }
            continue;
        }

        let Some(node) = root.as_mut() else {
            // No root yet to attach to; the missing-root error is already filed.
            continue;
        };

        let prev_depth = path.len();
        if indent > prev_depth + 1 {
            issues.push(ParseIssue::error(line.number, "indentation skips a level"));
            continue;
        }

        // The new node lives at depth `indent`, so its parent is at depth
        // `indent - 1`; truncate the path down to that parent.
        path.truncate(indent - 1);

        let parent = node_at_mut(node, &path);
        parent.children.push(Node { label: line.content.clone(), children: Vec::new() });
        path.push(parent.children.len() - 1);
    }

    (root, issues)
}

/// Walk `root` down through `path`, returning a mutable reference to the node
/// at the end of that path. An empty path yields the root itself.
fn node_at_mut<'a>(root: &'a mut Node, path: &[usize]) -> &'a mut Node {
    let mut cur = root;
    for &i in path {
        cur = &mut cur.children[i];
    }
    cur
}

fn render(model: Option<Node>) -> AnyView {
    let Some(root) = model else {
        return view! {
            <p class="canvas-empty">"Write a root question on the left, then indent sub-issues."</p>
        }
        .into_any();
    };
    view! {
        <div class="issue-tree">{render_node(&root)}</div>
    }
    .into_any()
}

fn render_node(node: &Node) -> AnyView {
    let label = node.label.clone();
    if node.children.is_empty() {
        view! {
            <div class="issue-node">
                <div class="issue-box">{label}</div>
            </div>
        }
        .into_any()
    } else {
        let children = node.children.iter().map(render_node).collect_view();
        view! {
            <div class="issue-node">
                <div class="issue-box">{label}</div>
                <div class="issue-children">{children}</div>
            </div>
        }
        .into_any()
    }
}

#[component]
pub fn AriadneTool() -> impl IntoView {
    let meta = registry::find("ariadne").expect("ariadne registered");
    let text = crate::ui::use_persisted("ariadne", DEFAULT);
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
    use crate::dsl::Severity;

    #[test]
    fn parses_default_into_three_branches() {
        let (root, issues) = parse(DEFAULT);
        let root = root.expect("default has a root");
        assert_eq!(root.label, "Can we cut cloud spend 30% this quarter?");
        assert_eq!(root.children.len(), 3);
        for branch in &root.children {
            assert_eq!(branch.children.len(), 2, "each branch has two sub-issues");
            for leaf in &branch.children {
                assert!(leaf.children.is_empty(), "leaves are at depth 2");
            }
        }
        assert!(issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn errors_when_no_root_line() {
        let (_, issues) = parse("  only indented\n  also indented\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("root")));
    }

    #[test]
    fn errors_on_indentation_skip() {
        let (_, issues) = parse("Root\n    too deep\n");
        assert!(issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("skips a level")));
    }
}
