// Translation of BRepOffset_SimpleOffset at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod callbacks;
mod preparation;

use super::{OffsetEdgeGeometry, OffsetVertexGeometry, SimpleOffsetFaceData};
use rustycad_tkbrep::brep_tools::BRepToolsModification;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, StandardTransient};

/// Precompute the geometry used by OCCT's simple-offset modification callbacks.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkoffset::brep_offset::BRepOffsetSimpleOffset;
/// let modification = BRepOffsetSimpleOffset::new(&Shape::default(), 1.0, 1e-7).unwrap();
/// ```
pub struct BRepOffsetSimpleOffset {
    parent: StandardTransient,
    faces: DataMap<Face, SimpleOffsetFaceData>,
    edges: DataMap<Edge, OffsetEdgeGeometry>,
    vertices: DataMap<Vertex, OffsetVertexGeometry>,
}

impl_transient!(
    BRepOffsetSimpleOffset, "BRepOffset_SimpleOffset", dyn BRepToolsModification,
    parent, interfaces: [dyn BRepToolsModification]
);

impl BRepOffsetSimpleOffset {
    /// Prepare faces, then edges, then vertices, preserving native ancestor order.
    /// `BRepOffsetSimpleOffset::new(&shape, distance, tolerance)?`.
    pub fn new(shape: &Shape, offset: f64, tolerance: f64) -> Result<Self, Failure> {
        let faces = preparation::offset_faces(shape, offset, tolerance)?;
        let edges = preparation::offset_edges(shape, &faces)?;
        let vertices = preparation::offset_vertices(shape, &edges)?;
        Ok(Self {
            parent: StandardTransient::default(),
            faces,
            edges,
            vertices,
        })
    }
}
