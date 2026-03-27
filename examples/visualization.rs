//! Visualization Example
//!
//! This example demonstrates how to use ConstraintTheory for
//! geometric visualization and rendering applications.
//!
//! Run with: cargo run --release --example visualization

use constraint_theory_core::manifold::PythagoreanManifold;
use constraint_theory_core::tile::Tile;
use constraint_theory_core::percolation::FastPercolation;

fn main() {
    println!("================================================");
    println!("ConstraintTheory - Visualization Example");
    println!("================================================\n");

    // Example 1: Grid Snapping for UI Layout
    println!("Example 1: Grid Snapping for UI Layout");
    println!("---------------------------------------\n");

    let manifold = PythagoreanManifold::new(100);

    // Simulate UI element positions (pixels, normalized to 0-1)
    let ui_elements = vec![
        ("Button A", [0.123, 0.456]),
        ("Button B", [0.789, 0.234]),
        ("Panel C", [0.567, 0.890]),
        ("Slider D", [0.345, 0.678]),
    ];

    println!("Snapping UI elements to Pythagorean grid:");
    println!("  Element        Original      -> Snapped       | Noise");
    println!("  -------------------------------------------------------");

    for (name, pos) in &ui_elements {
        let (snapped, noise) = manifold.snap(*pos);
        println!(
            "  {:12}   ({:.3}, {:.3}) -> ({:.3}, {:.3}) | {:.4}",
            name, pos[0], pos[1], snapped[0], snapped[1], noise
        );
    }

    // Example 2: Procedural Generation
    println!("\n\nExample 2: Procedural Generation");
    println!("----------------------------------\n");

    // Generate deterministic random directions
    println!("Generating 8 deterministic directions:");
    for i in 0..8 {
        let angle = (i as f32) * std::f32::consts::PI / 4.0;
        let vec = [angle.cos(), angle.sin()];
        let (snapped, noise) = manifold.snap(vec);

        let degrees = angle * 180.0 / std::f32::consts::PI;
        println!(
            "  {:2}. {:.1}° -> ({:.3}, {:.3}) | noise = {:.4}",
            i + 1, degrees, snapped[0], snapped[1], noise
        );
    }

    // Example 3: Tile Rendering
    println!("\n\nExample 3: Tile Rendering");
    println!("--------------------------\n");

    // Create tiles with different constraints
    let tiles = [
        Tile::new(0),
        Tile::new(1),
        Tile::new(2),
    ];

    println!("Tile constraints for rendering:");
    for (i, tile) in tiles.iter().enumerate() {
        println!(
            "  Tile {}: origin_id = {}, size = {} bytes",
            i,
            tile.origin.id,
            std::mem::size_of_val(tile)
        );
    }

    // Example 4: Rigidity Analysis for Structures
    println!("\n\nExample 4: Rigidity Analysis for Structures");
    println!("--------------------------------------------\n");

    // Analyze structural rigidity using Laman's theorem
    let n_vertices = 6;
    let edges = [(0, 1), (1, 2), (2, 3), (3, 4), (4, 5),
                 (5, 0), (0, 2), (2, 4), (1, 3)];  // 9 edges

    let mut percolation = FastPercolation::new(n_vertices);
    let result = percolation.compute_rigidity(&edges, n_vertices);

    println!("Structure rigidity analysis:");
    println!("  Vertices: {}", n_vertices);
    println!("  Edges: {}", edges.len());
    println!("  Rigid: {}", result.is_rigid);
    println!("  Rank: {}", result.rank);
    println!("  Deficiency: {}", result.deficiency);
    println!("  Rigid fraction: {:.2}", result.rigid_fraction);

    if result.is_rigid {
        println!("  ✓ Structure is rigid (Laman's theorem satisfied)");
    } else {
        println!("  ✗ Structure is flexible (deficiency = {})", result.deficiency);
    }

    // Example 5: Symmetry Detection
    println!("\n\nExample 5: Symmetry Detection");
    println!("-------------------------------\n");

    // Detect symmetric patterns
    let patterns = vec![
        ([1.0, 0.0], "East"),
        ([0.0, 1.0], "North"),
        ([-1.0, 0.0], "West"),
        ([0.0, -1.0], "South"),
    ];

    println!("Symmetry detection (4-fold rotational):");
    println!("  Direction   Snapped       Match");
    println!("  -------------------------------");

    let mut snapped_dirs = Vec::new();
    for (vec, name) in patterns {
        let (snapped, _) = manifold.snap(vec);
        snapped_dirs.push((snapped, name));
        println!("  {:8}   ({:.1}, {:.1})", name, snapped[0], snapped[1]);
    }

    // Check if all snapped to same magnitude
    let magnitudes: Vec<f32> = snapped_dirs.iter()
        .map(|(v, _)| (v[0].powi(2) + v[1].powi(2)).sqrt())
        .collect();

    let all_equal = magnitudes.windows(2)
        .all(|w| (w[0] - w[1]).abs() < 0.01);

    println!("\n  4-fold symmetry: {}", if all_equal { "✓ Yes" } else { "✗ No" });

    // Example 6: Color Quantization
    println!("\n\nExample 6: Color Quantization (RGB to 2D)");
    println!("---------------------------------------------\n");

    // Quantize 2D color space (e.g., RG plane)
    let colors = [
        ([0.8, 0.6], "Orange-ish"),
        ([0.6, 0.8], "Cyan-ish"),
        ([0.5, 0.5], "Gray-ish"),
        ([1.0, 0.0], "Red-ish"),
    ];

    println!("Color quantization to Pythagorean palette:");
    println!("  Color          Original      -> Quantized    | Noise");
    println!("  --------------------------------------------------------");

    for (color, name) in colors {
        let (quantized, noise) = manifold.snap(color);
        println!(
            "  {:12}   ({:.1}, {:.1}) -> ({:.3}, {:.3}) | {:.4}",
            name, color[0], color[1], quantized[0], quantized[1], noise
        );
    }

    println!("\n================================================");
    println!("Visualization Example Complete");
    println!("================================================");
}
