// Rust translation of BRepBuilderAPI_MakeVertex.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepBuilderAPI_MakeVertex`: a vertex at a point, of the default precision.
use super::make_shape::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};
use super::wrap::adopt;
use crate::brep_lib::BRepLibMakeVertex;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Point3;

/// `BRepBuilderAPI_MakeVertex` (docs/topalgo-design.md section 10): wraps
/// `BRepLib_MakeVertex`, so the vertex has the tolerance `BRepBuilderAPI::Precision()`.
///
/// ```
/// use rustycad_tkmath::gp::Point3;
/// use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeVertex};
/// let mut maker = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0))?;
/// assert!(maker.is_done());
/// let vertex = maker.vertex()?.into_shape();
/// assert!(maker.shape()?.is_same(&vertex));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepBuilderAPIMakeVertex {
    base: BRepBuilderAPIMakeShapeBase,
    my_make_vertex: BRepLibMakeVertex,
}

impl BRepBuilderAPIMakeShape for BRepBuilderAPIMakeVertex {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }
}

impl BRepBuilderAPIMakeVertex {
    /// `BRepBuilderAPI_MakeVertex(P)`: the vertex of `BRepLib_MakeVertex(P)`;
    /// `BRepBuilderAPIMakeVertex::new(point)?`.
    pub fn new(point: Point3) -> Result<Self, Failure> {
        let mut maker = Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_make_vertex: BRepLibMakeVertex::new(point)?,
        };
        adopt(&mut maker.base, &maker.my_make_vertex)?;
        Ok(maker)
    }

    /// `Vertex()` and `operator TopoDS_Vertex()`: the vertex of the BRepLib maker;
    /// `maker.vertex()?`.
    pub fn vertex(&self) -> Result<Vertex, Failure> {
        self.my_make_vertex.vertex()
    }
}
