[![github]](https://github.com/fuderis/rs-simple-summarize)&ensp;
[![crates-io]](https://crates.io/crates/simple-summarize)&ensp;
[![docs-rs]](https://docs.rs/simple-summarize)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# Simple Summarize

Fast extractive text summarizer in Rust (with 30-70% compression).<br>
Uses frequency analysis + fuzzy Levenshtein + position bias. Works offline, supports EN+RU languages, no ML dependencies.


## Examples:

```rust
use simple_summarize::prelude::*;

fn main() -> Result<()> {
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

    let compress_coof = 3.0;
    let (summary, keywords) = summarize_text(input_text, compress_coof)?;
    
    println!("=== Compression level: {compress_coof:.1} ===");
    println!("=== Summary text: ===");
    println!("{}", summary);
    
    println!("\n=== Keywords: ===");
    for (word, count) in keywords {
        println!("{}: {}", word, count);
    }

    println!("\n=== Compressed: ===");
    let input_count = input_text.chars().count();
    let output_count = summary.chars().count();
    println!("Chars: -{chars} \nCoof: -{coof:.2}%",
        chars = (input_count as i32 - output_count as i32).abs(),
        coof = 100.0 - (output_count as f64 * 100.0 / input_count as f64),
    );
    
    Ok(())
}
```

## Licensing:

Distributed under the MIT license.


## Feedback:

You can [find me here](https://t.me/fuderis), also [see my channel](https://t.me/fuderis_club).
I welcome your suggestions and feedback!

> Copyright (c) 2025 *Bulat Sh.* ([fuderis](https://t.me/fuderis))
