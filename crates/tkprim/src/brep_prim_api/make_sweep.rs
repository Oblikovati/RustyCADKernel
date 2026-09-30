// Rust translation of BRepPrimAPI_MakeSweep.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeSweep`: the root of the sweep makers.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// `BRepPrimAPI_MakeSweep` (docs/prim-design.md section 4): a maker of a sweep, which
/// gives the bottom and the top of the sweep (the two pure virtual members).
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkmath::gp::{Point3, Vector3};
/// use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakePrism, BRepPrimAPIMakeSweep};
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;
/// let vertex: Shape = BRepBuilderAPIMakeVertex::new(Point3::new(0.0, 0.0, 0.0))?.vertex()?.into();
/// let mut maker = BRepPrimAPIMakePrism::new(&vertex, Vector3::new(0.0, 0.0, 1.0), false, true)?;
/// assert!(!maker.last_shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait BRepPrimAPIMakeSweep: BRepBuilderAPIMakeShape {
    /// `FirstShape()`: the bottom of the sweep; `maker.first_shape()?`.
    fn first_shape(&mut self) -> Result<Shape, Failure>;

    /// `LastShape()`: the top of the sweep; `maker.last_shape()?`.
    fn last_shape(&mut self) -> Result<Shape, Failure>;
}
