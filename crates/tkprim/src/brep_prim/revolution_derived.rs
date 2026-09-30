// The derivation of BRepPrim_Cylinder, BRepPrim_Cone, BRepPrim_Sphere and BRepPrim_Torus
// from BRepPrim_Revolution (their headers) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The classes derived from `BRepPrim_Revolution`, which only override
//! `MakeEmptyLateralFace` (docs/prim-design.md section 2).
use super::{BRepPrimOneAxis, BRepPrimOneAxisBase, BRepPrimRevolution};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::Point2;

/// A class derived from `BRepPrim_Revolution`: the revolution it holds and its own
/// lateral face. Every implementor is a [`BRepPrimOneAxis`] whose other members are the
/// revolution's.
///
/// ```
/// use rustycad_tkprim::brep_prim::{BRepPrimCylinder, BRepPrimOneAxis, BRepPrimRevolutionDerived};
/// let mut cylinder = BRepPrimCylinder::from_radius_height(1.0, 2.0);
/// assert_eq!(cylinder.revolution().vmax(), 2.0);
/// assert!(!cylinder.lateral_face()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait BRepPrimRevolutionDerived {
    /// The `BRepPrim_Revolution` base.
    fn revolution(&self) -> &BRepPrimRevolution;

    /// The `BRepPrim_Revolution` base, to write.
    fn revolution_mut(&mut self) -> &mut BRepPrimRevolution;

    /// The override of `MakeEmptyLateralFace()`: a face on the elementary surface of
    /// the primitive, its normal outward; `cylinder.derived_lateral_face()?`.
    fn derived_lateral_face(&self) -> Result<Face, Failure>;
}

impl<T: BRepPrimRevolutionDerived> BRepPrimOneAxis for T {
    fn one_axis_base(&self) -> &BRepPrimOneAxisBase {
        self.revolution().one_axis_base()
    }

    fn one_axis_base_mut(&mut self) -> &mut BRepPrimOneAxisBase {
        self.revolution_mut().one_axis_base_mut()
    }

    fn make_empty_lateral_face(&self) -> Result<Face, Failure> {
        self.derived_lateral_face()
    }

    fn make_empty_meridian_edge(&self, angle: f64) -> Result<Edge, Failure> {
        self.revolution().make_empty_meridian_edge(angle)
    }

    fn set_meridian_pcurve(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        self.revolution().set_meridian_pcurve(edge, face)
    }

    fn meridian_value(&self, v: f64) -> Result<Point2, Failure> {
        self.revolution().meridian_value(v)
    }
}

/// The empty lateral face of a primitive: `MakeFace(F, S, Precision::Confusion())`
/// with the surface `surface` built by the primitive.
pub(crate) fn lateral_face_on(
    revolution: &BRepPrimRevolution,
    surface: Handle<dyn GeomSurface>,
) -> Result<Face, Failure> {
    let mut face = Face::default();
    revolution
        .one_axis_base()
        .my_builder
        .builder()
        .make_face_on_surface(&mut face, &surface, precision::confusion())?;
    Ok(face)
}
