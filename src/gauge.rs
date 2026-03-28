//! Gauge Connection and Parallel Transport
//!
//! This module implements gauge connections for parallel transport
//! of vectors across tile networks using holonomy matrices.

use crate::tile::Tile;

/// Gauge connection for parallel transport across tiles
///
/// Maintains a collection of tiles with holonomy matrices that
/// define how vectors transform when transported between tiles.
pub struct GaugeConnection {
    /// Collection of tiles with holonomy information
    tiles: Vec<Tile>,
}

impl GaugeConnection {
    /// Create a new gauge connection from a collection of tiles
    ///
    /// # Arguments
    ///
    /// * `tiles` - Vector of tiles defining the connection
    ///
    /// # Example
    ///
    /// ```rust
    /// use constraint_theory_core::gauge::GaugeConnection;
    /// use constraint_theory_core::tile::Tile;
    ///
    /// let tiles = vec![Tile::new(0), Tile::new(1)];
    /// let conn = GaugeConnection::new(tiles);
    /// ```
    pub fn new(tiles: Vec<Tile>) -> Self {
        Self { tiles }
    }

    /// Transport a vector along a path through tiles
    ///
    /// # Arguments
    ///
    /// * `vector` - 3D vector to transport
    /// * `path` - Sequence of tile indices defining the path
    ///
    /// # Returns
    ///
    /// Transformed vector after parallel transport
    ///
    /// # Example
    ///
    /// ```rust
    /// use constraint_theory_core::gauge::GaugeConnection;
    /// use constraint_theory_core::tile::Tile;
    ///
    /// let tiles = vec![Tile::new(0), Tile::new(1)];
    /// let conn = GaugeConnection::new(tiles);
    /// let result = conn.parallel_transport([1.0, 0.0, 0.0], &[0, 1]);
    /// ```
    pub fn parallel_transport(&self, vector: [f32; 3], path: &[usize]) -> [f32; 3] {
        let mut transported = vector;

        for i in 0..path.len().saturating_sub(1) {
            let u = path[i];
            let v = path[i + 1];

            if u < self.tiles.len() && v < self.tiles.len() {
                let h = &self.tiles[u].constraints.holonomy_matrix;

                let x =
                    h[0][0] * transported[0] + h[0][1] * transported[1] + h[0][2] * transported[2];
                let y =
                    h[1][0] * transported[0] + h[1][1] * transported[1] + h[1][2] * transported[2];
                let z =
                    h[2][0] * transported[0] + h[2][1] * transported[1] + h[2][2] * transported[2];

                transported = [x, y, z];
            }
        }

        transported
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tile::Tile;

    #[test]
    fn test_parallel_transport() {
        let tiles = vec![Tile::new(0), Tile::new(1)];
        let conn = GaugeConnection::new(tiles);

        let result = conn.parallel_transport([1.0, 0.0, 0.0], &[0, 1]);
        // Default holonomy is identity, so vector should be unchanged
        assert!((result[0] - 1.0).abs() < 0.01);
    }
}
