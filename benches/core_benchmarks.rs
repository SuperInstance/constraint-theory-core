//! Criterion Benchmarks for Constraint Theory Core
//!
//! These benchmarks measure the performance of core operations:
//! - Manifold snapping (single and batch)
//! - Quantization (all modes)
//! - Hidden dimension encoding
//! - Holonomy verification
//!
//! # Running Benchmarks
//!
//! ```bash
//! cargo bench
//! ```
//!
//! For specific benchmark groups:
//! ```bash
//! cargo bench -- manifold
//! cargo bench -- quantizer
//! cargo bench -- hidden_dims
//! cargo bench -- holonomy
//! ```

use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

use constraint_theory_core::{
    hidden_dimensions::{hidden_dim_count, lift_to_hidden, HiddenDimensionConfig},
    holonomy::{compute_holonomy, identity_matrix, rotation_x, rotation_y, rotation_z},
    manifold::PythagoreanManifold,
    quantizer::{PythagoreanQuantizer, QuantizationMode},
};

// ============================================================================
// Manifold Benchmarks
// ============================================================================

fn bench_manifold_snap(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifold_snap");

    // Test different manifold densities
    for density in [50, 100, 200, 500] {
        let manifold = PythagoreanManifold::new(density);
        let state_count = manifold.state_count();

        group.bench_with_input(
            BenchmarkId::new("density", density),
            &manifold,
            |b, manifold| {
                b.iter(|| {
                    black_box(manifold.snap(black_box([0.6, 0.8])))
                });
            },
        );
    }

    group.finish();
}

fn bench_manifold_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifold_batch");

    let manifold = PythagoreanManifold::new(200);

    // Test different batch sizes
    for batch_size in [8, 64, 256, 1024] {
        let vectors: Vec<[f32; 2]> = (0..batch_size)
            .map(|i| {
                let angle = (i as f32) * 0.01;
                [angle.cos(), angle.sin()]
            })
            .collect();

        // SIMD batch
        group.bench_with_input(
            BenchmarkId::new("simd_batch", batch_size),
            &vectors,
            |b, vectors| {
                b.iter(|| {
                    black_box(manifold.snap_batch_simd(black_box(vectors)))
                });
            },
        );

        // Scalar batch
        group.bench_with_input(
            BenchmarkId::new("scalar_batch", batch_size),
            &vectors,
            |b, vectors| {
                b.iter(|| {
                    let mut results = vec![([0.0, 0.0], 0.0f32); vectors.len()];
                    manifold.snap_batch(vectors, &mut results);
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_manifold_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifold_construction");

    for density in [50, 100, 200, 500] {
        group.bench_with_input(
            BenchmarkId::new("build", density),
            &density,
            |b, &density| {
                b.iter(|| {
                    black_box(PythagoreanManifold::new(black_box(density)))
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Quantizer Benchmarks
// ============================================================================

fn bench_quantizer_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantizer_modes");

    // Generate test data
    let data_4d = vec![0.6, 0.8, 0.0, 0.0];
    let data_128d: Vec<f64> = (0..128)
        .map(|i| ((i as f64 + 1.0) / 129.0).sin())
        .collect();
    let data_512d: Vec<f64> = (0..512)
        .map(|i| ((i as f64 + 1.0) / 513.0).sin())
        .collect();

    let modes = [
        ("ternary", QuantizationMode::Ternary),
        ("polar", QuantizationMode::Polar),
        ("turbo", QuantizationMode::Turbo),
        ("hybrid", QuantizationMode::Hybrid),
    ];

    for (name, mode) in modes {
        let quantizer = PythagoreanQuantizer::new(mode, 8);

        // 4D
        group.bench_with_input(
            BenchmarkId::new(format!("{}_4d", name), 0),
            &data_4d,
            |b, data| {
                b.iter(|| {
                    black_box(quantizer.quantize(black_box(data)))
                });
            },
        );

        // 128D
        group.bench_with_input(
            BenchmarkId::new(format!("{}_128d", name), 0),
            &data_128d,
            |b, data| {
                b.iter(|| {
                    black_box(quantizer.quantize(black_box(data)))
                });
            },
        );

        // 512D
        group.bench_with_input(
            BenchmarkId::new(format!("{}_512d", name), 0),
            &data_512d,
            |b, data| {
                b.iter(|| {
                    black_box(quantizer.quantize(black_box(data)))
                });
            },
        );
    }

    group.finish();
}

fn bench_quantizer_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("quantizer_batch");

    let quantizer = PythagoreanQuantizer::for_embeddings();

    for batch_size in [8, 64, 256, 1024] {
        let vectors: Vec<Vec<f64>> = (0..batch_size)
            .map(|i| {
                let angle = (i as f64) * 0.01;
                vec![angle.cos(), angle.sin()]
            })
            .collect();

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            &vectors,
            |b, vectors| {
                b.iter(|| {
                    black_box(quantizer.quantize_batch(black_box(vectors)))
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Hidden Dimensions Benchmarks
// ============================================================================

fn bench_hidden_dim_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("hidden_dims");

    let epsilons = [1e-4, 1e-6, 1e-10, 1e-15];

    for epsilon in epsilons {
        group.bench_with_input(
            BenchmarkId::new("count", format!("1e{}", -epsilon.log10() as i32)),
            &epsilon,
            |b, &epsilon| {
                b.iter(|| {
                    black_box(hidden_dim_count(black_box(epsilon)))
                });
            },
        );
    }

    group.finish();
}

fn bench_lift_to_hidden(c: &mut Criterion) {
    let mut group = c.benchmark_group("lift_to_hidden");

    let point = vec![0.6, 0.8, 0.0, 0.0];

    for k in [4, 10, 20, 34] {
        group.bench_with_input(
            BenchmarkId::new("k_dims", k),
            &k,
            |b, &k| {
                b.iter(|| {
                    black_box(lift_to_hidden(black_box(&point), black_box(k)))
                });
            },
        );
    }

    group.finish();
}

fn bench_hidden_dim_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("hidden_dim_encoding");

    let points: Vec<Vec<f64>> = vec![
        vec![0.6, 0.8],
        vec![0.707, 0.707],
        vec![0.5, 0.866],
    ];

    for epsilon in [1e-4, 1e-6, 1e-10] {
        let config = HiddenDimensionConfig::new(epsilon);

        group.bench_with_input(
            BenchmarkId::new("encode", format!("1e{}", -epsilon.log10() as i32)),
            &config,
            |b, config| {
                b.iter(|| {
                    for point in &points {
                        black_box(config.encode(black_box(point)));
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Holonomy Benchmarks
// ============================================================================

fn bench_holonomy_compute(c: &mut Criterion) {
    let mut group = c.benchmark_group("holonomy");

    // Test different cycle lengths
    for cycle_len in [1, 4, 16, 64] {
        let cycle: Vec<_> = (0..cycle_len)
            .map(|i| {
                if i % 2 == 0 {
                    rotation_x(0.1)
                } else {
                    rotation_y(0.1)
                }
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("cycle_len", cycle_len),
            &cycle,
            |b, cycle| {
                b.iter(|| {
                    black_box(compute_holonomy(black_box(cycle)))
                });
            },
        );
    }

    group.finish();
}

fn bench_rotation_matrices(c: &mut Criterion) {
    let mut group = c.benchmark_group("rotation_matrices");

    let angles: Vec<f64> = (0..100).map(|i| (i as f64) * 0.01).collect();

    group.bench_function("rotation_x", |b| {
        b.iter(|| {
            for &angle in &angles {
                black_box(rotation_x(black_box(angle)));
            }
        });
    });

    group.bench_function("rotation_y", |b| {
        b.iter(|| {
            for &angle in &angles {
                black_box(rotation_y(black_box(angle)));
            }
        });
    });

    group.bench_function("rotation_z", |b| {
        b.iter(|| {
            for &angle in &angles {
                black_box(rotation_z(black_box(angle)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// Integration Benchmarks
// ============================================================================

fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let manifold = PythagoreanManifold::new(200);
    let quantizer = PythagoreanQuantizer::for_embeddings();
    let config = HiddenDimensionConfig::new(1e-6);

    group.bench_function("snap_only", |b| {
        b.iter(|| {
            black_box(manifold.snap(black_box([0.6, 0.8])))
        });
    });

    group.bench_function("quantize_only", |b| {
        let data = vec![0.6, 0.8, 0.0, 0.0];
        b.iter(|| {
            black_box(quantizer.quantize(black_box(&data)))
        });
    });

    group.bench_function("encode_only", |b| {
        let point = vec![0.6, 0.8];
        b.iter(|| {
            black_box(config.encode(black_box(&point)))
        });
    });

    group.bench_function("full_encoding_pipeline", |b| {
        let point = vec![0.6_f64, 0.8];
        b.iter(|| {
            // 1. Lift
            let k = hidden_dim_count(1e-6);
            let lifted = lift_to_hidden(&point, k);
            // 2. Quantize
            let result = quantizer.quantize(&lifted);
            // 3. Project (simplified)
            black_box(&result.data[..2])
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    benches,
    bench_manifold_snap,
    bench_manifold_batch,
    bench_manifold_construction,
    bench_quantizer_modes,
    bench_quantizer_batch,
    bench_hidden_dim_count,
    bench_lift_to_hidden,
    bench_hidden_dim_encoding,
    bench_holonomy_compute,
    bench_rotation_matrices,
    bench_full_pipeline,
);

criterion_main!(benches);
