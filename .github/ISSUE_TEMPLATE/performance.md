---
name: Performance Report
about: Report benchmark results or performance improvements
title: '[PERF] '
labels: performance
assignees: ''
---

## Performance Report

**Type of Report:**
- [ ] Benchmark results
- [ ] Performance improvement
- [ ] Regression report
- [ ] Optimization proposal
- [ ] Other (explain below)

## Baseline

**What are you comparing against?**

- Implementation: [e.g., Python baseline, Rust scalar, existing CUDA kernel]
- Configuration: [e.g., 100K tiles, RTX 4090, AVX-512]
- Baseline performance: [e.g., 74 ns/op, 280x speedup]

## Results

**What are your results?**

| Metric | Baseline | New | Change |
|--------|----------|-----|--------|
| Latency | 74 ns | ? ns | ?% |
| Throughput | 13.5M ops/sec | ? ops/sec | ?% |
| Memory usage | ? MB | ? MB | ?% |
| Speedup vs NumPy | 280x | ?x | ?% |

## Methodology

**How did you measure this?**

- [ ] Standard benchmark suite ([BASELINE_BENCHMARKS.md](BASELINE_BENCHMARKS.md))
- [ ] Custom benchmark (describe below)
- [ ] Production workload (describe below)
- [ ] Simulation results

**Hardware:**
- CPU: [e.g., Intel i9-13900K, AMD Ryzen 9 7950X]
- GPU: [e.g., RTX 4090, A100, or "N/A"]
- RAM: [e.g., 64GB DDR5]
- OS: [e.g., Ubuntu 22.04, Windows 11]

## Analysis

**What do these results mean?**

- Is this expected or surprising?
- What factors contribute to the performance?
- Are there any caveats or limitations?

## Code Changes

**Did you make code changes to achieve this?**

- [ ] Yes, link to PR or branch:
- [ ] No, this is measurement only
- [ ] Yes, but not yet committed (describe below)

## Next Steps

**What should happen next?**

- [ ] Update documentation with new results
- [ ] Merge performance improvement
- [ ] Investigate performance regression
- [ ] Further optimization needed
- [ ] Other (specify)

## Additional Context

**Profiling data, flame graphs, etc.:**

Attach any additional information:
- Profiling output
- Flame graphs
- Assembly analysis
- Hardware counters

---

**Thank you for helping us go faster!** ⚡

Every nanosecond counts. Every speedup matters.
