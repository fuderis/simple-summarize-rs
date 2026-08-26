use super::*;
use std::collections::BinaryHeap;

/// Performs adaptive extractive text summarization on raw text inputs.
///
/// Strips HTML tags, parses keywords, and ranks sentences based on keyword frequency,
/// position, and density to generate an optimized summary report.
///
/// # Arguments
///
/// * `input_text` - The raw string slice to be summarized.
/// * `cut_ratio` - The target reduction ratio as a fraction from 0.0 to 1.0 (e.g., 0.20 for 20%).
///
/// # Returns
///
/// Returns a [`Result`] containing a [`SummaryReport`] populated with statistics, metrics,
/// and extracted text summary.
pub fn summarize_text(input_text: &str, cut_ratio: f32) -> Result<SummaryReport> {
    // sanitize input text by stripping html tags
    let clean_input = strip_html_tags(input_text);

    // normalize cut ratio, bounding it strictly between 0.0 and 0.95
    let target_cut = cut_ratio.clamp(0.0, 0.95);
    let target_ratio = 1.0 - target_cut;

    // extract text metadata, keywords, stop words, and sentence boundaries
    let keywords = parse_keywords(&clean_input, 2)?;
    let stop_set = get_stop_set();
    let sentences = split_sentences(&clean_input);

    let orig = TextStats {
        chars: clean_input.chars().count(),
        words: clean_input.split_whitespace().count(),
        sentences: sentences.len(),
    };

    // return an empty report early if input contains no valid sentences
    if sentences.is_empty() {
        return Ok(SummaryReport {
            summary: String::new(),
            orig,
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

    // score each sentence based on keyword frequency and position
    for (idx, sentence) in sentences.iter().enumerate() {
        let mut raw_score = 0.0;
        let mut word_count = 0usize;

        for (pos, word) in sentence.split_whitespace().enumerate() {
            let clean = word.trim_matches(PUNCTUATIONS).to_lowercase();
            if clean.len() < 3 || stop_set.contains(clean.as_str()) {
                continue;
            }

            if let Some(&freq) = keywords.get(&clean) {
                // assign higher weight to words appearing near sentence start
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

        // normalize sentence score by square root of word count to prevent long-sentence bias
        if word_count > 0 {
            let density_score = raw_score / (word_count as f32).sqrt();
            heap.push(ScoredSentence {
                score: density_score,
                order: idx,
                text: sentence,
            });
        }
    }

    // calculate overall keyword density relative to total words
    let total_keyword_occurrences: usize = keywords.values().sum();
    let keyword_density = if total_text_words > 0 {
        total_keyword_occurrences as f32 / total_text_words as f32
    } else {
        0.0
    };

    let max_score = heap.peek().map(|s| s.score).unwrap_or(0.0);

    // select top-scoring sentences or fallback to uniform sampling if scoring fails
    let selected = if keywords.is_empty() || max_score == 0.0 {
        // uniform fallback sampling when keywords are missing or zero-scored
        let target_count = ((orig.sentences as f32 * target_ratio).ceil() as usize).max(1);
        let step = orig.sentences as f32 / target_count as f32;

        (0..target_count)
            .map(|i| {
                let idx = ((i as f32 * step) as usize).min(orig.sentences - 1);
                ScoredSentence {
                    score: 0.0,
                    order: idx,
                    text: sentences[idx],
                }
            })
            .collect::<Vec<_>>()
    } else {
        // adapt target sentence count dynamically based on keyword density
        let balanced_density = 0.15;
        let density_factor = (keyword_density / balanced_density).clamp(0.6, 1.4);
        let adjusted_ratio = (target_ratio * density_factor).clamp(0.05, 0.95);
        let target_count = ((orig.sentences as f32 * adjusted_ratio).ceil() as usize).max(1);

        let min_score_threshold = max_score * 0.15;
        let mut picked = Vec::with_capacity(target_count);

        // pop highest-scoring sentences from heap until target count or threshold is reached
        while picked.len() < target_count {
            if let Some(item) = heap.pop() {
                if item.score < min_score_threshold && !picked.is_empty() {
                    break;
                }
                picked.push(item);
            } else {
                break;
            }
        }
        // sort selected sentences to maintain original narrative flow
        picked.sort_by_key(|s| s.order);
        picked
    };

    // combine selected sentences into final formatted summary text
    let summary = selected
        .into_iter()
        .map(|s| s.text.replace('\n', " "))
        .collect::<Vec<_>>()
        .join("\n");

    // compute output statistics and top keywords
    let summary_stats = TextStats {
        chars: summary.chars().count(),
        words: summary.split_whitespace().count(),
        sentences: split_sentences(&summary).len(),
    };

    let mut top_keywords: Vec<(String, usize)> =
        keywords.iter().map(|(k, v)| (k.clone(), *v)).collect();
    top_keywords.sort_by(|a, b| b.1.cmp(&a.1));
    top_keywords.truncate(10);

    let retained_ratio = summary_stats.sentences as f32 / orig.sentences as f32;
    // actual cut ratio calculated as a fraction (0.0 to 1.0) based on character reduction
    let actual_cut_ratio = 1.0 - (summary_stats.chars as f32 / orig.chars.max(1) as f32);

    Ok(SummaryReport {
        summary,
        orig,
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
