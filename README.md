# Mentor 🦉

A fast, offline-friendly suite of **visual tools for planning software** — from
first architecture sketches to root-cause analysis. Named after the guides of
Greek myth (Mentor was Odysseus's trusted advisor, a guise taken by Athena
herself).

Every tool follows the same idea: **you write in a small, syntax-checked text
format on the left, and a live visual renders on the right.** Text is the source
of truth, so your work is diffable, shareable, and never trapped in a canvas.

Built entirely in **Rust → WebAssembly** with [Leptos](https://leptos.dev),
served as a static site on GitHub Pages. No backend, no tracking; your work is
saved locally in your browser.

## The tools

| Greek name | Technique | What it's for |
|---|---|---|
| **Agora** 🏛️ | [CRC cards](https://en.wikipedia.org/wiki/Class-responsibility-collaboration_card) | Class · Responsibility · Collaborator object design |
| **Morpheus** 🔷 | [Morphological analysis](https://en.wikipedia.org/wiki/Morphological_analysis_(problem-solving)) | Parameters × options, explore combinations |
| **Ariadne** 🧵 | [Issue tree](https://en.wikipedia.org/wiki/Issue_tree) | MECE decomposition of a question |
| **Socrates** ❓ | [5 Whys](https://en.wikipedia.org/wiki/Five_whys) | Iterative root-cause questioning |
| **Cetus** 🐟 | [Ishikawa diagram](https://en.wikipedia.org/wiki/Ishikawa_diagram) | Fishbone cause-and-effect analysis |
| **Herakles** 🦁 | [8D report](https://en.wikipedia.org/wiki/Eight_disciplines_problem_solving) | Eight disciplines of problem solving |
| **Cassandra** 🔮 | [Premortem](https://en.wikipedia.org/wiki/Pre-mortem) | Imagine the failure, then rank why |
| **Pythia** 📈 | [Reference-class forecasting](https://en.wikipedia.org/wiki/Reference_class_forecasting) | Estimate against how similar projects went |
| **Themis** ⚖️ | [Weighted decision matrix](https://en.wikipedia.org/wiki/Decision-matrix_method) | Score options against weighted criteria |
| **Tyche** 🎲 | [Probabilistic estimate](https://en.wikipedia.org/wiki/Convolution_of_probability_distributions) | Convolve uncertain task ranges into one total |
| **Metis** 🦉 | [Heuristics](https://en.wikipedia.org/wiki/Heuristic) | Visual reference of rules of thumb |

The front page lists everything, grouped by category, with fuzzy search.

## Develop

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open      # http://localhost:8080
cargo test              # parser unit tests
```

## Architecture

- `src/registry.rs` — the catalogue. Adding a tool is one entry here plus a
  module. The home page, search and nav are all driven from it.
- `src/dsl.rs` — the shared line/indentation scanner and issue types every
  text format is built on.
- `src/ui.rs` — the shared `ToolShell`, editor pane and issue list.
- `src/app.rs` — a tiny hash-based router (chosen so GitHub Pages needs no
  base-path juggling or 404 fallback).
- `src/tools/*.rs` — one module per tool. `agora.rs` is the reference pattern:
  a pure `parse` fn, a `render` fn, and a component wiring them together.

Deploys automatically to GitHub Pages on push to `main` via
`.github/workflows/deploy.yml`.

## License

MIT
