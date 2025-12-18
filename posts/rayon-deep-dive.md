---
id: "2"
title: "Deep Dive into Rayon"
slug: "rayon-deep-dive"
date: "2023-11-15"
tags: ["rust", "concurrency", "performance"]
summary: "Exploring how Sinter uses Rayon for parallel data processing."
---

# Parallel Processing with Rayon

Sinter leverages [Rayon](https://github.com/rayon-rs/rayon) to process Markdown files in parallel.

## The Loop

Instead of a standard `for` loop, we use:

```rust
entries.par_iter().map(|entry| {
    // heavy lifting here
}).collect()
```

This ensures we utilize all CPU cores efficienty.

### Benchmarks

| File Count | Serial Time | Parallel Time |
|------------|-------------|---------------|
| 100        | 500ms       | 120ms         |
| 1000       | 4.5s        | 0.8s          |

> Note: Actual results may vary based on hardware.
