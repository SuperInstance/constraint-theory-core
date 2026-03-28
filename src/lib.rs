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
//! # Core Concepts
//!
//! ## Pythagorean Manifold
//!
//! The `PythagoreanManifold` is the primary data structure, representing a discrete
//! set of exact Pythagorean coordinates on the unit circle. It enables deterministic
//! projection of continuous vectors to exact rational ratios.
//!
//! ## Hidden Dimensions Formula
//!
//! The number of hidden dimensions for precision ε:
//! ```text
//! k = ⌈log₂(1/ε)⌉
//! ```
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
//!
//! # Error Handling
//!
//! For consensus-critical applications, validate inputs before snapping:
//!
//! ```
//! use constraint_theory_core::PythagoreanManifold;
//!
//! let manifold = PythagoreanManifold::new(200);
//!
//! // Validate input for consensus-critical code
//! if let Err(reason) = manifold.validate_input([f32::NAN, 0.0]) {
//!     println!("Invalid input: {}", reason);
//! }
//! ```
//!
//! # Feature Flags
//!
//! - `simd`: Enable SIMD optimizations (enabled automatically on supported platforms)

#![deny(missing_docs)]
#![warn(unused_extern_crates)]
#![warn(clippy::all)]

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

/// Core error type for constraint theory operations
///
/// This enum represents all possible errors that can occur during
/// constraint theory operations. All fallible operations return `CTResult<T>`.
///
/// # Example
///
/// ```
/// use constraint_theory_core::{CTErr, CTResult};
///
/// fn process_input(x: f32, y: f32) -> CTResult<([f32; 2], f32)> {
///     if !x.is_finite() || !y.is_finite() {
///         return Err(CTErr::InvalidDimension);
///     }
///     Ok(([x, y], 0.0))
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CTErr {
    /// Invalid input dimension - expected 2D vector
    InvalidDimension,
    /// Manifold not initialized - call `new()` first
    ManifoldEmpty,
    /// Numerical instability detected - input may contain NaN or Infinity
    NumericalInstability,
    /// Input vector is zero length - cannot normalize
    ZeroVector,
    /// Input contains NaN values
    NaNInput,
    /// Input contains Infinity values
    InfinityInput,
    /// Batch size mismatch - input and output buffers have different lengths
    BufferSizeMismatch,
}

impl std::fmt::Display for CTErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimension => write!(f, "Invalid input dimension - expected 2D vector"),
            Self::ManifoldEmpty => write!(f, "Manifold is empty - initialize with new()"),
            Self::NumericalInstability => write!(f, "Numerical instability detected"),
            Self::ZeroVector => write!(f, "Input vector is zero length - cannot normalize"),
            Self::NaNInput => write!(f, "Input contains NaN values"),
            Self::InfinityInput => write!(f, "Input contains Infinity values"),
            Self::BufferSizeMismatch => write!(f, "Input and output buffers have different lengths"),
        }
    }
}

impl std::error::Error for CTErr {}

/// Result type for constraint theory operations
///
/// This is a type alias for `Result<T, CTErr>` used throughout the library.
///
/// # Example
///
/// ```
/// use constraint_theory_core::{CTResult, CTErr};
///
/// fn fallible_operation() -> CTResult<f32> {
///     Ok(1.0)
/// }
/// ```
pub type CTResult<T> = Result<T, CTErr>;

/// Version information string
///
/// Matches the crate version from Cargo.toml.
///
/// # Example
///
/// ```
/// use constraint_theory_core::VERSION;
/// println!("Constraint Theory Core v{}", VERSION);
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crate version as semver components
pub const VERSION_MAJOR: usize = 1;
pub const VERSION_MINOR: usize = 0;
pub const VERSION_PATCH: usize = 1;

/// Hidden dimensions required for target precision
///
/// Computes k = ⌈log₂(1/ε)⌉
///
/// # Arguments
///
/// * `epsilon` - Target precision (must be > 0)
///
/// # Returns
///
/// Number of hidden dimensions needed
///
/// # Example
///
/// ```
/// use constraint_theory_core::hidden_dimensions;
///
/// let k = hidden_dimensions(0.01);
/// assert_eq!(k, 7);
/// ```
pub fn hidden_dimensions(epsilon: f32) -> usize {
    if epsilon <= 0.0 {
        return usize::MAX;
    }
    (1.0 / epsilon).log2().ceil() as usize
}

/// Compute maximum angular error for a manifold
///
/// # Arguments
///
/// * `state_count` - Number of valid states in the manifold
///
/// # Returns
///
/// Maximum angular deviation in radians
///
/// # Example
///
/// ```
/// use constraint_theory_core::max_angular_error_for_states;
///
/// let error = max_angular_error_for_states(1000);
/// assert!(error < 0.01);  // ~0.36 degrees
/// ```
pub fn max_angular_error_for_states(state_count: usize) -> f32 {
    if state_count == 0 {
        return std::f32::consts::PI;
    }
    std::f32::consts::PI / state_count as f32
}

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
        assert_eq!(VERSION_MAJOR, 1);
        assert_eq!(VERSION_MINOR, 0);
        assert_eq!(VERSION_PATCH, 1);
    }

    #[test]
    fn test_hidden_dimensions() {
        assert_eq!(hidden_dimensions(0.1), 4);
        assert_eq!(hidden_dimensions(0.01), 7);
        assert_eq!(hidden_dimensions(0.001), 10);
        assert_eq!(hidden_dimensions(0.0001), 14);
    }

    #[test]
    fn test_max_angular_error() {
        let error = max_angular_error_for_states(1000);
        assert!(error > 0.0);
        assert!(error < 0.01); // ~0.36 degrees
    }

    #[test]
    fn test_cterr_display() {
        assert!(!CTErr::InvalidDimension.to_string().is_empty());
        assert!(!CTErr::ManifoldEmpty.to_string().is_empty());
        assert!(!CTErr::NumericalInstability.to_string().is_empty());
        assert!(!CTErr::ZeroVector.to_string().is_empty());
        assert!(!CTErr::NaNInput.to_string().is_empty());
        assert!(!CTErr::InfinityInput.to_string().is_empty());
        assert!(!CTErr::BufferSizeMismatch.to_string().is_empty());
    }
}
