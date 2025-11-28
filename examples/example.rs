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
