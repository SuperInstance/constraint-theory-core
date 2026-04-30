# Contributing to Constraint Theory

Thank you for your interest in contributing!

## Building and Testing

```bash
git clone https://github.com/SuperInstance/constraint-theory-core.git
cd constraint-theory-core
cargo build --release
cargo test --release
```

## Code Style

All code must meet these requirements:

- **Clippy clean**: `cargo clippy -- -D warnings` must pass
- **Formatted**: Run `cargo fmt` before committing
- **Documented**: Every public item needs a doc comment (`#![deny(missing_docs)]` is enforced)

## Pull Request Process

1. Fork the repository and create a feature branch from `main`
2. Make your changes with clear commit messages
3. Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt`
4. Open a Pull Request with a description of what changed and why

## Contributions Welcome

- Higher-dimensional geometry (3D Pythagorean quadruples, nD)
- GPU implementations (CUDA, WebGPU)
- Performance benchmarks and optimizations
- Real-world use case examples

## Code of Conduct

This project follows the [Rust Community Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).
