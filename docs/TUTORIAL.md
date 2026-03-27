# ConstraintTheory Tutorial

**A Step-by-Step Guide for Beginners**

**Last Updated:** 2026-03-17

---

## Introduction

This tutorial will guide you through using ConstraintTheory, from basic concepts to practical applications. No advanced mathematics background is required.

### What You'll Learn

1. Understanding Pythagorean snapping
2. Setting up the environment
3. Basic operations with the manifold
4. Performance optimization techniques
5. Practical use cases

### Prerequisites

- Basic Rust knowledge (or willingness to learn)
- A development environment with Rust installed
- About 30 minutes of time

---

## Part 1: Understanding the Concept

### What is Pythagorean Snapping?

Imagine you have a compass and you're trying to point it exactly northeast (45 degrees). In the real world, you might point it at 44.7 degrees or 45.3 degrees - slightly off.

Pythagorean snapping is like having a grid of "allowed" directions. When you point in any direction, the system finds the closest "allowed" direction and snaps your vector to it.

**Example:**
- You provide: `(0.612, 0.791)` (slightly off from the 3-4-5 triangle)
- System snaps to: `(0.6, 0.8)` (the exact 3-4-5 ratio)

### Why Pythagorean Triples?

Pythagorean triples (like 3-4-5, 5-12-13, 8-15-17) have special properties:

1. **Exact arithmetic**: No floating-point errors
2. **Integer ratios**: Clean mathematical properties
3. **Abundant**: Infinite supply of them

### What Does "Deterministic" Mean?

Deterministic means: the same input always produces the same output. Unlike neural networks that might give slightly different results each time, ConstraintTheory is mathematically guaranteed to produce identical results every time.

---

## Part 2: Setting Up

### Step 1: Install Rust

If you don't have Rust installed:

```bash
# On macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# On Windows, download from https://rustup.rs
```

### Step 2: Clone the Repository

```bash
git clone https://github.com/SuperInstance/constraint-theory
cd constraint-theory
```

### Step 3: Verify Installation

```bash
# Build the project
cargo build --release

# Run tests
cargo test --release

# You should see all tests pass
```

---

## Part 3: Your First Snap

### Basic Code Example

Create a file called `my_first_snap.rs`:

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

fn main() {
    // Step 1: Create a manifold with 200 density
    // This creates ~1000 valid Pythagorean states
    let manifold = PythagoreanManifold::new(200);

    // Step 2: Define a vector to snap
    let my_vector = [0.6, 0.8];

    // Step 3: Snap it to the nearest Pythagorean triple
    let (snapped, noise) = snap(&manifold, my_vector);

    // Step 4: Print results
    println!("Original: ({}, {})", my_vector[0], my_vector[1]);
    println!("Snapped:  ({}, {})", snapped[0], snapped[1]);
    println!("Noise:    {}", noise);

    // Step 5: Verify it's an exact match
    if noise < 0.001 {
        println!("This is an exact Pythagorean triple!");
    }
}
```

### Run Your Code

```bash
# Create examples directory if needed
# Put your file in crates/constraint-theory-core/examples/

cargo run --release --example my_first_snap
```

### Expected Output

```
Original: (0.6, 0.8)
Snapped:  (0.6, 0.8)
Noise:    0.0000001
This is an exact Pythagorean triple!
```

### Understanding the Output

- **Original**: What you provided
- **Snapped**: The nearest valid Pythagorean triple
- **Noise**: How far off your original was (0 = perfect match)

---

## Part 4: Working with Multiple Vectors

### Batch Processing

When you have many vectors, use batch processing for better performance:

```rust
use constraint_theory_core::PythagoreanManifold;

fn main() {
    let manifold = PythagoreanManifold::new(200);

    // Create multiple vectors
    let vectors = vec![
        [0.6, 0.8],   // 3-4-5 triangle
        [0.8, 0.6],   // 4-3-5 triangle
        [0.28, 0.96], // 7-24-25 triangle
        [0.707, 0.707], // ~45 degrees (not exact)
        [0.5, 0.866],   // ~60 degrees (not exact)
    ];

    // Process all at once with SIMD optimization
    let results = manifold.snap_batch_simd(&vectors);

    // Print results
    println!("Vector Processing Results:\n");
    println!("{:<15} {:<15} {:<10}", "Original", "Snapped", "Noise");
    println!("{}", "-".repeat(45));

    for (i, (original, (snapped, noise))) in vectors.iter()
        .zip(results.iter())
        .enumerate()
    {
        println!(
            "({:.3}, {:.3})    ({:.3}, {:.3})    {:.4}",
            original[0], original[1],
            snapped[0], snapped[1],
            noise
        );
    }

    // Count exact matches
    let exact_count = results.iter()
        .filter(|(_, noise)| *noise < 0.001)
        .count();

    println!("\nExact matches: {}/{}", exact_count, vectors.len());
}
```

---

## Part 5: Understanding Performance

### The KD-Tree Advantage

ConstraintTheory uses a KD-tree for fast lookups. Here's what that means:

**Without KD-tree (brute force):**
- Check every point: O(n) time
- For 1000 points: ~1000 comparisons per query

**With KD-tree:**
- Binary search: O(log n) time
- For 1000 points: ~10 comparisons per query

### Benchmark Your System

```bash
# Run the built-in benchmark
cargo run --release --example bench

# Run comparison benchmark
cargo run --release --example bench_comparison
```

### Performance Tips

1. **Use Release Mode**: Always compile with `--release`
2. **Reuse the Manifold**: Build once, query many times
3. **Use Batch Operations**: `snap_batch_simd` for multiple vectors
4. **Choose Right Density**: Higher density = more accuracy but slower

---

## Part 6: Practical Example - Direction Classification

Let's build a simple direction classifier:

```rust
use constraint_theory_core::{PythagoreanManifold, snap};

#[derive(Debug)]
enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

fn classify_direction(vector: [f32; 2]) -> Direction {
    // Normalize to unit vector
    let norm = (vector[0] * vector[0] + vector[1] * vector[1]).sqrt();
    let x = vector[0] / norm;
    let y = vector[1] / norm;

    // Calculate angle in degrees
    let angle = y.atan2(x).to_degrees();
    let angle = if angle < 0.0 { angle + 360.0 } else { angle };

    // Classify into 8 directions
    match angle {
        337.5..=360.0 | 0.0..22.5 => Direction::East,
        22.5..67.5 => Direction::Northeast,
        67.5..112.5 => Direction::North,
        112.5..157.5 => Direction::Northwest,
        157.5..202.5 => Direction::West,
        202.5..247.5 => Direction::Southwest,
        247.5..292.5 => Direction::South,
        292.5..337.5 => Direction::Southeast,
        _ => Direction::East, // Fallback
    }
}

fn main() {
    let manifold = PythagoreanManifold::new(200);

    let test_vectors = vec![
        ([1.0, 0.0], "Pointing right"),
        ([0.0, 1.0], "Pointing up"),
        ([-1.0, 0.0], "Pointing left"),
        ([0.0, -1.0], "Pointing down"),
        ([0.707, 0.707], "Pointing diagonal"),
        ([0.6, 0.8], "3-4-5 triangle"),
    ];

    println!("Direction Classification with Geometric Snapping\n");
    println!("{:<20} {:<12} {:<12}", "Description", "Direction", "Noise");
    println!("{}", "-".repeat(50));

    for (vector, description) in test_vectors {
        let (snapped, noise) = snap(&manifold, vector);
        let direction = classify_direction(snapped);

        println!(
            "{:<20} {:<12} {:.4}",
            description,
            format!("{:?}", direction),
            noise
        );
    }

    println!("\nNotice: The snapped vector is used for classification,");
    println!("ensuring deterministic results regardless of input noise.");
}
```

---

## Part 7: Common Patterns

### Pattern 1: Quantizing Embeddings

```rust
// Reduce floating-point vectors to discrete states
fn quantize_embedding(manifold: &PythagoreanManifold, embedding: [f32; 2]) -> usize {
    let (snapped, _) = manifold.snap(embedding);

    // Find index of snapped value
    for (i, &state) in manifold.states().iter().enumerate() {
        if (state[0] - snapped[0]).abs() < 0.001 &&
           (state[1] - snapped[1]).abs() < 0.001 {
            return i;
        }
    }
    0 // Fallback
}
```

### Pattern 2: Measuring Similarity

```rust
// Compare two vectors via their snapped states
fn geometric_similarity(
    manifold: &PythagoreanManifold,
    v1: [f32; 2],
    v2: [f32; 2]
) -> f32 {
    let (s1, _) = manifold.snap(v1);
    let (s2, _) = manifold.snap(v2);

    // If they snap to the same point, they're geometrically equivalent
    if (s1[0] - s2[0]).abs() < 0.001 && (s1[1] - s2[1]).abs() < 0.001 {
        return 1.0;
    }

    // Otherwise, compute cosine similarity
    s1[0] * s2[0] + s1[1] * s2[1]
}
```

### Pattern 3: Validation

```rust
// Validate that a vector is an exact Pythagorean triple
fn is_exact_pythagorean(manifold: &PythagoreanManifold, vector: [f32; 2]) -> bool {
    let (_, noise) = manifold.snap(vector);
    noise < 0.0001
}
```

---

## Part 8: Troubleshooting

### Common Issues

**Issue 1: "noise is always high"**
- Your input vectors might not be normalized
- Solution: Divide by the norm before snapping

```rust
let norm = (x * x + y * y).sqrt();
let normalized = [x / norm, y / norm];
let (snapped, noise) = manifold.snap(normalized);
```

**Issue 2: "Performance is slow"**
- Make sure you're using `--release` mode
- Use `snap_batch_simd` for multiple vectors
- Don't rebuild the manifold in a loop

**Issue 3: "Results seem wrong"**
- Check that your input is in the range [-1, 1]
- Verify the manifold density is appropriate
- Run the test suite: `cargo test --release`

---

## Part 9: Next Steps

### Explore the Documentation

- `docs/MATHEMATICAL_FOUNDATIONS_DEEP_DIVE.md` - Theory
- `docs/BENCHMARKS.md` - Performance details
- `docs/DISCLAIMERS.md` - Important clarifications

### Try the Examples

```bash
# ML demonstration
cargo run --release --example ml_demo

# Benchmark comparison
cargo run --release --example bench_comparison
```

### Run the Web Simulator

```bash
cd web-simulator
npm install
npm run dev
# Open http://localhost:8787
```

---

## Part 10: Summary

### What You've Learned

1. Pythagorean snapping maps vectors to exact geometric ratios
2. The manifold contains all valid states
3. KD-tree provides O(log n) lookup performance
4. Batch operations enable SIMD optimization
5. Results are deterministic and exact

### When to Use ConstraintTheory

**Good for:**
- Direction/angle calculations
- Vector quantization
- Deterministic transformations
- Educational purposes

**Not ideal for:**
- General-purpose constraint solving (use OR-Tools)
- High-dimensional data (>2D currently)
- Applications requiring exact original values

### Getting Help

- Open an issue on GitHub
- Read the documentation in `docs/`
- Run the examples to understand patterns

---

**Congratulations!** You've completed the ConstraintTheory tutorial.

---

**Document Version:** 1.0
**Last Updated:** 2026-03-17
