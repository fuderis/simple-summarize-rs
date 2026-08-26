use simple_summarize::summarize_markdown;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_text = r#"
<p align="center">
  <img src="https://raw.githubusercontent.com/fuderis/osy-kernel/main/assets/logo.png" alt="Logo" width="80" />
</p>

<h1 align="center">Osy Kernel</h1>
<p align="center">
  <strong>Deterministic, Token-Efficient Engine for Next-Gen AI Assistants</strong><br>
  <code>lightweight</code> • <code>token-optimized</code> • <code>process-isolated</code> • <code>ultra-fast</code>
</p>

<img src="https://raw.githubusercontent.com/fuderis/osy-kernel/main/assets/cover.png" alt="Cover" width="100%" />

**Osy** is an open-source, high-performance orchestration kernel written in Rust. It is built for deploying ultra-lightweight, secure, and fully predictable personal and enterprise-grade AI assistants.

Modern agentic frameworks often suffer from uncontrolled agent autonomy, runaway token usage, and context leaks. Osy solves these issues at a deep system level: agents are isolated at the process level, system calls are purged from dialogue history, and memory operates in a hybrid mode.

> ⚠️ EXPERIMENTAL: **Osy** is undergoing rapid architectural evolution, experimental testing, and active refinement:
> * **Resource Usage & Storage Overhead:** Embedded storage drivers (LanceDB & Sled) currently run directly inside the kernel runtime and can consume significant RAM/I/O under heavy loads. API abstraction layers for external database backends (e.g., remote vector/KV servers) are actively planned for future optimization.
> * **Architectural Volatility:** Interfaces, memory formats, and IPC contracts are frequently refactored as we experiment with novel prompt-processing techniques and execution pipelines. API stability and production reliability are not guaranteed between commits.
> * **Solo Project:** This engine is currently developed and maintained by a single engineer. While code quality is strictly prioritized, managing every edge case takes time. Source audits before deployment are strongly recommended.
 
> 💡 **Contributions Welcome:** If you are passionate about low-level Rust systems, deterministic AI orchestration, or IPC engine design, feel free to open issues, submit pull requests, or reach out!

---

## Key Features

* **Extreme Context Optimization (Token Scrubbing):** Intermediate tool calls and service context are isolated and automatically purged from the active session history. You pay only for the final useful answers.
* **Full Determinism (Star Topology):** Agents act as strict executors with no permission to communicate unauthorized with one another or enter infinite recursive loops. All planning and context control are strictly managed by the Kernel.
* **Smart Hybrid Memory (RAG + Context Injection):** Automatic pre-fetching of relevant facts/embeddings before sending requests to the LLM, plus the model's ability to explicitly query the vector store on demand.
* **Native UDS & SSE Transport:** Agent interaction occurs strictly via Unix Domain Sockets (IPC) without network stack overhead, featuring full support for Server-Sent Events (SSE) streaming.
* **Secure Sandbox (Embedded JS):** Mathematical calculations, scripting, and data filtering are executed in an isolated Boa JS interpreter directly inside the process.
* **Self-Healing Execution:** Automatic restarts for failed agents and localized prompt adjustments upon receiving invalid arguments from the model.

---

## Architecture & Ecosystem

Osy utilizes a centralized orchestration model:

<img src="https://raw.githubusercontent.com/fuderis/osy-kernel/main/assets/scheme.png" alt="Scheme" width="100%" />

### Ecosystem of Specialized Rust Crates:

* **AnyLM:** Unified SDK layer for seamless operation across any model provider (OpenAI, Anthropic, Ollama, Local vLLM).
* **Cistern:** High-level async abstraction built on top of Sled (fast KV store) and LanceDB (embedded vector DB).
* **Pearce:** Axum-based networking engine with native UDS client and SSE streaming support.
* **Atoman:** Thread-safe management of asynchronous state and kernel configurations.
* **Boa JS:** Embedded lightweight JavaScript interpreter for deterministic computations without invoking external processes.

---

## Hybrid RAG & Smart Memory

Memory in Osy is split across several managed layers:

| Mechanism | Description |
|---|---|
| **Auto-Trigger Memory** | The kernel scans incoming context and automatically pulls relevant embeddings from LanceDB before sending the request to the LLM. |
| **Explicit Model Pull** | The model can initiate memory calls (`search_fact`, `remember_fact`, `forget_fact`) on its own if it lacks sufficient data for an accurate response. |
| **Dynamic System Prompts** | User preferences and global instructions are injected into the session in isolation without bloating the dialogue history. |

---

## Comparison: Standard Frameworks vs. Osy

| Parameter | Traditional Agent Frameworks | Osy Core Engine |
|---|---|---|
| **Agent Communication** | Mesh / P2P (agents spam each other) | Isolated Star (exclusively through Kernel) |
| **Token Consumption** | Grows linearly with every Tool Call | Fixed (service context is scrubbed) |
| **Memory** | Simple Vector Search / RAG | Hybrid RAG (Auto + Explicit + Dynamic Prompts) |
| **Network Stack** | Heavy HTTP/REST wrappers | Native Unix Domain Sockets + SSE Stream |
| **Predictability** | Probabilistic (high risk of hallucinations) | Deterministic (strict kernel scenarios) |
| **Processes** | Spawning per request | Long-lived persistent IPC workers |

---

## Roadmap

* [x] Long-Term Memory (RAG + Fact Storage)
* [x] Task-Scoped Context & Token Scrubbing
* [x] Native Process Lifecycle & UDS IPC
* [x] Embedded JS Engine (Boa Runtime for Isolated Computations)
* [ ] Native Web Search (Obscure integration)

---

## Quickstart

### Requirements
* **OS:** Unix-like (`Linux`, `macOS`, `BSD`)
* **Rust:** `nightly` toolchain
* **Dependencies:** `jq`

### Building from Source

```bash
# Clone repository
git clone https://github.com/fuderis/osy-kernel.git && cd osy-kernel

# Build project
bash build.sh

# Run CLI
osy --help
```

---

## Licensing & Commercial Usage

This project is distributed under the [**GNU General Public License v3.0 (GPL-3.0)**](LICENSE.md).

### Dual Licensing

* **Open Source Use:** You are free to use, modify, and deploy Osy in non-commercial or open-source projects in accordance with GPL-3.0.
* **Commercial License:** To integrate the Osy kernel into proprietary commercial products without disclosing your source code, you must acquire a commercial license.

> For commercial licensing inquiries and enterprise support, please contact the project author: **Bulat Sharipov** ([@fuderis](https://github.com/fuderis) / `synapdrake@ya.ru`).
"#;

    // Передаем желаемый процент урезания (например, 0.7 = вырезать 70% объема)
    let cut_percentage = 0.7;
    let report = summarize_markdown(input_text, cut_percentage)?;

    println!("=== Summary Text ===");
    println!("{}\n", report.summary);

    println!("=== Compression & Volume Stats ===");
    println!(
        "Requested cut:      {:.0}%",
        report.metrics.target_cut * 100.0
    );
    println!(
        "Actual cut (chars): {:.2}%",
        report.metrics.cut_percentage * 100.0
    );
    println!(
        "Retained ratio:     {:.1}%",
        report.metrics.retained_ratio * 100.0
    );

    println!(
        "Sentences:          {} -> {}",
        report.orig.sentences, report.summary_stats.sentences
    );
    println!(
        "Words:              {} -> {}",
        report.orig.words, report.summary_stats.words
    );
    println!(
        "Chars:              {} -> {}",
        report.orig.chars, report.summary_stats.chars
    );

    println!("\n=== SEO Metrics & Keywords ===");
    println!("Keyword Density:    {:.2}%", report.keyword_density * 100.0);

    println!("\nTop Keywords:");
    for (word, count) in &report.top_keywords {
        println!("  • {:<12} : {} times", word, count);
    }

    Ok(())
}
