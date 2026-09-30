//! Native lightweight Delaunay elements and hash/equality semantics.
mod edge;
mod pair;
mod triangle;
mod vertex;
pub use edge::{BRepMeshEdge, BRepMeshOrientedEdge};
pub use pair::PairOfIndex;
pub use triangle::BRepMeshTriangle;
pub use vertex::BRepMeshVertex;
/// Native element constraint flags; `DegreeOfFreedom::Deleted as i32 == 6`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum DegreeOfFreedom {
    /// Unconstrained element.
    #[default]
    Free,
    /// Interior volume element.
    InVolume,
    /// Surface-constrained element.
    OnSurface,
    /// Curve-constrained element.
    OnCurve,
    /// Fixed element.
    Fixed,
    /// Boundary element.
    Frontier,
    /// Deleted element, unequal even to itself.
    Deleted,
}
