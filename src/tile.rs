//! Core data structures for the Constraint Theory engine
//!
//! Defines the 384-byte Tile structure and related types.

use std::mem;

/// 384-byte Tile structure - the fundamental unit of computation
///
/// Memory layout:
/// - Origin: 64 bytes (id + reference frame + rate of change)
/// - Input/Output: 16 bytes
/// - Confidence/Safety: 8 bytes
/// - Pointers: 16 bytes
/// - Tensor payload: 64 bytes (16 floats)
/// - Provenance: 4 bytes
/// - Self-play: 2 bytes
/// - Hydraulic flux: 4 bytes
/// - Constraints: 192 bytes
/// - Padding: to 384 bytes
#[repr(C, align(64))]
#[derive(Clone, Debug)]
pub struct Tile {
    /// Origin reference frame (64 bytes)
    pub origin: Origin,

    /// Input value
    pub input: u64,

    /// Output value
    pub output: u64,

    /// Confidence score (Phi cascade)
    pub confidence: f32,

    /// Safety predicate (Sigma)
    pub safety: u32,

    /// Bytecode pointer
    pub bytecode_ptr: u64,

    /// Trace pointer
    pub trace: u64,

    /// Tensor payload (16 floats for geometric data)
    pub tensor_payload: [f32; 16],

    /// Provenance head index
    pub provenance_head: u32,

    /// Self-play generation
    pub self_play_gen: u16,

    /// Hydraulic flux value
    pub hydraulic_flux: f32,

    /// Constraint block (192 bytes)
    pub constraints: ConstraintBlock,
}

impl Tile {
    /// Create a new tile with the given origin ID
    pub fn new(id: u64) -> Self {
        Self {
            origin: Origin::new(id),
            input: 0,
            output: 0,
            confidence: 0.5,
            safety: 1,
            bytecode_ptr: 0,
            trace: 0,
            tensor_payload: [0.0; 16],
            provenance_head: 0,
            self_play_gen: 0,
            hydraulic_flux: 0.0,
            constraints: ConstraintBlock::new(),
        }
    }

    /// Get the 2D vector from tensor payload (first 2 components)
    pub fn vector_2d(&self) -> [f32; 2] {
        [self.tensor_payload[0], self.tensor_payload[1]]
    }

    /// Set the 2D vector in tensor payload
    pub fn set_vector_2d(&mut self, vec: [f32; 2]) {
        self.tensor_payload[0] = vec[0];
        self.tensor_payload[1] = vec[1];
    }

    /// Reset tile to initial state
    pub fn reset(&mut self) {
        self.input = 0;
        self.output = 0;
        self.confidence = 0.5;
        self.tensor_payload = [0.0; 16];
        self.provenance_head = 0;
        self.hydraulic_flux = 0.0;
        self.constraints = ConstraintBlock::new();
    }
}

/// Origin reference frame (64 bytes aligned)
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Origin {
    /// Unique origin identifier
    pub id: u64,

    /// SO(3) rotation matrix (3x3, column-major)
    pub reference_frame: [[f32; 3]; 3],

    /// Rate of change vector
    pub rate_of_change: [f32; 3],
}

impl Origin {
    /// Create a new origin with the given ID
    pub fn new(id: u64) -> Self {
        Self {
            id,
            reference_frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            rate_of_change: [0.0; 3],
        }
    }

    /// Reset reference frame to identity
    pub fn reset(&mut self) {
        self.reference_frame = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        self.rate_of_change = [0.0; 3];
    }
}

/// Constraint block (192 bytes)
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct ConstraintBlock {
    /// Pythagorean snap target (a, b, c)
    pub snap_target: [f32; 3],

    /// Holonomy matrix (SO(3) transport)
    pub holonomy_matrix: [[f32; 3]; 3],

    /// Holonomy norm (gauge-invariant phase)
    pub holonomy_norm: f32,

    /// Ricci curvature tensor (4x4)
    pub ricci_curvature: [[f32; 4]; 4],

    /// Ricci scalar (trace of curvature)
    pub ricci_scalar: f32,

    /// Rigid cluster identifier
    pub rigid_cluster_id: u64,

    /// Percolation probability
    pub percolation_p: f32,

    /// Gluing map (tangent-edge translation)
    pub gluing_map: [f32; 2],

    /// Gluing status flag
    pub gluing_status: u32,

    /// LVQ codebook index
    pub lvq_codebook_idx: u32,

    /// Omega density
    pub omega_density: f32,

    /// Constraint tolerance
    pub constraint_tolerance: f32,

    /// Persistence hash
    pub persistence_hash: u64,
}

impl Default for ConstraintBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstraintBlock {
    /// Create a new constraint block with default values
    pub fn new() -> Self {
        Self {
            snap_target: [0.0; 3],
            holonomy_matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            holonomy_norm: 0.0,
            ricci_curvature: [[0.0; 4]; 4],
            ricci_scalar: 0.0,
            rigid_cluster_id: 0,
            percolation_p: 0.6602741, // Critical threshold
            gluing_map: [0.0; 2],
            gluing_status: 0,
            lvq_codebook_idx: 0,
            omega_density: 0.0,
            constraint_tolerance: 0.05,
            persistence_hash: 0,
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Compute holonomy norm from matrix
    pub fn compute_holonomy_norm(&mut self) {
        // ||H - I||_F / (2 * sqrt(p))
        let mut sum = 0.0f32;
        for i in 0..3 {
            for j in 0..3 {
                let val = self.holonomy_matrix[i][j] - if i == j { 1.0 } else { 0.0 };
                sum += val * val;
            }
        }
        self.holonomy_norm = sum.sqrt() / (2.0 * self.percolation_p.sqrt());
    }
}

// Compile-time size checks
const _: () = assert!(mem::size_of::<Tile>() == 384, "Tile must be 384 bytes");
const _: () = assert!(mem::size_of::<Origin>() == 64, "Origin must be 64 bytes");
const _: () = assert!(
    mem::size_of::<ConstraintBlock>() == 192,
    "ConstraintBlock must be 192 bytes"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_size() {
        assert_eq!(mem::size_of::<Tile>(), 384);
    }

    #[test]
    fn test_origin_size() {
        assert_eq!(mem::size_of::<Origin>(), 64);
    }

    #[test]
    fn test_constraint_block_size() {
        assert_eq!(mem::size_of::<ConstraintBlock>(), 192);
    }

    #[test]
    fn test_tile_creation() {
        let tile = Tile::new(42);
        assert_eq!(tile.origin.id, 42);
        assert_eq!(tile.confidence, 0.5);
        assert_eq!(tile.safety, 1);
    }

    #[test]
    fn test_vector_2d() {
        let mut tile = Tile::new(0);
        tile.set_vector_2d([0.6, 0.8]);
        let vec = tile.vector_2d();
        assert_eq!(vec[0], 0.6);
        assert_eq!(vec[1], 0.8);
    }

    #[test]
    fn test_holonomy_norm() {
        let mut cb = ConstraintBlock::new();
        cb.holonomy_matrix = [[1.0, 0.1, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        cb.compute_holonomy_norm();
        assert!(cb.holonomy_norm > 0.0);
    }
}
