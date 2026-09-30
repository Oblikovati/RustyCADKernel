//! Cell geometry separated from callback failure policy without changing existing inspectors.
use super::CellInspector;
/// Static spatial policy, also supplied by every `CellInspector`; `CellFilter::<Geometry>::new(1.0)`.
pub trait CellGeometry {
    /// Stored candidate payload.
    type CellTarget: Clone;
    /// Spatial query point.
    type CellPoint;
    /// Number of spatial coordinates.
    const CELL_DIMENSION: usize;
    /// Extract a zero-based coordinate; `Geometry::cell_coordinate(0,&point)`.
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64;
    /// Match candidate copies for removal; `Geometry::targets_equal(&first,&second)`.
    fn targets_equal(first: &Self::CellTarget, second: &Self::CellTarget) -> bool;
}
impl<I: CellInspector> CellGeometry for I {
    type CellTarget = I::Target;
    type CellPoint = I::Point;
    const CELL_DIMENSION: usize = I::DIMENSION;
    fn cell_coordinate(axis: usize, point: &Self::CellPoint) -> f64 {
        I::coord(axis, point)
    }
    fn targets_equal(first: &Self::CellTarget, second: &Self::CellTarget) -> bool {
        I::is_equal(first, second)
    }
}
