//! Robotics and Motion Planning Example
//!
//! This example demonstrates how to use ConstraintTheory for
//! robotics applications, including motion planning and path optimization.
//!
//! Run with: cargo run --release --example robotics

use constraint_theory_core::manifold::PythagoreanManifold;
use constraint_theory_core::curvature::RicciFlow;
use constraint_theory_core::gauge::GaugeConnection;
use constraint_theory_core::tile::Tile;

fn main() {
    println!("================================================");
    println!("ConstraintTheory - Robotics Example");
    println!("================================================\n");

    // Example 1: Motion Direction Discretization
    println!("Example 1: Motion Direction Discretization");
    println!("------------------------------------------\n");

    let manifold = PythagoreanManifold::new(200);

    // Robot motion commands (continuous directions)
    let motion_commands = vec![
        ("Forward", [0.0, 1.0]),
        ("Diagonal", [0.707, 0.707]),
        ("Turn Left", [-0.5, 0.866]),
        ("Backup", [0.0, -1.0]),
    ];

    println!("Discretizing motion commands:");
    println!("  Command       Original      -> Discretized   | Noise");
    println!("  --------------------------------------------------------");

    for (name, direction) in motion_commands {
        let (discretized, noise) = manifold.snap(direction);
        println!(
            "  {:12}   ({:.3}, {:.3}) -> ({:.3}, {:.3}) | {:.4}",
            name, direction[0], direction[1], discretized[0], discretized[1], noise
        );
    }

    // Example 2: Path Planning
    println!("\n\nExample 2: Path Planning");
    println!("-------------------------\n");

    // Plan path through waypoints
    let waypoints = [
        [0.0, 0.0],
        [0.3, 0.4],
        [0.6, 0.8],
        [1.0, 0.0],
    ];

    println!("Path through waypoints:");
    println!("  Waypoint      Original      -> Snapped");
    println!("  --------------------------------------------");

    let mut snapped_path = Vec::new();
    for (i, waypoint) in waypoints.iter().enumerate() {
        let (snapped, _) = manifold.snap(*waypoint);
        snapped_path.push(snapped);
        println!(
            "  {:2}.        ({:.2}, {:.2}) -> ({:.3}, {:.3})",
            i + 1, waypoint[0], waypoint[1], snapped[0], snapped[1]
        );
    }

    // Calculate path length
    let mut path_length = 0.0;
    for window in snapped_path.windows(2) {
        let dx = window[1][0] - window[0][0];
        let dy = window[1][1] - window[0][1];
        path_length += (dx * dx + dy * dy).sqrt();
    }

    println!("\n  Total path length: {:.3}", path_length);

    // Example 3: Terrain Analysis
    println!("\n\nExample 3: Terrain Analysis");
    println!("----------------------------\n");

    // Analyze terrain curvature
    let mut rf = RicciFlow::new(0.1, 0.0);
    let mut terrain_curvature = vec![1.0, 0.8, 0.5, 0.3, 0.6, 0.9];

    println!("Terrain curvature evolution (Ricci flow):");
    println!("  Step | Curvature values");
    println!("  ------|----------------------------------");

    println!("  {:4} | [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
             0, terrain_curvature[0], terrain_curvature[1],
             terrain_curvature[2], terrain_curvature[3],
             terrain_curvature[4], terrain_curvature[5]);

    for step in 1..=5 {
        rf.evolve(&mut terrain_curvature, 1);
        println!("  {:4} | [{:.2}, {:.2}, {:.2}, {:.2}, {:.2}, {:.2}]",
                 step, terrain_curvature[0], terrain_curvature[1],
                 terrain_curvature[2], terrain_curvature[3],
                 terrain_curvature[4], terrain_curvature[5]);
    }

    // Example 4: Obstacle Avoidance
    println!("\n\nExample 4: Obstacle Avoidance");
    println!("------------------------------\n");

    // Check if planned paths avoid obstacles
    let obstacles = vec![
        ([0.4, 0.4], 0.2),  // (x, y), radius
        ([0.7, 0.3], 0.15),
    ];

    let test_path: Vec<[f32; 2]> = vec![
        [0.0, 0.0],
        [0.5, 0.5],
        [1.0, 1.0],
    ];

    println!("Checking path for obstacles:");
    for (i, point) in test_path.iter().enumerate() {
        let mut safe = true;
        for (obs, radius) in &obstacles {
            let dx: f32 = point[0] - obs[0];
            let dy: f32 = point[1] - obs[1];
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < *radius {
                safe = false;
                println!("  Point {} ({:.2}, {:.2}) - COLLISION with obstacle at ({:.2}, {:.2})",
                         i + 1, point[0], point[1], obs[0], obs[1]);
            }
        }
        if safe {
            println!("  Point {} ({:.2}, {:.2}) - Safe", i + 1, point[0], point[1]);
        }
    }

    // Example 5: Sensor Fusion
    println!("\n\nExample 5: Sensor Fusion");
    println!("-------------------------\n");

    // Fuse multiple sensor readings
    let sensor_readings = vec![
        ([0.6, 0.8], "Camera", 0.9),
        ([0.62, 0.78], "Lidar", 0.95),
        ([0.59, 0.81], "Radar", 0.85),
    ];

    println!("Fusing sensor readings:");
    println!("  Sensor    Reading        Confidence -> Fused");
    println!("  -----------------------------------------------");

    let mut fused_x = 0.0;
    let mut fused_y = 0.0;
    let mut total_confidence = 0.0;

    for (reading, sensor, confidence) in &sensor_readings {
        let (snapped, _) = manifold.snap(*reading);
        fused_x += snapped[0] * confidence;
        fused_y += snapped[1] * confidence;
        total_confidence += confidence;

        println!(
            "  {:8}   ({:.2}, {:.2})     {:.2}      -> ({:.3}, {:.3})",
            sensor, reading[0], reading[1], confidence, snapped[0], snapped[1]
        );
    }

    fused_x /= total_confidence;
    fused_y /= total_confidence;

    let (final_fused, _) = manifold.snap([fused_x, fused_y]);
    println!("\n  Final fused position: ({:.3}, {:.3})", final_fused[0], final_fused[1]);

    // Example 6: Parallel Transport
    println!("\n\nExample 6: Parallel Transport on Manifolds");
    println!("---------------------------------------------\n");

    // Transport vectors across curved surface
    let tiles = vec![
        Tile::new(0),
        Tile::new(1),
        Tile::new(2),
    ];
    let conn = GaugeConnection::new(tiles);

    let initial_vector = [1.0, 0.0, 0.0];
    let path = vec![0, 1, 2];

    let transported = conn.parallel_transport(initial_vector, &path);

    println!("Parallel transport along path {:?}:", path);
    println!("  Initial vector: ({:.1}, {:.1}, {:.1})",
             initial_vector[0], initial_vector[1], initial_vector[2]);
    println!("  Transported:    ({:.3}, {:.3}, {:.3})",
             transported[0], transported[1], transported[2]);

    let rotation_angle = (transported[0].acos() * 180.0 / std::f32::consts::PI).abs();
    println!("  Rotation angle: {:.2}°", rotation_angle);

    println!("\n================================================");
    println!("Robotics Example Complete");
    println!("================================================");
}
