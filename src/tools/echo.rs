use super::source_program;
use crate::dsl::ParseIssue;
use leptos::prelude::*;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

#[derive(Clone, PartialEq)]
struct Code {
    symbol: char,
    count: usize,
    bits: String,
}

#[derive(Clone, PartialEq)]
struct Encoding {
    codes: Vec<Code>,
    encoded_bits: u64,
    fixed_bits: u64,
    preview: String,
}

fn encode(source: &str) -> Result<Encoding, Vec<ParseIssue>> {
    let mut counts = BTreeMap::new();
    for symbol in source.chars() {
        *counts.entry(symbol).or_insert(0usize) += 1;
    }
    let symbols = counts.len();
    let mut parents = vec![None; symbols];
    let mut queue: BinaryHeap<_> = counts
        .values()
        .enumerate()
        .map(|(index, &count)| Reverse((count, index)))
        .collect();
    while queue.len() > 1 {
        let Reverse((a, left)) = queue.pop().unwrap();
        let Reverse((b, right)) = queue.pop().unwrap();
        let parent = parents.len();
        parents.push(None);
        parents[left] = Some((parent, '0'));
        parents[right] = Some((parent, '1'));
        queue.push(Reverse((a + b, parent)));
    }
    let mut codes = Vec::new();
    let mut encoded_bits = 0;
    let mut characters = 0;
    for (index, (symbol, count)) in counts.into_iter().enumerate() {
        let mut reversed = Vec::new();
        let mut node = index;
        while let Some((parent, bit)) = parents[node] {
            reversed.push(bit);
            node = parent;
        }
        if reversed.is_empty() {
            reversed.push('0');
        }
        let bits: String = reversed.into_iter().rev().collect();
        encoded_bits += count as u64 * bits.len() as u64;
        characters += count as u64;
        codes.push(Code {
            symbol,
            count,
            bits,
        });
    }
    let mut preview = String::new();
    for symbol in source.chars() {
        let index = codes
            .binary_search_by_key(&symbol, |code| code.symbol)
            .unwrap();
        preview.extend(codes[index].bits.chars().take(96 - preview.len()));
        if preview.len() == 96 {
            break;
        }
    }
    codes.sort_by_key(|code| (Reverse(code.count), code.symbol));
    let fixed_width = symbols
        .saturating_sub(1)
        .checked_ilog2()
        .map_or(1, |n| n + 1);
    Ok(Encoding {
        codes,
        encoded_bits,
        fixed_bits: characters * u64::from(fixed_width),
        preview,
    })
}

fn codebook(model: Encoding, _: RwSignal<String>) -> AnyView {
    if model.codes.is_empty() {
        return view! { <p class="canvas-empty">"Type a message to build its codebook."</p> }
            .into_any();
    }
    let rows = model.codes.into_iter().map(|code| {
        let label = match code.symbol {
            ' ' => "Space".into(),
            '\n' => "Newline".into(),
            '\t' => "Tab".into(),
            c if c.is_control() || c.is_whitespace() => format!("U+{:04X}", c as u32),
            c => c.to_string(),
        };
        view! { <tr><th scope="row">{label}</th><td>{code.count}</td><td><code>{code.bits}</code></td></tr> }
    }).collect_view();
    view! {
        <section class="schedule-sheet"><h2>{format!("{} encoded bits", model.encoded_bits)}</h2>
            <p>{format!("Fixed-width coding of the same alphabet uses {} bits. Both totals cover the message payload only.", model.fixed_bits)}</p>
            <p class="encoded-preview"><code>{model.preview}{(model.encoded_bits > 96).then_some("…")}</code></p>
            <table class="codebook"><thead><tr><th scope="col">"Symbol"</th><th scope="col">"Count"</th><th scope="col">"Code"</th></tr></thead>
                <tbody>{rows}</tbody>
            </table>
        </section>
    }.into_any()
}

#[component]
pub fn EchoTool() -> impl IntoView {
    source_program("echo", "BANANA BANDANA", "Type the message itself. Each Unicode character counts as one symbol, including spaces and newlines.",
        "https://sicp.sourceacademy.org/chapters/2.3.4.html", &[], encode, codebook)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_prefix_free_codes_and_the_weighted_bit_count() {
        let model = encode("AAAAABBC").ok().unwrap();
        assert_eq!(
            (model.encoded_bits, model.fixed_bits, model.preview.len()),
            (11, 16, 11)
        );
        for a in &model.codes {
            for b in &model.codes {
                assert!(a.symbol == b.symbol || !b.bits.starts_with(&a.bits));
            }
        }
    }

    #[test]
    fn handles_unicode_single_symbol_and_truncated_previews() {
        let model = encode(&"🦉".repeat(100)).ok().unwrap();
        assert_eq!(model.encoded_bits, 100);
        assert_eq!(model.preview, "0".repeat(96));
        assert!(encode("").ok().unwrap().codes.is_empty());
    }
}
