# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported | Notes |
| ------- | --------- | ----- |
| 1.0.x | :white_check_mark: | Current stable |
| 0.1.x | :x: | Pre-release, not supported |

## Security Model

### Threat Model

`constraint-theory-core` is designed for:

1. **Mathematical correctness** - Deterministic output for identical inputs
2. **Graceful degradation** - Invalid inputs return safe error indicators
3. **Memory safety** - No buffer overflows, use-after-free, or data races

**NOT designed for:**
- Cryptographic security
- Handling untrusted network input without validation
- Secret key storage or derivation

### Input Handling

The library handles all inputs safely:

```rust
// NaN inputs return error indicator (noise = 1.0)
let (snapped, noise) = manifold.snap([f32::NAN, 0.0]);
assert_eq!(noise, 1.0);

// Infinity inputs return error indicator
let (snapped, noise) = manifold.snap([f32::INFINITY, 0.0]);
assert_eq!(noise, 1.0);

// Zero vectors return safe default
let (snapped, noise) = manifold.snap([0.0, 0.0]);
assert!(snapped[0].is_finite());
```

### Memory Safety Guarantees

- **No unsafe in public API** - All public functions are safe Rust
- **Bounds-checked arrays** - No buffer overflow possible
- **No mutable global state** - Thread-safe by design
- **Deterministic allocation** - Memory usage is predictable

### SIMD Safety

SIMD code uses `unsafe` internally but is wrapped in safe APIs:

```rust
// SIMD is automatically selected at runtime
let results = manifold.snap_batch_simd(&vectors);

// SIMD path has platform-specific tie-breaking
// For consensus-critical code, use scalar:
manifold.snap_batch(&vectors, &mut results);
```

## Reporting a Vulnerability

We take the security of constraint-theory-core seriously. If you believe you have found a security vulnerability, please report it to us as described below.

### How to Report

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via GitHub Security Advisories:

1. Go to https://github.com/SuperInstance/constraint-theory-core/security/advisories/new
2. Fill out the form with details about the vulnerability

You can also email us at: security@superinstance.ai

### What to Include

Please include the following information in your report:

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** of the vulnerability
- **Possible mitigations** (if you have any)
- Your **contact information** for follow-up

### Response Timeline

- **Initial Response**: Within 48 hours
- **Triage & Assessment**: Within 5 business days
- **Fix Development**: Depends on severity and complexity
- **Disclosure**: After fix is released

### Disclosure Policy

- We follow **responsible disclosure** practices
- We will credit you in the security advisory (unless you prefer to remain anonymous)
- We request that you do not disclose the vulnerability publicly until a fix has been released

## Security Considerations by Use Case

### Real-time Applications

For games, animations, and real-time systems:

- SIMD path is appropriate (performance critical)
- Invalid inputs produce safe defaults
- Monitor for unexpected noise values

### Consensus Systems

For blockchain, distributed systems, and consensus-critical applications:

- **Always use scalar path** (`snap_batch` not `snap_batch_simd`)
- **Validate inputs** before processing
- **Reject inputs** that fail validation

```rust
// Consensus-safe pattern
match manifold.validate_input([x, y]) {
    Ok(()) => {
        let (snapped, noise) = manifold.snap([x, y]);
        // Process result
    }
    Err(reason) => {
        // Reject input
        return Err(ConsensusError::InvalidInput(reason));
    }
}
```

### Scientific Computing

For research and scientific applications:

- Document manifold density used
- Report noise values with results
- Consider precision requirements carefully

## Known Security Limitations

1. **Not Cryptographic**: Do not use for cryptographic hashing, key derivation, or encryption
2. **Deterministic Only**: SIMD path may vary across platforms; use scalar for determinism
3. **No Authentication**: The library does not authenticate inputs
4. **No Rate Limiting**: Callers must implement their own rate limiting

## Dependency Security

The library has **zero dependencies** in its public API, minimizing attack surface.

Development dependencies (for testing only):
- `rand` - Used only in test code

## Security Best Practices

When using constraint-theory-core:

1. **Keep dependencies updated** - Run `cargo update` regularly
2. **Enable Dependabot** - We recommend enabling Dependabot security updates
3. **Review code changes** - Be cautious when accepting contributions
4. **Report issues promptly** - If you find a security issue, report it immediately
5. **Validate inputs** - Use `validate_input()` for consensus-critical code
6. **Choose correct path** - Use scalar for determinism, SIMD for performance

## Security Audit History

| Date | Auditor | Scope | Result |
|------|---------|-------|--------|
| 2025-01-15 | Internal | All modules | Passed |

## Recognition

We appreciate security researchers who help keep our project safe. Contributors who report valid security vulnerabilities will be:

- Listed in our security advisories (with permission)
- Credited in release notes
- Eligible for recognition in our Hall of Fame (coming soon)

## Contact

For general security questions (non-vulnerability reports):
- Open a discussion: https://github.com/SuperInstance/constraint-theory-core/discussions
- Email: security@superinstance.ai

---

Thank you for helping keep constraint-theory-core and our users safe!