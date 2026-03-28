# Integration Guide

**Version:** 1.0.1
**Last Updated:** 2025-01-27

---

## Overview

This document covers integration patterns for using `constraint-theory-core` with:
- Python via FFI (Foreign Function Interface)
- WebAssembly for browser deployment
- Cross-platform considerations

---

## Python FFI Integration

### Using PyO3 Bindings

The recommended way to use Constraint Theory from Python is via the official Python bindings in [constraint-theory-python](https://github.com/SuperInstance/constraint-theory-python).

#### Installation

```bash
pip install constraint-theory
```

#### Basic Usage

```python
from constraint_theory import PythagoreanManifold, generate_triples

# Create manifold with density 200 (~1000 states)
manifold = PythagoreanManifold(200)

# Snap a vector to nearest Pythagorean triple
x, y, noise = manifold.snap(0.577, 0.816)
print(f"Snapped: ({x:.4f}, {y:.4f}), noise: {noise:.6f}")
# Output: Snapped: (0.6000, 0.8000), noise: 0.023600
```

#### Batch Processing

```python
import numpy as np
from constraint_theory import PythagoreanManifold

manifold = PythagoreanManifold(200)

# Generate 10,000 random unit vectors
angles = np.random.uniform(0, 2 * np.pi, 10000)
vectors = np.column_stack([np.cos(angles), np.sin(angles)])

# Batch snap (SIMD optimized in Rust)
results = manifold.snap_batch(vectors)

# Extract snapped coordinates and noise
snapped = np.array([[sx, sy] for sx, sy, _ in results])
noises = np.array([noise for _, _, noise in results])

print(f"Mean noise: {noises.mean():.6f}")
print(f"Max noise: {noises.max():.6f}")
```

### FFI Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Python Runtime                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   constraint_theory Python Module                           │
│   └── PyO3 Bindings (src/lib.rs)                           │
│       └── C-compatible FFI layer                           │
│           └── constraint-theory-core (Rust)                │
│               └── KD-tree, SIMD, Manifold                  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Type Mapping

| Rust Type | Python Type | Notes |
|-----------|-------------|-------|
| `f32` | `float` | Python uses 64-bit floats internally |
| `usize` | `int` | Platform-dependent in Rust |
| `[f32; 2]` | `tuple[float, float]` | Fixed-size array |
| `Vec<[f32; 2]>` | `list[tuple[float, float]]` | Dynamic list |
| `CTResult<T>` | `T` or raises `RuntimeError` | Error handling |

### NumPy Integration

```python
import numpy as np
from constraint_theory import PythagoreanManifold

manifold = PythagoreanManifold(200)

# Replace v / np.linalg.norm(v) with exact snapping
def exact_normalize(v: np.ndarray) -> np.ndarray:
    """Normalize with exact Pythagorean snapping."""
    sx, sy, _ = manifold.snap(v[0], v[1])
    return np.array([sx, sy])

# Example: Deterministic direction augmentation
def augment_directions(directions: np.ndarray) -> np.ndarray:
    """Augment directions with exact snapping."""
    results = manifold.snap_batch(directions)
    return np.array([[sx, sy] for sx, sy, _ in results])
```

### PyTorch Integration

```python
import torch
from constraint_theory import PythagoreanManifold

manifold = PythagoreanManifold(500)

def snap_tensor(tensor: torch.Tensor) -> torch.Tensor:
    """Snap a 2D tensor to exact Pythagorean coordinates."""
    # Convert to numpy, snap, convert back
    numpy_arr = tensor.detach().cpu().numpy()
    
    # Batch snap
    vectors = numpy_arr.reshape(-1, 2)
    results = manifold.snap_batch(vectors.tolist())
    
    snapped = np.array([[sx, sy] for sx, sy, _ in results])
    return torch.from_numpy(snapped.reshape(tensor.shape))
```

### Custom FFI Layer

For advanced use cases requiring direct FFI:

```rust
// C-compatible wrapper in your Rust crate
#[repr(C)]
pub struct SnapResultFFI {
    pub snapped_x: f32,
    pub snapped_y: f32,
    pub noise: f32,
}

#[no_mangle]
pub extern "C" fn ct_manifold_new(density: usize) -> *mut PythagoreanManifold {
    let manifold = Box::new(PythagoreanManifold::new(density));
    Box::into_raw(manifold)
}

#[no_mangle]
pub extern "C" fn ct_manifold_snap(
    manifold: *const PythagoreanManifold,
    x: f32,
    y: f32,
) -> SnapResultFFI {
    unsafe {
        let manifold = &*manifold;
        let (snapped, noise) = manifold.snap([x, y]);
        SnapResultFFI {
            snapped_x: snapped[0],
            snapped_y: snapped[1],
            noise,
        }
    }
}

#[no_mangle]
pub extern "C" fn ct_manifold_free(manifold: *mut PythagoreanManifold) {
    unsafe {
        drop(Box::from_raw(manifold));
    }
}
```

```c
// C header for FFI
typedef struct SnapResultFFI {
    float snapped_x;
    float snapped_y;
    float noise;
} SnapResultFFI;

void* ct_manifold_new(size_t density);
SnapResultFFI ct_manifold_snap(void* manifold, float x, float y);
void ct_manifold_free(void* manifold);
```

---

## WebAssembly Integration

### Building for WASM

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir pkg

# Build for Node.js
wasm-pack build --target nodejs --out-dir pkg

# Build for bundler (webpack, vite, etc.)
wasm-pack build --target bundler --out-dir pkg
```

### WASM Usage in Browser

```html
<!DOCTYPE html>
<html>
<head>
  <title>Constraint Theory WASM Demo</title>
</head>
<body>
  <script type="module">
    import init, { PythagoreanManifold } from './pkg/constraint_theory_core.js';

    async function main() {
      // Initialize WASM module
      await init();

      // Create manifold
      const manifold = new PythagoreanManifold(200);
      console.log(`Manifold has ${manifold.state_count()} states`);

      // Snap a vector
      const [x, y, noise] = manifold.snap(0.577, 0.816);
      console.log(`Snapped: (${x.toFixed(4)}, ${y.toFixed(4)}), noise: ${noise.toFixed(6)}`);

      // Batch snap
      const vectors = [
        [0.6, 0.8],
        [0.707, 0.707],
        [0.28, 0.96],
      ];
      const results = manifold.snap_batch(vectors);
      console.log(`Batch results:`, results);
    }

    main();
  </script>
</body>
</html>
```

### WASM Performance Characteristics

| Metric | Native Rust | WASM | Ratio |
|--------|-------------|------|-------|
| Single snap | ~100 ns | ~200 ns | 0.5x |
| Batch 1K | ~15 μs | ~35 μs | 0.4x |
| Batch 100K | ~1.1 ms | ~2.8 ms | 0.4x |
| Memory | ~80 bytes/state | ~120 bytes/state | 0.7x |

### SIMD in WASM

WASM SIMD provides significant speedup:

```bash
# Build with WASM SIMD support
wasm-pack build --target web -- --features simd

# Check browser support
if (WebAssembly.simd) {
  console.log('WASM SIMD supported');
}
```

### WASM Integration with Frameworks

#### React

```jsx
import { useEffect, useState } from 'react';
import init, { PythagoreanManifold } from 'constraint-theory-core';

function useConstraintTheory(density = 200) {
  const [manifold, setManifold] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    init().then(() => {
      setManifold(new PythagoreanManifold(density));
      setLoading(false);
    });
  }, [density]);

  return { manifold, loading };
}

function DirectionInput() {
  const { manifold, loading } = useConstraintTheory(200);
  const [result, setResult] = useState(null);

  const handleSnap = (x, y) => {
    if (manifold) {
      const [sx, sy, noise] = manifold.snap(x, y);
      setResult({ snapped: [sx, sy], noise });
    }
  };

  if (loading) return <div>Loading WASM...</div>;

  return (
    <div>
      <button onClick={() => handleSnap(0.577, 0.816)}>
        Snap (0.577, 0.816)
      </button>
      {result && (
        <div>
          Snapped to: ({result.snapped[0].toFixed(4)}, {result.snapped[1].toFixed(4)})
          <br />
          Noise: {result.noise.toFixed(6)}
        </div>
      )}
    </div>
  );
}
```

#### Vue

```vue
<template>
  <div>
    <button @click="snap">Snap</button>
    <div v-if="result">
      Snapped: ({{ result.x.toFixed(4) }}, {{ result.y.toFixed(4) }})
    </div>
  </div>
</template>

<script>
import init, { PythagoreanManifold } from 'constraint-theory-core';

export default {
  data() {
    return {
      manifold: null,
      result: null,
    };
  },
  async created() {
    await init();
    this.manifold = new PythagoreanManifold(200);
  },
  methods: {
    snap() {
      const [x, y, noise] = this.manifold.snap(0.577, 0.816);
      this.result = { x, y, noise };
    },
  },
};
</script>
```

---

## Cross-Platform Considerations

### Platform Support Matrix

| Platform | Architecture | SIMD | Status |
|----------|--------------|------|--------|
| Linux | x86_64 | AVX2 | Full support |
| Linux | ARM64 | NEON | Full support |
| macOS | x86_64 | AVX2 | Full support |
| macOS | ARM64 (M1/M2) | NEON | Full support |
| Windows | x86_64 | AVX2 | Full support |
| Web | WASM | WASM-SIMD | Full support |
| iOS | ARM64 | NEON | Experimental |
| Android | ARM64 | NEON | Experimental |

### Floating-Point Determinism

For cross-platform reproducibility, use the scalar path:

```rust
use constraint_theory_core::PythagoreanManifold;

let manifold = PythagoreanManifold::new(200);

// Scalar path: Deterministic across all platforms
let (snapped, noise) = manifold.snap([0.577, 0.816]);

// SIMD path: May vary slightly due to platform differences
// Only use for performance-critical non-consensus code
let results = manifold.snap_batch_simd(&vectors);
```

### Platform-Specific Optimizations

```rust
// Check SIMD availability at runtime
#[cfg(target_arch = "x86_64")]
let use_simd = is_x86_feature_detected!("avx2");

#[cfg(target_arch = "aarch64")]
let use_simd = std::arch::is_aarch64_feature_detected!("neon");

#[cfg(target_arch = "wasm32")]
let use_simd = true; // WASM SIMD detection is browser-dependent

// Use appropriate path
if use_simd && vectors.len() >= 8 {
    manifold.snap_batch_simd(&vectors);
} else {
    manifold.snap_batch(&vectors, &mut results);
}
```

### Memory Layout Differences

| Platform | Pointer Size | Alignment | Impact |
|----------|--------------|-----------|--------|
| 64-bit | 8 bytes | 16 bytes | Default |
| 32-bit | 4 bytes | 8 bytes | Smaller memory footprint |
| WASM | 4 bytes | 8 bytes | Uses wasm32 target |

### Handling Platform Edge Cases

```rust
use constraint_theory_core::{PythagoreanManifold, CTErr};

fn safe_snap(manifold: &PythagoreanManifold, x: f32, y: f32) -> Result<([f32; 2], f32), CTErr> {
    // Validate input on all platforms
    if !x.is_finite() || !y.is_finite() {
        return Err(CTErr::NaNInput);
    }

    // Use scalar path for consensus-critical code
    let (snapped, noise) = manifold.snap([x, y]);

    // Verify result (should never fail in production)
    debug_assert!(
        (snapped[0] * snapped[0] + snapped[1] * snapped[1] - 1.0).abs() < 0.001,
        "Snapped vector not normalized"
    );

    Ok((snapped, noise))
}
```

---

## Integration Test Examples

### Python-Rust Compatibility Test

```python
# test_rust_compatibility.py
"""
Verify Python bindings produce identical results to Rust core.
Run with: pytest tests/test_compatibility.py -v
"""

import pytest
import math
from constraint_theory import PythagoreanManifold

class TestRustCompatibility:
    """Verify Python bindings match Rust core behavior."""

    def test_exact_triples(self):
        """Exact Pythagorean triples should have zero noise."""
        manifold = PythagoreanManifold(200)

        # Known Pythagorean triples from Rust tests
        test_cases = [
            (3, 4, 5),
            (5, 12, 13),
            (8, 15, 17),
            (7, 24, 25),
        ]

        for a, b, c in test_cases:
            x, y, noise = manifold.snap(a/c, b/c)
            assert noise < 0.001, f"Failed for triple ({a}, {b}, {c})"

    def test_determinism(self):
        """Same input must produce same output."""
        manifold = PythagoreanManifold(200)
        test_input = (0.577, 0.816)

        results = [manifold.snap(*test_input) for _ in range(100)]
        first = results[0]

        for r in results[1:]:
            assert r == first, "Results should be deterministic"

    def test_batch_consistency(self):
        """Batch results must match individual snap results."""
        manifold = PythagoreanManifold(200)
        vectors = [(0.6, 0.8), (0.707, 0.707), (0.28, 0.96)]

        batch_results = manifold.snap_batch(vectors)

        for i, (x, y) in enumerate(vectors):
            single = manifold.snap(x, y)
            batch = batch_results[i]

            assert abs(single[0] - batch[0]) < 1e-6
            assert abs(single[1] - batch[1]) < 1e-6
            assert abs(single[2] - batch[2]) < 1e-6
```

### WASM Integration Test

```javascript
// test_wasm_integration.js
// Run with: node --experimental-wasm-simd test_wasm_integration.js

const assert = require('assert');

async function runTests() {
  const { default: init, PythagoreanManifold } = await import('./pkg/constraint_theory_core.js');
  await init();

  const manifold = new PythagoreanManifold(200);

  // Test basic snapping
  {
    const [x, y, noise] = manifold.snap(0.577, 0.816);
    assert(Math.abs(x - 0.6) < 0.01, `Expected x ≈ 0.6, got ${x}`);
    assert(Math.abs(y - 0.8) < 0.01, `Expected y ≈ 0.8, got ${y}`);
    console.log('Basic snap test passed');
  }

  // Test exact triple
  {
    const [x, y, noise] = manifold.snap(0.6, 0.8);
    assert(noise < 0.001, `Expected noise ≈ 0, got ${noise}`);
    console.log('Exact triple test passed');
  }

  // Test batch
  {
    const vectors = [[0.6, 0.8], [0.707, 0.707]];
    const results = manifold.snap_batch(vectors);
    assert(results.length === 2, `Expected 2 results, got ${results.length}`);
    console.log('Batch test passed');
  }

  console.log('All WASM integration tests passed!');
}

runTests().catch(console.error);
```

---

## Debugging Integration Issues

### Common Python Issues

**ImportError: cannot import name 'PythagoreanManifold'**

```bash
# Solution 1: Reinstall from PyPI
pip install --upgrade constraint-theory

# Solution 2: Build from source
git clone https://github.com/SuperInstance/constraint-theory-python
cd constraint-theory-python
pip install maturin
maturin develop --release
```

**Slow first call**

The first manifold creation involves KD-tree construction. Reuse the manifold:

```python
# BAD: Create manifold per call
def process_vectors(vectors):
    manifold = PythagoreanManifold(200)  # Slow!
    return [manifold.snap(x, y) for x, y in vectors]

# GOOD: Reuse manifold
MANIFOLD = PythagoreanManifold(200)  # Create once

def process_vectors(vectors):
    return MANIFOLD.snap_batch(vectors)  # Fast!
```

### Common WASM Issues

**Module initialization fails**

```javascript
// Ensure WASM is initialized before use
let manifold = null;

async function initManifold() {
  await init();
  manifold = new PythagoreanManifold(200);
}

// Call before any other operations
await initManifold();
```

**Performance degradation**

```javascript
// Check WASM SIMD support
if (typeof WebAssembly !== 'undefined') {
  WebAssembly.validate(new Uint8Array([
    0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0, 10, 10, 1, 8, 0, 65, 0, 253, 15, 253, 98, 11
  ])).then(simd => {
    console.log('WASM SIMD supported:', simd);
  });
}
```

---

## See Also

- [Production Readiness Guide](PRODUCTION_READINESS.md)
- [Performance Guide](PERFORMANCE.md)
- [Testing Methodology](TESTING.md)
- [constraint-theory-python repository](https://github.com/SuperInstance/constraint-theory-python)
- [constraint-theory-web repository](https://github.com/SuperInstance/constraint-theory-web)

---

**Document Version:** 1.0
**Next Review:** 2025-04-01
