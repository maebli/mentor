//! Agora — CRC card board.
//!
//! Unlike the other tools, Agora is a fully visual, direct-manipulation board:
//! draggable index cards, collaborator "wires" that appear when names match,
//! colour tagging, multi-select, and JSON/Markdown import-export. The board is a
//! self-contained offline HTML/JS app (`public/crc-card-board.html`, copied to
//! the site root by Trunk) which we embed in an iframe here. Keeping it as one
//! self-contained document lets it own its own canvas, drag maths and storage
//! without fighting the Leptos reactive tree.

use crate::registry;
use leptos::prelude::*;

const REFERENCE: &str = "https://en.wikipedia.org/wiki/Class-responsibility-collaboration_card";

#[component]
pub fn AgoraTool() -> impl IntoView {
    let meta = registry::find("agora").expect("agora registered");
    view! {
        <div class="board-tool">
            <header class="board-bar">
                <a class="back" href="#/">"← Mentor"</a>
                <div class="tool-title">
                    <span class="glyph">{meta.glyph}</span>
                    <h1>{meta.greek}</h1>
                    <span class="technique">{meta.title}</span>
                </div>
                <a class="reference" href=REFERENCE target="_blank" rel="noreferrer">
                    "reference ↗"
                </a>
            </header>
            // `src` is relative so it resolves against the document base in both
            // local dev (`/`) and GitHub Pages (`/mentor/`).
            <iframe
                class="board-frame"
                src="crc-card-board.html"
                title="CRC card board"
            ></iframe>
        </div>
    }
}
