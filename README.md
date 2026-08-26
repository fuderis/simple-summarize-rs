[![github]](https://github.com/fuderis/rs-simple-summarize)&ensp;
[![crates-io]](https://crates.io/crates/simple-summarize)&ensp;
[![docs-rs]](https://docs.rs/simple-summarize)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# Simple Summarizer

Fast and lightweight extractive text summarizer written in Rust.

It uses frequency analysis, keyword density scoring, and position weighting to extract key sentences while preserving structure.
Works completely offline without heavy ML dependencies, and supports processing both plain text and full Markdown documents via AST parsing.

## Features

* **Plain Text & Markdown Support** — dedicated AST parsing (`summarize_markdown`) for Markdown documents that preserves code blocks, tables, and heading context.
* **Smart Sentence Ranking** — scores content based on keyword frequency, position bias, and length normalization.
* **Zero Heavy ML** — pure Rust, lightweight, and fast execution for edge or serverless environments.
* **Rich Analytics** — returns detailed metrics, character/word statistics, keyword densities, and top keywords via `SummaryReport`.

## Feature Flags

Markdown support is disabled by default to keep the core crate as lean as possible. Enable it in your `Cargo.toml`:

```toml
[dependencies]
simple-summarize = { version = "0.2", features = ["markdown"] }
```

---

## Quickstart

Pass the raw string along with a target reduction percentage (e.g., `20.0` for 20% cut or `0.2` as a fraction).

```rust
use simple_summarize::{SummaryReport, summarize_text};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_text = r#"
    Rust is a programming language focused on security, speed, and concurrency. 
It helps you write fast and reliable software. Rust is especially good for system programming,
where performance and no memory errors are important. 

Most programming languages struggle with the issue of memory security. 
Rust solves this problem by checking at the compilation stage. 
The Rust compiler guarantees memory safety without a garbage collector.

The syntax of Rust is similar to C++, but with modern features. 
The language supports a powerful type system and memory borrowing. 
Rust is popular for web builds, network applications, and embedded systems.
    "#;

    // Cut target: 70% reduction
    let report: SummaryReport = summarize_text(input_text, 0.7)?;

    println!("=== Summary ===");
    println!("{}", report.summary);

    println!("\n=== Metrics ===");
    println!("Target cut: {:.1}%", report.metrics.target_cut * 100.0);
    println!("Actual cut: {:.2}%", report.metrics.cut_percentage * 100.0);
    println!("Original chars: {}", report.orig_stats.chars);
    println!("Summary chars:  {}", report.summary_stats.chars);

    println!("\n=== Top Keywords ===");
    for (word, count) in &report.top_keywords {
        println!("{word}: {count}");
    }

    Ok(())
}
```

## License & Feedback

> Distributed under the [MIT](https://github.com/fuderis/pearce-rs/blob/main/LICENSE.md) license.

You can contact me via [GitHub](https://github.com/fuderis) or send a message to my [E-Mail](mailto:synapdrake@ya.ru).
This library is actively evolving, and your suggestions and feedback are always welcome!
