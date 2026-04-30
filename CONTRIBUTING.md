# Contributing to Constraint Theory Core

Thank you for your interest in contributing to Constraint Theory Core!

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/constraint-theory-core.git`
3. Create a branch: `git checkout -b feature/my-feature`

## Development Setup

### Prerequisites

- Rust 1.75+ (for building the core library)
- Cargo (comes with Rust)

### Building and Testing

```bash
git clone https://github.com/SuperInstance/constraint-theory-core.git
cd constraint-theory-core
cargo build --release
cargo test --release
cargo bench  # Run benchmarks
```

## Making Changes

### Branch Naming

- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Adding or modifying tests

### Commit Messages

Follow conventional commits:

```
feat: add new batch processing method
fix: correct noise calculation for edge cases
docs: update installation instructions
test: add tests for edge cases
refactor: simplify manifold initialization
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_snap_accuracy

# Run benchmarks
cargo bench
```

### Writing Tests

- Place unit tests in the same file using `#[cfg(test)]` modules
- Place integration tests in the `tests/` directory
- Use descriptive test function names
- Document expected behavior in test comments

## Pull Request Process

1. **Update Documentation**: Ensure README.md and doc comments are updated
2. **Add Tests**: New features need tests
3. **Run Tests**: All tests must pass (`cargo test`)
4. **Check Style**: Run `cargo clippy -- -D warnings` and `cargo fmt`
5. **Submit PR**: Use the PR template

### PR Checklist

- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated (doc comments)
- [ ] Tests added and passing
- [ ] No new warnings introduced
- [ ] `cargo clippy` passes with no warnings

## Style Guidelines

### Rust Code

- Follow Rust standard formatting (`cargo fmt`)
- Document all public APIs with doc comments (`//!` and `///`)
- Run clippy: `cargo clippy -- -D warnings`
- **Clippy clean**: `cargo clippy -- -D warnings` must pass
- **Formatted**: Run `cargo fmt` before committing
- **Documented**: Every public item needs a doc comment (`#![deny(missing_docs)]` is enforced)

## Contributions Welcome

- Higher-dimensional geometry (3D Pythagorean quadruples, nD)
- GPU implementations (CUDA, WebGPU)
- Performance benchmarks and optimizations
- Real-world use case examples
- Language bindings (Go, TypeScript, Julia, etc.)

## Questions?

- Open a [Discussion](https://github.com/SuperInstance/constraint-theory-core/discussions)
- Check existing [Issues](https://github.com/SuperInstance/constraint-theory-core/issues)

Thank you for contributing!
