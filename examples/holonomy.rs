//! Holonomy Verification Example
//!
//! Demonstrates holonomy computation and verification for ensuring
//! global consistency of constraint satisfaction around cycles.
//!
//! # Key Concepts
//!
//! - **Holonomy**: Measures inconsistency when transporting around a closed loop
//! - **Zero holonomy**: Globally consistent constraints
//! - **Non-zero holonomy**: Inconsistent constraints, need resolution

use constraint_theory_core::{
    holonomy::{
        compute_holonomy, compute_edge_holonomy, identity_matrix, rotation_x, rotation_y,
        rotation_z, rotation_from_euler, verify_holonomy, triangular_holonomy, HolonomyChecker,
    },
    PythagoreanManifold,
};

fn main() {
    println!("=== Holonomy Verification Example ===\n");

    // Example 1: Identity rotation (zero holonomy)
    println!("--- Example 1: Identity Rotation ---");
    let id = identity_matrix();
    println!("Identity matrix:");
    println!("  [{:.1}, {:.1}, {:.1}]", id[0][0], id[0][1], id[0][2]);
    println!("  [{:.1}, {:.1}, {:.1}]", id[1][0], id[1][1], id[1][2]);
    println!("  [{:.1}, {:.1}, {:.1}]", id[2][0], id[2][1], id[2][2]);

    let result = compute_holonomy(&[id]);
    println!("\nHolonomy norm: {:.6}", result.norm);
    println!("Is identity: {}", result.is_identity());
    println!("Information: {:.2}\n", result.information);

    // Example 2: Full rotation returns to identity
    println!("--- Example 2: Full Rotation ---");
    let rz_180 = rotation_z(std::f64::consts::PI);
    let result = compute_holonomy(&[rz_180.clone(), rz_180]);

    println!("Two 180° rotations around Z axis:");
    println!("  Holonomy norm: {:.6}", result.norm);
    println!("  Is identity: {}", result.is_identity());
    println!("  Angular deviation: {:.4} radians\n", result.angular_deviation());

    // Example 3: 90-degree rotation doesn't return
    println!("--- Example 3: Incomplete Rotation ---");
    let rz_90 = rotation_z(std::f64::consts::FRAC_PI_2);
    let result = compute_holonomy(&[rz_90]);

    println!("Single 90° rotation around Z axis:");
    println!("  Holonomy norm: {:.6}", result.norm);
    println!("  Is identity: {}", result.is_identity());
    println!("  Angular deviation: {:.4} radians ({:.1}°)\n", 
             result.angular_deviation(), 
             result.angular_deviation().to_degrees());

    // Example 4: Rotation matrix generation
    println!("--- Example 4: Rotation Matrices ---");
    let angle = std::f64::consts::FRAC_PI_4; // 45 degrees

    let rx = rotation_x(angle);
    let ry = rotation_y(angle);
    let rz = rotation_z(angle);

    println!("Rotation matrices for 45° around each axis:");
    println!("\nRotation around X:");
    print_matrix(&rx);
    println!("\nRotation around Y:");
    print_matrix(&ry);
    println!("\nRotation around Z:");
    print_matrix(&rz);
    println!();

    // Example 5: Euler angles
    println!("--- Example 5: Euler Angles ---");
    let euler = rotation_from_euler(0.1, 0.2, 0.3);
    println!("Rotation from Euler(roll=0.1, pitch=0.2, yaw=0.3):");
    print_matrix(&euler);
    println!();

    // Example 6: HolonomyChecker for incremental verification
    println!("--- Example 6: Incremental Holonomy Checker ---");
    let mut checker = HolonomyChecker::new(1e-6);

    println!("Applying rotations incrementally:");
    checker.apply(&rotation_x(0.1));
    println!("After R_x(0.1): step = {}", checker.step_count());

    checker.apply(&rotation_y(0.2));
    println!("After R_y(0.2): step = {}", checker.step_count());

    let partial = checker.check_partial();
    println!("Partial holonomy norm: {:.6}", partial.norm);

    let closed = checker.check_closed();
    println!("Closed holonomy norm: {:.6}", closed.norm);

    checker.reset();
    println!("After reset: step = {}", checker.step_count());
    println!();

    // Example 7: Verify multiple cycles
    println!("--- Example 7: Verify Multiple Cycles ---");
    let cycles = vec![
        vec![identity_matrix()],                                    // Identity cycle
        vec![rotation_x(std::f64::consts::PI), 
             rotation_x(std::f64::consts::PI)],                    // Full X rotation
        vec![rotation_y(std::f64::consts::PI), 
             rotation_y(std::f64::consts::PI)],                    // Full Y rotation
    ];

    let all_zero = verify_holonomy(&cycles, 1e-6);
    println!("All {} cycles have zero holonomy: {}", cycles.len(), all_zero);
    println!();

    // Example 8: Triangular holonomy
    println!("--- Example 8: Triangular Holonomy ---");
    let a = rotation_x(0.1);
    let b = rotation_y(0.2);
    let c = rotation_z(0.3);

    let result = triangular_holonomy(&a, &b, &c);
    println!("Triangular cycle A -> B -> C -> A:");
    println!("  Holonomy norm: {:.6}", result.norm);
    println!("  Is identity: {}", result.is_identity());
    println!();

    // Example 9: Edge-based holonomy
    println!("--- Example 9: Edge-Based Holonomy ---");
    let edges = vec![
        rotation_x(0.5),
        rotation_y(0.5),
    ];

    let result_open = compute_edge_holonomy(&edges, false);
    let result_closed = compute_edge_holonomy(&edges, true);

    println!("Two edge transformations:");
    println!("  Open cycle norm: {:.6}", result_open.norm);
    println!("  Closed cycle norm: {:.6}", result_closed.norm);
    println!();

    // Example 10: Integration with constraint theory
    println!("--- Example 10: Integration with Constraint Theory ---");
    let manifold = PythagoreanManifold::new(200);

    // Create a cycle of snapped vectors
    let v1 = manifold.snap([0.6, 0.8]).0;
    let v2 = manifold.snap([0.8, 0.6]).0;
    let v3 = manifold.snap([1.0, 0.0]).0;

    println!("Snapped vectors (should be unit norm):");
    println!("  v1 = {:?}, norm = {:.4}", v1, (v1[0]*v1[0] + v1[1]*v1[1]).sqrt());
    println!("  v2 = {:?}, norm = {:.4}", v2, (v2[0]*v2[0] + v2[1]*v2[1]).sqrt());
    println!("  v3 = {:?}, norm = {:.4}", v3, (v3[0]*v3[0] + v3[1]*v3[1]).sqrt());

    // These unit vectors satisfy the unit norm constraint consistently
    println!("\nUnit norm constraint is satisfied consistently.");

    println!("\n=== Holonomy Example Complete ===");
}

/// Helper function to print a 3x3 matrix
fn print_matrix(m: &[[f64; 3]; 3]) {
    for row in m {
        println!("  [{:7.4}, {:7.4}, {:7.4}]", row[0], row[1], row[2]);
    }
}
