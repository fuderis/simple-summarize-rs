use super::*;
use ::markdown::{ParseOptions, mdast::Node, to_mdast};
use regex::Regex;
use std::collections::{BinaryHeap, HashSet};
use std::sync::OnceLock;

/// Represents intermediate Markdown blocks extracted during AST parsing.
#[derive(Debug, Clone)]
enum MarkdownBlock {
    Heading(u8, String),
    Code(Option<String>, String), // (language, code_content)
    Table(String),
    Prose(String, u8), // (text, parent_heading_level)
}

/// Represents a discrete structural unit of Markdown text prepared for scoring and selection.
#[derive(Debug, Clone)]
struct MarkdownUnit {
    display_text: String,
    score_text: String,
    is_atomic: bool,
    heading_weight: f32,
    /// Reference index to an associated atomic block (e.g., Code/Table).
    linked_atomic_idx: Option<usize>,
}

/// Sanitizes prose text by stripping timecodes and Markdown links while preserving bracket balance and sentence structure.
pub fn sanitize_markdown_text(input: &str) -> String {
    static TIMECODE_RE: OnceLock<Regex> = OnceLock::new();
    static MD_LINK_RE: OnceLock<Regex> = OnceLock::new();
    static EXTRA_PAREN_RE: OnceLock<Regex> = OnceLock::new();
    static MULTI_SPACE_RE: OnceLock<Regex> = OnceLock::new();

    let timecode_re = TIMECODE_RE.get_or_init(|| {
        Regex::new(r"(?:\s*:\u{200a}?\d{1,2}:\d{2}(?::\d{2})?|\b\d{1,2}:\d{2}(?::\d{2})?\b)")
            .unwrap()
    });

    let md_link_re = MD_LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap());
    let extra_paren_re = EXTRA_PAREN_RE.get_or_init(|| Regex::new(r"\(\s*\)").unwrap());
    let multi_space_re = MULTI_SPACE_RE.get_or_init(|| Regex::new(r"\s+").unwrap());

    // 1. remove timecodes
    let cleaned_timecodes = timecode_re.replace_all(input, "");

    // 2. normalize markdown links: [Text](link) -> Text
    let cleaned_links = md_link_re.replace_all(&cleaned_timecodes, "$1");

    // 3. clean up empty parentheses left behind by stripped links/text
    let cleaned_parens = extra_paren_re.replace_all(&cleaned_links, "");

    // 4. normalize whitespace sequences
    let result = multi_space_re.replace_all(&cleaned_parens, " ");

    result.trim().to_string()
}

/// Extracts clean prose text from AST nodes without sanitizing code contents.
fn extract_prose_text(node: &Node) -> String {
    let mut out = String::new();
    collect_prose_text(node, &mut out);
    sanitize_markdown_text(&out)
}

/// Traverses nodes recursively to accumulate plain text from prose and inline elements.
fn collect_prose_text(node: &Node, acc: &mut String) {
    match node {
        Node::Text(t) => acc.push_str(&t.value),
        Node::InlineCode(c) => {
            acc.push('`');
            acc.push_str(&c.value);
            acc.push('`');
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    collect_prose_text(child, acc);
                }
            }
        }
    }
}

/// Serializes a Markdown table AST node into valid Markdown table syntax.
fn format_mdast_table(table_node: &Node) -> String {
    let mut rows_text: Vec<Vec<String>> = Vec::new();

    if let Some(children) = table_node.children() {
        for row in children {
            if let Node::TableRow(_) = row {
                let mut row_cells = Vec::new();
                if let Some(cells) = row.children() {
                    for cell in cells {
                        let text = extract_prose_text(cell)
                            .replace('\n', " ")
                            .replace('\r', "");
                        let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
                        row_cells.push(clean);
                    }
                }
                rows_text.push(row_cells);
            }
        }
    }

    if rows_text.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    let cols = rows_text[0].len();

    // build header row
    out.push_str("| ");
    out.push_str(&rows_text[0].join(" | "));
    out.push_str(" |\n");

    // build delimiter row
    out.push('|');
    for _ in 0..cols {
        out.push_str("---|");
    }
    out.push('\n');

    // build data rows
    for row in rows_text.iter().skip(1) {
        if row.is_empty() {
            continue;
        }
        out.push_str("| ");
        out.push_str(&row.join(" | "));
        out.push_str(" |\n");
    }

    out.trim_end().to_string()
}

/// Recursively traverses the AST to extract block elements.
fn traverse_ast(node: &Node, current_h_level: u8, blocks: &mut Vec<MarkdownBlock>) {
    match node {
        Node::Heading(h) => {
            // current_h_level = h.depth;
            let text = extract_prose_text(node).trim().to_string();
            if !text.is_empty() {
                blocks.push(MarkdownBlock::Heading(h.depth, text));
            }
        }
        Node::Code(c) => {
            let lang = c.lang.clone();
            blocks.push(MarkdownBlock::Code(lang, c.value.clone()));
        }
        Node::Table(_) => {
            let table_md = format_mdast_table(node);
            if !table_md.is_empty() {
                blocks.push(MarkdownBlock::Table(table_md));
            }
        }
        Node::Paragraph(_) => {
            let text = extract_prose_text(node).trim().to_string();
            if !text.is_empty() {
                blocks.push(MarkdownBlock::Prose(text, current_h_level));
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    traverse_ast(child, current_h_level, blocks);
                }
            }
        }
    }
}

/// Parses raw Markdown text into a sequence of [`MarkdownBlock`] nodes.
fn extract_markdown_blocks(markdown: &str) -> Vec<MarkdownBlock> {
    let mut options = ParseOptions::gfm();
    options.constructs.gfm_table = true;

    let mut blocks = Vec::new();
    if let Ok(ast) = to_mdast(markdown, &options) {
        traverse_ast(&ast, 0, &mut blocks);
    }
    blocks
}

/// Checks a sentence candidate for syntactic validity and completeness.
fn is_valid_sentence(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.len() < 10 {
        return false;
    }

    let words_count = trimmed.split_whitespace().count();
    if words_count < 3 {
        return false;
    }

    // verify balanced parentheses
    let mut balance = 0i32;
    for ch in trimmed.chars() {
        if ch == '(' {
            balance += 1;
        } else if ch == ')' {
            balance -= 1;
            if balance < 0 {
                return false;
            }
        }
    }
    if balance != 0 {
        return false;
    }

    // verify balanced backticks
    if trimmed.matches('`').count() % 2 != 0 {
        return false;
    }

    true
}

/// Performs extractive summarization on a Markdown document.
///
/// # Arguments
/// * `input_text` - The raw Markdown content to be summarized.
/// * `cut_ratio` - Target reduction ratio as a fraction from 0.0 to 1.0 (e.g., 0.20 for 20%).
pub fn summarize_markdown(input_text: &str, cut_ratio: f32) -> Result<SummaryReport> {
    // normalize cut ratio, bounding it strictly between 0.0 and 0.95
    let target_cut = cut_ratio.clamp(0.0, 0.95);
    let target_ratio = 1.0 - target_cut;

    let blocks = extract_markdown_blocks(input_text);
    let mut units: Vec<MarkdownUnit> = Vec::new();
    let mut prose_for_keywords = String::new();

    for block in blocks {
        match block {
            MarkdownBlock::Code(lang, code_content) => {
                let language_str = lang.as_deref().unwrap_or("");
                let formatted_code = format!("```{}\n{}\n```", language_str, code_content.trim());

                units.push(MarkdownUnit {
                    display_text: formatted_code,
                    score_text: String::new(),
                    is_atomic: true,
                    heading_weight: 1.0,
                    linked_atomic_idx: None,
                });
            }
            MarkdownBlock::Table(table_content) => {
                prose_for_keywords.push_str(&table_content);
                prose_for_keywords.push('\n');
                units.push(MarkdownUnit {
                    display_text: table_content.clone(),
                    score_text: table_content,
                    is_atomic: true,
                    heading_weight: 1.1,
                    linked_atomic_idx: None,
                });
            }
            MarkdownBlock::Heading(level, text) => {
                prose_for_keywords.push_str(&text);
                prose_for_keywords.push('\n');
                let heading_md = format!("{} {}", "#".repeat(level as usize), text);

                let h_weight = match level {
                    1 => 1.8,
                    2 => 1.5,
                    3 => 1.3,
                    _ => 1.1,
                };

                units.push(MarkdownUnit {
                    display_text: heading_md,
                    score_text: text,
                    is_atomic: false,
                    heading_weight: h_weight,
                    linked_atomic_idx: None,
                });
            }
            MarkdownBlock::Prose(text, parent_h_level) => {
                prose_for_keywords.push_str(&text);
                prose_for_keywords.push('\n');

                let h_weight = match parent_h_level {
                    1 => 1.4,
                    2 => 1.25,
                    3 => 1.1,
                    _ => 1.0,
                };

                let sentences = split_sentences(&text);
                for s in sentences {
                    let cleaned_sentence = sanitize_markdown_text(s);
                    if is_valid_sentence(&cleaned_sentence) {
                        units.push(MarkdownUnit {
                            display_text: cleaned_sentence.clone(),
                            score_text: cleaned_sentence,
                            is_atomic: false,
                            heading_weight: h_weight,
                            linked_atomic_idx: None,
                        });
                    }
                }
            }
        }
    }

    // link introductory sentences ending with a colon to subsequent atomic blocks
    for i in 0..units.len() {
        if !units[i].is_atomic && units[i].display_text.trim_end().ends_with(':') {
            if let Some(next_idx) = (i + 1..units.len()).find(|&j| units[j].is_atomic) {
                if (i + 1..next_idx).all(|k| !units[k].is_atomic) {
                    units[i].linked_atomic_idx = Some(next_idx);
                }
            }
        }
    }

    let keywords = parse_keywords(&prose_for_keywords, 2)?;
    let stop_set = get_stop_set();

    let orig_stats = TextStats {
        chars: input_text.chars().count(),
        words: input_text.split_whitespace().count(),
        sentences: units.len(),
    };

    if units.is_empty() {
        return Ok(SummaryReport {
            summary: String::new(),
            orig_stats,
            summary_stats: TextStats {
                chars: 0,
                words: 0,
                sentences: 0,
            },
            metrics: CompressionMetrics {
                target_cut,
                retained_ratio: 0.0,
                cut_percentage: 1.0,
            },
            keywords,
            top_keywords: Vec::new(),
            keyword_density: 0.0,
        });
    }

    let mut heap = BinaryHeap::new();
    let mut total_text_words = 0usize;

    for (idx, unit) in units.iter().enumerate() {
        if unit.is_atomic {
            heap.push(ScoredSentence {
                score: 0.1 * unit.heading_weight,
                order: idx,
                text: &unit.display_text,
            });
            continue;
        }

        let mut raw_score = 0.0;
        let mut word_count = 0usize;

        for (pos, word) in unit.score_text.split_whitespace().enumerate() {
            let clean = word.trim_matches(PUNCTUATIONS).to_lowercase();
            if clean.len() < 3 || stop_set.contains(clean.as_str()) {
                continue;
            }

            if let Some(&freq) = keywords.get(&clean) {
                let pos_weight = match pos {
                    0..=2 => 1.5,
                    3..=6 => 1.2,
                    _ => 1.0,
                };
                raw_score += (freq as f32) * pos_weight;
            }
            word_count += 1;
        }

        total_text_words += word_count;

        if word_count > 0 {
            let density_score = (raw_score / (word_count as f32).sqrt()) * unit.heading_weight;
            heap.push(ScoredSentence {
                score: density_score,
                order: idx,
                text: &unit.display_text,
            });
        } else {
            heap.push(ScoredSentence {
                score: 0.0,
                order: idx,
                text: &unit.display_text,
            });
        }
    }

    let total_keyword_occurrences: usize = keywords.values().sum();
    let keyword_density = if total_text_words > 0 {
        total_keyword_occurrences as f32 / total_text_words as f32
    } else {
        0.0
    };

    let max_score = heap.peek().map(|s| s.score).unwrap_or(0.0);
    let mut selected_indices: HashSet<usize> = HashSet::new();

    if keywords.is_empty() || max_score == 0.0 {
        let target_count = ((orig_stats.sentences as f32 * target_ratio).ceil() as usize).max(1);
        let step = orig_stats.sentences as f32 / target_count as f32;

        for i in 0..target_count {
            let idx = ((i as f32 * step) as usize).min(orig_stats.sentences - 1);
            selected_indices.insert(idx);
        }
    } else {
        let balanced_density = 0.15;
        let density_factor = (keyword_density / balanced_density).clamp(0.6, 1.4);
        let adjusted_ratio = (target_ratio * density_factor).clamp(0.05, 0.95);
        let target_count = ((orig_stats.sentences as f32 * adjusted_ratio).ceil() as usize).max(1);

        let min_score_threshold = max_score * 0.15;

        while selected_indices.len() < target_count {
            if let Some(item) = heap.pop() {
                if item.score < min_score_threshold && !selected_indices.is_empty() {
                    break;
                }
                selected_indices.insert(item.order);
            } else {
                break;
            }
        }
    };

    // context and dependency resolution phase
    let initial_selected: Vec<usize> = selected_indices.iter().copied().collect();
    for &idx in &initial_selected {
        // 1. attach linked atomic block (code/table) to the selected introductory sentence
        if let Some(linked_code_idx) = units[idx].linked_atomic_idx {
            selected_indices.insert(linked_code_idx);
        }

        // 2. attach introductory sentence to high-ranked atomic code blocks (ignoring headings)
        if units[idx].is_atomic && idx > 0 {
            let prev = &units[idx - 1];
            if !prev.is_atomic && !prev.display_text.starts_with('#') {
                selected_indices.insert(idx - 1);
            }
        }
    }

    let mut final_orders: Vec<usize> = selected_indices.into_iter().collect();
    final_orders.sort_unstable();

    let summary = final_orders
        .iter()
        .map(|&idx| units[idx].display_text.clone())
        .collect::<Vec<_>>()
        .join("\n\n");

    let summary_stats = TextStats {
        chars: summary.chars().count(),
        words: summary.split_whitespace().count(),
        sentences: final_orders.len(),
    };

    let mut top_keywords: Vec<(String, usize)> =
        keywords.iter().map(|(k, v)| (k.clone(), *v)).collect();
    top_keywords.sort_by(|a, b| b.1.cmp(&a.1));
    top_keywords.truncate(10);

    let retained_ratio = summary_stats.sentences as f32 / orig_stats.sentences as f32;
    // actual cut ratio calculated as a fraction (0.0 to 1.0) based on character reduction
    let actual_cut_ratio = 1.0 - (summary_stats.chars as f32 / orig_stats.chars.max(1) as f32);

    Ok(SummaryReport {
        summary,
        orig_stats,
        summary_stats,
        metrics: CompressionMetrics {
            target_cut,
            retained_ratio,
            cut_percentage: actual_cut_ratio,
        },
        keywords,
        top_keywords,
        keyword_density,
    })
}
