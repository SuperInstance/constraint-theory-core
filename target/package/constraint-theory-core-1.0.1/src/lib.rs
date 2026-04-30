//! Constraint Theory Core - High-Performance Geometric Engine
//!
//! This crate provides the core mathematical operations for the SuperInstance
//! Constraint Theory system, including:
//! - Pythagorean snapping (Phi-Folding Operator)
//! - Ricci flow evolution
//! - Holonomy transport
//! - Sheaf cohomology
//! - Rigidity percolation
//!
//! # Performance
//!
//! - Target: <100ns per tile operation
//! - SIMD: AVX2/AVX-512 for x86_64, NEON for ARM
//! - Memory: Zero-allocation hot paths
//!
//! # Example
//!
//! ```
//! use constraint_theory_core::{PythagoreanManifold, snap};
//!
//! let manifold = PythagoreanManifold::new(200);
//! let vec = [0.6f32, 0.8];
//! let (snapped, noise) = snap(&manifold, vec);
//! assert!(noise < 0.01);
//! ```
//!
//! # SIMD Batch Processing
//!
//! For high-throughput applications, use SIMD batch processing:
//!
//! ```
//! use constraint_theory_core::PythagoreanManifold;
//!
//! let manifold = PythagoreanManifold::new(200);
//! let vectors = vec![[0.6, 0.8], [0.8, 0.6], [0.1, 0.99]];
//! let results = manifold.snap_batch_simd(&vectors);
//!
//! for (snapped, noise) in results {
//!     println!("Snapped: {:?}, Noise: {}", snapped, noise);
//! }
//! ```

#![deny(missing_docs)]
#![warn(unused_extern_crates)]

pub mod cohomology;
pub mod curvature;
pub mod gauge;
pub mod kdtree;
pub mod manifold;
pub mod percolation;
pub mod simd;
pub mod tile;

#[cfg(test)]
mod edge_case_tests;

// Re-export key types
pub use curvature::{ricci_flow_step, RicciFlow};
pub use manifold::{snap, PythagoreanManifold, PythagoreanTriple};
pub use percolation::{FastPercolation, RigidityResult};
pub use tile::{ConstraintBlock, Origin, Tile};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Core error type
#[derive(Debug, Clone, Copy)]
pub enum CTErr {
    /// Invalid input dimension
    InvalidDimension,
    /// Manifold not initialized
    ManifoldEmpty,
    /// Numerical instability detected
    NumericalInstability,
}

impl std::fmt::Display for CTErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimension => write!(f, "Invalid input dimension"),
            Self::ManifoldEmpty => write!(f, "Manifold is empty"),
            Self::NumericalInstability => write!(f, "Numerical instability detected"),
        }
    }
}

impl std::error::Error for CTErr {}

/// Result type for constraint theory operations
pub type CTResult<T> = Result<T, CTErr>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_accuracy() {
        let manifold = PythagoreanManifold::new(200);

        // Test 3-4-5 triple
        let vec = [0.6f32, 0.8];
        let (snapped, noise) = snap(&manifold, vec);

        assert!(noise < 0.001, "Noise should be near zero for exact triple");
        assert!((snapped[0] - 0.6).abs() < 0.01);
        assert!((snapped[1] - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
