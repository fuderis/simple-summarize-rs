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
