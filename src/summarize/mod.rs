pub mod text;
pub use text::summarize_text;

#[cfg(feature = "markdown")]
pub mod markdown;
#[cfg(feature = "markdown")]
pub use markdown::summarize_markdown;

use crate::prelude::*;
use ahash::{AHashMap, AHashSet};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, sync::OnceLock};

/// Represents statistical metrics of a text input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStats {
    /// Total character count.
    pub chars: usize,
    /// Total word count.
    pub words: usize,
    /// Total sentence count.
    pub sentences: usize,
}

/// Represents metrics showing how much the text was compressed during summarization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetrics {
    /// Target reduction ratio requested by the caller.
    pub target_cut: f32,
    /// Proportion of text length retained after processing.
    pub retained_ratio: f32,
    /// Percentage of original text length removed.
    pub cut_percentage: f32,
}

/// Contains the final result of text summarization, including stats and keywords.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryReport {
    /// Generated text summary.
    pub summary: String,
    /// Statistical metrics of the original text.
    pub orig_stats: TextStats,
    /// Statistical metrics of the generated summary.
    pub summary_stats: TextStats,
    /// Compression metrics comparing original text and summary.
    pub metrics: CompressionMetrics,
    /// Frequency map of all processed non-stop words.
    pub keywords: AHashMap<String, usize>,
    /// Sorted list of the most frequent keywords with their counts.
    pub top_keywords: Vec<(String, usize)>,
    /// Ratio of key words to total word count in original text.
    pub keyword_density: f32,
}

/// Helper struct for scoring and ordering sentences during summarization.
#[derive(PartialEq)]
pub(crate) struct ScoredSentence<'a> {
    /// Calculated importance score for the sentence.
    pub score: f32,
    /// Original index position of the sentence in the text.
    pub order: usize,
    /// Slice reference to the sentence content.
    pub text: &'a str,
}

impl<'a> Eq for ScoredSentence<'a> {}

impl<'a> Ord for ScoredSentence<'a> {
    /// Compares two scored sentences primarily by their score value.
    fn cmp(&self, other: &Self) -> Ordering {
        // partial ordering comparison falling back to equality on NaN
        self.score
            .partial_cmp(&other.score)
            .unwrap_or(Ordering::Equal)
    }
}

impl<'a> PartialOrd for ScoredSentence<'a> {
    /// Compares two scored sentences for partial ordering.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Array of punctuation characters stripped during word normalisation.
pub const PUNCTUATIONS: &[char] = &[
    '.', ',', '!', '?', ';', ':', '…', '„', '«', '»', '`', '\'', '"', '(', ')', '[', ']', '{', '}',
    '<', '>', '-', '–', '—', '/', '\\', '|', '@', '#', '%', '&', '*', '+', '=', '^', '~', '$', '€',
    '£', '¢', '§', '°',
];

/// List of common multilingual stop words omitted from key term analysis.
static STOP_WORDS: &[&str] = &[
    // English
    "etc.",
    "i.e",
    "e.g",
    "dr.",
    "mr.",
    "mrs.",
    "u.s.",
    "a",
    "an",
    "the",
    "i",
    "you",
    "he",
    "she",
    "it",
    "we",
    "they",
    "me",
    "him",
    "her",
    "us",
    "them",
    "my",
    "your",
    "his",
    "its",
    "our",
    "their",
    "this",
    "that",
    "these",
    "those",
    "who",
    "what",
    "which",
    "where",
    "when",
    "why",
    "how",
    "all",
    "any",
    "each",
    "some",
    "no",
    "nor",
    "only",
    "other",
    "such",
    "same",
    "so",
    "than",
    "too",
    "very",
    "just",
    "now",
    "here",
    "there",
    "own",
    "not",
    "and",
    "or",
    "but",
    "if",
    "in",
    "on",
    "at",
    "to",
    "for",
    "of",
    "with",
    "by",
    "from",
    "up",
    "about",
    "into",
    "through",
    "during",
    "before",
    "after",
    "above",
    "below",
    "between",
    "under",
    "over",
    "out",
    "off",
    "down",
    "be",
    "been",
    "being",
    "is",
    "am",
    "are",
    "was",
    "were",
    "have",
    "has",
    "had",
    "having",
    "do",
    "does",
    "did",
    "doing",
    "can",
    "could",
    "will",
    "would",
    "shall",
    "should",
    "may",
    "might",
    "must",
    "use",
    "used",
    "uses",
    "using",
    "make",
    "made",
    "makes",
    "making",
    "see",
    "seen",
    "saw",
    "look",
    "get",
    "got",
    "give",
    "take",
    "type",
    "types",
    "value",
    "values",
    "example",
    "examples",
    "code",
    "way",
    "one",
    "two",
    "also",
    "like",
    "well",
    "even",
    "first",
    "new",
    "many",
    "more",
    // Russian
    "т.д.",
    "т.п.",
    "т.е.",
    "напр.",
    "см.",
    "стр.",
    "а",
    "и",
    "в",
    "на",
    "с",
    "у",
    "к",
    "по",
    "для",
    "из",
    "не",
    "но",
    "что",
    "это",
    "как",
    "кто",
    "где",
    "когда",
    "зачем",
    "почему",
    "я",
    "ты",
    "он",
    "она",
    "оно",
    "мы",
    "вы",
    "они",
    "мой",
    "твой",
    "его",
    "ее",
    "их",
    "этот",
    "тот",
    "все",
    "каждый",
    "другой",
    "такой",
    "столько",
    "если",
    "или",
    "даже",
    "только",
    "всего",
    "уже",
    "еще",
    "очень",
    "совсем",
    "сам",
    "самый",
    "например",
    "быть",
    "был",
    "была",
    "было",
    "были",
    "есть",
    "будет",
    "будут",
    "иметь",
    "делать",
    "сделать",
    "мочь",
    "хотеть",
    "нужно",
    "можно",
    "просто",
    "также",
    "время",
    "место",
    "часть",
    "вид",
    "пример",
];

/// Singleton reference holder for the stop words lookup set.
static STOP_SET: OnceLock<AHashSet<&'static str>> = OnceLock::new();

/// Returns a reference to the global lazily initialized set of stop words.
pub(crate) fn get_stop_set() -> &'static AHashSet<&'static str> {
    STOP_SET.get_or_init(|| {
        // initialize hash set with exact capacity needed
        let mut set = AHashSet::with_capacity(STOP_WORDS.len());
        for &word in STOP_WORDS {
            set.insert(word);
        }
        set
    })
}

/// Strips all HTML elements from a string, returning plain text.
pub fn strip_html_tags(input: &str) -> String {
    // pre-allocate output string capacity matching input size
    let mut result = String::with_capacity(input.len());
    let mut inside_tag = false;

    for c in input.chars() {
        match c {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(c),
            _ => {}
        }
    }
    result
}

/// Determines whether a byte index represents a valid sentence boundary.
pub fn is_sentence_boundary(text: &str, byte_idx: usize) -> bool {
    let bytes = text.as_bytes();
    let ch = bytes[byte_idx];

    // standard unambiguous sentence terminators
    if ch == b'!' || ch == b'?' || ch == b';' {
        return true;
    }

    if ch != b'.' {
        return false;
    }

    // avoid splitting decimal numbers
    let prev_is_digit = byte_idx > 0 && bytes[byte_idx - 1].is_ascii_digit();
    let next_is_digit = byte_idx + 1 < bytes.len() && bytes[byte_idx + 1].is_ascii_digit();
    if prev_is_digit && next_is_digit {
        return false;
    }

    // check if preceding word is an abbreviation or stop word
    let prefix = &text[..byte_idx];
    if let Some(last_word) = prefix.split_whitespace().next_back() {
        let clean_word = last_word
            .trim_matches(|c: char| PUNCTUATIONS.contains(&c) || c == '*' || c == '_')
            .to_lowercase();

        if get_stop_set().contains(clean_word.as_str())
            || get_stop_set().contains(format!("{clean_word}.").as_str())
        {
            return false;
        }

        // skip boundaries after short abbreviations followed by lower case or no spaces
        if clean_word.chars().count() <= 2 {
            if let Some(next_char) = text[byte_idx + 1..].chars().next() {
                if next_char.is_lowercase() || !next_char.is_whitespace() {
                    return false;
                }
            }
        }
    }

    // boundary requires subsequent whitespace or line break
    if byte_idx + 1 < bytes.len() {
        let next_byte = bytes[byte_idx + 1];
        if !next_byte.is_ascii_whitespace() && next_byte != b'\n' && next_byte != b'\r' {
            return false;
        }
    }

    true
}

/// Splits a text block into individual sentences based on detected boundaries.
pub fn split_sentences(text: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (idx, ch) in text.char_indices() {
        if is_sentence_boundary(text, idx) {
            // slice text up to boundary index and trim outer whitespace
            let sentence = text[start..=idx].trim();
            if !sentence.is_empty() {
                sentences.push(sentence);
            }
            start = idx + ch.len_utf8();
        }
    }

    // collect trailing text remaining after the final boundary
    if start < text.len() {
        let tail = text[start..].trim();
        if !tail.is_empty() {
            sentences.push(tail);
        }
    }

    sentences
}

/// Extracts keywords from text and returns a map of their occurrences matching or exceeding `min_count`.
pub fn parse_keywords(input_text: &str, min_count: usize) -> Result<AHashMap<String, usize>> {
    let stop_set = get_stop_set();
    let mut keywords: AHashMap<String, usize> = AHashMap::default();

    for line in input_text.lines() {
        for raw_word in line.split_whitespace() {
            // normalize by trimming punctuation and converting to lowercase
            let clean_word = raw_word.trim_matches(PUNCTUATIONS).to_lowercase();
            if clean_word.len() < 3 || stop_set.contains(clean_word.as_str()) {
                continue;
            }
            *keywords.entry(clean_word).or_insert(0) += 1;
        }
    }

    // filter out key terms below the minimum required frequency threshold
    keywords.retain(|_, count| *count >= min_count);

    Ok(keywords)
}
