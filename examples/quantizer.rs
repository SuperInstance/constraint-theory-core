//! Quantizer Example
//!
//! Demonstrates the PythagoreanQuantizer which integrates multiple
//! quantization technologies:
//!
//! - **Ternary (BitNet)**: {-1, 0, 1} for LLM weights
//! - **Polar (PolarQuant)**: Exact unit norm preservation
//! - **Turbo (TurboQuant)**: Near-optimal distortion
//! - **Hybrid**: Auto-select based on input characteristics

use constraint_theory_core::{
    quantizer::{PythagoreanQuantizer, QuantizationMode, Rational},
    PythagoreanManifold,
};

fn main() {
    println!("=== Pythagorean Quantizer Example ===\n");

    // Example 1: Different quantization modes
    println!("--- Example 1: Quantization Modes ---");
    let data = vec![0.6, 0.8, 0.0, 0.0];

    let modes = [
        ("Ternary", QuantizationMode::Ternary),
        ("Polar", QuantizationMode::Polar),
        ("Turbo", QuantizationMode::Turbo),
        ("Hybrid", QuantizationMode::Hybrid),
    ];

    for (name, mode) in modes {
        let quantizer = PythagoreanQuantizer::new(mode, 8);
        let result = quantizer.quantize(&data);

        println!("\n{} mode:", name);
        println!("  Input:  {:?}", data);
        println!("  Output: {:?}", result.data);
        println!("  MSE:    {:.6}", result.mse);
        println!("  Unit norm: {}", result.unit_norm_preserved);
    }
    println!();

    // Example 2: Unit norm preservation (Polar mode)
    println!("--- Example 2: Unit Norm Preservation ---");
    let quantizer = PythagoreanQuantizer::for_embeddings();

    let unit_vectors = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.707, 0.707, 0.0, 0.0],
        vec![0.6, 0.8, 0.0, 0.0],
        vec![0.5, 0.5, 0.5, 0.5],
    ];

    for v in unit_vectors {
        let result = quantizer.quantize(&v);
        let original_norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        let quantized_norm = result.norm();

        println!("Original norm: {:.4} -> Quantized norm: {:.4} (preserved: {})",
                 original_norm, quantized_norm, result.unit_norm_preserved);
    }
    println!();

    // Example 3: Ternary quantization for LLM weights
    println!("--- Example 3: Ternary Quantization (LLM Weights) ---");
    let quantizer = PythagoreanQuantizer::for_llm();

    let weights = vec![-0.8, -0.1, 0.1, 0.9, 0.5, -0.3, 0.05, -0.95];
    let result = quantizer.quantize(&weights);

    println!("Weights: {:?}", weights);
    println!("Quantized: {:?}", result.data);
    println!("MSE: {:.4}", result.mse);
    println!("Memory reduction: 32x (FP32 -> 1-bit ternary)\n");

    // Example 4: Hybrid mode auto-selection
    println!("--- Example 4: Hybrid Mode Auto-Selection ---");
    let quantizer = PythagoreanQuantizer::hybrid();

    let inputs = [
        ("Unit vector", vec![0.6, 0.8]),
        ("Sparse vector", vec![0.01, 0.02, 0.0, 0.0, 0.0, 0.0]),
        ("Dense vector", vec![0.5, 0.6, 0.7, 0.8]),
    ];

    for (name, data) in inputs {
        let result = quantizer.quantize(&data);
        println!("{:15} -> {:?} mode", name, result.mode);
    }
    println!();

    // Example 5: Pythagorean rational representation
    println!("--- Example 5: Pythagorean Rationals ---");
    let quantizer = PythagoreanQuantizer::new(QuantizationMode::Polar, 8);

    let values = [0.6, 0.8, 0.5, 0.707];

    for value in values {
        let (snapped, num, den) = quantizer.snap_to_lattice(value, 50);
        let rational = Rational::new(num, den);

        println!("Value {:.4} -> {}/{} = {:.4} (is_pythagorean: {})",
                 value, num, den, snapped, rational.is_pythagorean());
    }
    println!();

    // Example 6: Batch quantization
    println!("--- Example 6: Batch Quantization ---");
    let quantizer = PythagoreanQuantizer::for_embeddings();

    let vectors: Vec<Vec<f64>> = (0..5)
        .map(|i| {
            let angle = (i as f64) * std::f64::consts::PI / 5.0;
            vec![angle.cos(), angle.sin()]
        })
        .collect();

    let results = quantizer.quantize_batch(&vectors);

    println!("Batch quantized {} vectors:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("  {}: MSE = {:.6}, unit_norm = {}", 
                 i, result.mse, result.unit_norm_preserved);
    }
    println!();

    // Example 7: Integration with manifold
    println!("--- Example 7: Integration with Manifold ---");
    let manifold = PythagoreanManifold::new(200);
    let quantizer = PythagoreanQuantizer::for_embeddings();

    let point = [0.577, 0.816]; // Close to 1/√3, 2/√3

    // Snap with manifold
    let (manifold_snapped, manifold_noise) = manifold.snap(point);
    println!("Manifold snap: {:?} (noise: {:.4})", manifold_snapped, manifold_noise);

    // Quantize
    let result = quantizer.quantize(&[point[0] as f64, point[1] as f64]);
    println!("Quantized:     {:?} (norm: {:.4})", &result.data[..2], result.norm());
    println!();

    // Example 8: Factory methods
    println!("--- Example 8: Factory Methods ---");
    let llm_q = PythagoreanQuantizer::for_llm();
    let emb_q = PythagoreanQuantizer::for_embeddings();
    let vdb_q = PythagoreanQuantizer::for_vector_db();
    let hybrid_q = PythagoreanQuantizer::hybrid();

    println!("for_llm():         mode = {:?}", llm_q.mode);
    println!("for_embeddings():  mode = {:?}", emb_q.mode);
    println!("for_vector_db():   mode = {:?}", vdb_q.mode);
    println!("hybrid():          mode = {:?}", hybrid_q.mode);
    println!();

    // Example 9: Comparing quantization quality
    println!("--- Example 9: Quantization Quality Comparison ---");
    let original = vec![0.6, 0.8, 0.1, 0.2, 0.3, 0.4];

    for bits in [1, 2, 4, 8] {
        let quantizer = PythagoreanQuantizer::new(QuantizationMode::Turbo, bits);
        let result = quantizer.quantize(&original);

        println!("{:2} bits: MSE = {:.6}, levels = {}", 
                 bits, result.mse, 1 << bits);
    }

    println!("\n=== Quantizer Example Complete ===");
}
