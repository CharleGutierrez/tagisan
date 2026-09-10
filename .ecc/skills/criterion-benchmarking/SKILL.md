---
name: criterion-benchmarking
description: Statistical micro-benchmarking with Criterion, throughput measurement, and allocation profiling
---

# Criterion Statistical Micro-Benchmarking

## Benchmarking Protocol
1. **Statistical Isolation**: Use `criterion::black_box` to prevent LLVM dead-code elimination and constant folding.
2. **Throughput Metrics**: Configure benchmarks with `Throughput::Bytes` or `Throughput::Elements` for realistic MB/s evaluations.
3. **Warmup & Outlier Detection**: Enforce minimum 3-second warmup and 5-second measurement periods across at least 100 samples.
4. **Flamegraph Integration**: Pair benchmarks with `pprof` or `cargo-flamegraph` to visualize CPU bottlenecks in hot loops.
