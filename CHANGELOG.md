# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2025-01-27

### Added
- **MASTER_SCHEMA.md**: Master documentation linking all ecosystem components
- **INTEGRATION.md**: Python FFI, WASM compilation, and cross-platform integration guide
- **RESEARCH_FOUNDATIONS.md**: Citations, mathematical proofs, and arXiv references
- **DEPLOYMENT.md**: CI/CD pipeline, release process, security audit checklist
- **ECOSYSTEM.md**: Complete ecosystem overview with cross-pollination use cases
- **Hidden dimensions formula**: `k = ⌈log₂(1/ε)⌉` documented throughout
- **Ecosystem integration section** in README.md with cross-repository links
- **Integration testing documentation** for Python binding compatibility
- **SIMD consistency testing** documentation
- Enhanced **Cargo.toml** metadata with additional keywords and categories
- **Enhanced CI/CD pipeline**: Cross-platform testing, coverage, benchmarks, security audit

### Changed
- **ONBOARDING.md**: Complete API consistency rewrite
  - Updated all code examples to match actual API (`constraint_theory_core` not `constraint_theory`)
  - Fixed function signatures (`snap(&manifold, [x, y])` not `manifold.snap(&point)`)
  - Added Grand Unified Constraint Theory (GUCT) branding
  - Corrected benchmark numbers (~100ns not 45ns for single snap)
  - Added proper cross-references to ecosystem
- **README.md**: Updated citation version to match Cargo.toml (1.0.1)
- **Ecosystem table**: Enhanced with Key Features column
- **Cargo.toml**: Added keywords and categories for better discoverability

### Fixed
- API documentation inconsistencies between ONBOARDING.md and actual code
- Version number mismatch in citation
- Incorrect benchmark claims in ONBOARDING.md
- Missing links to Python examples and web visualizations

### Documentation
- All code examples now use correct `constraint_theory_core` crate name
- All code examples compile against actual API
- Hidden dimensions formula prominently featured
- Cross-platform determinism notes added

## [1.0.0] - 2025-01-15

### Added
- **PythagoreanManifold**: Core manifold implementation with KD-tree indexing
- **snap() function**: O(log n) nearest neighbor lookup
- **SIMD batch processing**: AVX2 parallel batch snapping
- **PythagoreanTriple**: Struct for exact triple representation
- **KD-tree implementation**: Cache-optimized spatial indexing
- **Edge case handling**: NaN, zero, infinity inputs
- **Cross-platform determinism**: Scalar fallback for consensus-critical code
- **82 passing tests**: Comprehensive test coverage

### Performance
- Single snap: ~100 ns average
- Batch SIMD: ~74 ns/op average
- Manifold build (density=200): ~2.8 ms
- Memory usage: ~80 KB for density=200

### Dependencies
- Zero runtime dependencies (pure Rust)
- `rand` as dev-dependency for testing

---

## Version Compatibility

| Core Version | Rust Version | Python Version | Notes |
|--------------|--------------|----------------|-------|
| 1.0.1 | 1.75+ | 1.0.0+ | Current stable |
| 1.0.0 | 1.75+ | 1.0.0+ | Initial release |

---

## Roadmap

### Planned for 1.1.0
- AVX-512 support for wider SIMD parallelism
- ARM NEON optimization for Apple Silicon
- Persistent KD-tree serialization
- Approximate mode for sub-50ns operations

### Planned for 2.0.0
- 3D Pythagorean quadruple support
- GPU kernels (CUDA/WebGPU)
- Higher-dimensional extensions (E8 lattice)

---

[1.0.1]: https://github.com/SuperInstance/constraint-theory-core/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/SuperInstance/constraint-theory-core/releases/tag/v1.0.0
