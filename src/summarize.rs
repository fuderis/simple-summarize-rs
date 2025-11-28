use crate::prelude::*;

const PUNCTUATIONS: &[char] = &[
    '.', ',', '!', '?', ';', ':', '…', '„', '«', '»', '`', '\'', '"', '(', ')', 
    '[', ']', '{', '}', '<', '>', '-', '–', '—', '/', '\\', '|', '@', '#', '%', 
    '&', '*', '+', '=', '^', '~', '$', '€', '£', '¢', '§', '°',
];

const STOP_WORDS: &[&str] = &[
    // english:
    "etc.", "i.e", "e.g", "dr.", "mr.", "mrs.", "u.s.", "rep.", "sen.", "st.",
    "jan.", "feb.", "mar.", "apr.", "may.", "jun.", "jul.", "aug.", "sep.",
    "oct.", "nov.", "dec.", "lt.", "gov.", "a.m.", "p.m.", 

    "a", "an", "the", "and", "or", "but", "not", "be", "have", "do", 
    "can", "will", "shall", "may", "must", "should", "could", "would",
    "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", 
    "us", "them", "my", "your", "his", "its", "our", "their",
    "this", "that", "these", "those", "who", "what", "which", "where",
    "when", "why", "how", "if", "in", "on", "at", "to", "for", "of", 
    "with", "by", "from", "up", "about", "into", "through", "during",
    "before", "after", "above", "below", "between", "under", "over",
    "all", "any", "each", "some", "no", "nor", "only", "other", "such",
    "same", "so", "than", "too", "very", "just", "now", "here", "there",

    // russian:
    "т.д.", "т.п.", "т.е.", "напр.", "см.", "стр.", "гл.", 
    "р.", "г.", "ул.", "д.", "кв.", "тел.", "моб.",

    "а", "и", "в", "на", "с", "у", "к", "по", "для", "из", "не", "но", 
    "что", "это", "как", "кто", "где", "когда", "зачем", "почему",
    "быть", "иметь", "делать", "мочь", "хотеть", "нужно", "можно",
    "я", "ты", "он", "она", "оно", "мы", "вы", "они", "мой", "твой",
    "его", "ее", "их", "этот", "тот", "все", "каждый", "другой",
    "такой", "столько", "если", "или", "даже", "только", "всего",
    "уже", "еще", "очень", "совсем", "сам", "самый", "например",
];

const DIST_COOF: f64 = 0.8;


/// Summarizes input text and returns it with keywords (Summarized, Keywords)
pub fn summarize_text(input_text: &str, compress_coof: f64) -> Result<(String, HashMap<String, usize>)> {
    let keywords = parse_keywords(input_text, 2)?;
    
    // sorting text sentences by keywords & building output:
    let mut output_sentences = vec![];
    for sentence in input_text.split(['.', '!', '?', ';']) {
        let sentence = sentence.trim();
        let clear_sentence = sentence.to_lowercase().replace(PUNCTUATIONS, "");
        if clear_sentence.is_empty() { continue }

        let mut score = 0.0;
        for (i, word) in clear_sentence.split_whitespace().enumerate() {
            if keywords.get(word).is_some() {
                let weight = match i {
                    0..=2 => 1.5,    // first 3 words: +50%
                    3..=6 => 1.2,    // from 4 to 6: +20%  
                    _ => 1.0,        // other: +0%
                };
                score += weight;
            }
        }

        if score >= compress_coof {
            output_sentences.push(fmt!("{}.", sentence.replace("\n", " ").replace("  ", " ")));
        }
    }
    
    Ok((output_sentences.join("\n"), keywords))
}

/// Parses text into keywords
pub fn parse_keywords(input_text: &str, min_count: usize) -> Result<HashMap<String, usize>> {
    let clear_text = input_text.trim().to_lowercase().replace(PUNCTUATIONS, "");
    let mut keywords: HashMap<String, usize> = map![];

    // removing stop words & analysing keywords:
    'f1: for word in clear_text.split_whitespace() {
        // skip stop words:
        for stop_word in STOP_WORDS.iter() {
            if lev_compare(word, stop_word) {
                continue 'f1;
            }
        }

        // plus same keyword:
        for (keyword, count) in keywords.iter_mut() {
            if lev_compare(keyword, word) {
                *count += 1;
                continue 'f1;
            }
        }
        // or insert new keyword:
        keywords.insert(word.to_owned(), 1);
    }
    
    Ok(
        keywords
            .into_iter()
            .filter(|(_, count)| *count >= min_count)
            .collect()
    )
}

/// Compares two texts by levenshtein distance alghoritm
fn lev_compare(s1: &str, s2: &str) -> bool {
    let dist = distance::levenshtein(s1, s2);
    let max_len = s1.chars().count().max(s2.chars().count()).max(1);
    let coof = 1.0 - (dist as f64 / max_len as f64);
    
    coof >= DIST_COOF
}
