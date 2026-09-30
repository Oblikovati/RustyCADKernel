// Rust translation of BRepPrimAPI_MakeCone.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeCone`: the maker of cones.
use super::make_one_axis::one_axis_maker;
use crate::brep_prim::{BRepPrimCone, BRepPrimOneAxis};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, xoy};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `BRepPrimAPI_MakeCone`: a cone of radius R1 at Z = 0 and R2 at Z = H (a sharp cone
/// when one is null) on the reference frame or `Axes`, or the portion of it between the
/// angles 0 and `angle`; Standard_DomainError for equal or negative radii or a null
/// height (docs/prim-design.md section 4).
///
/// ```
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCone;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut maker = BRepPrimAPIMakeCone::new(5.0, 2.0, 10.0)?;
/// assert!(!maker.shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimAPIMakeCone {
    base: BRepBuilderAPIMakeShapeBase,
    my_cone: BRepPrimCone,
}

impl BRepPrimAPIMakeCone {
    fn of(my_cone: BRepPrimCone) -> Self {
        Self { base: BRepBuilderAPIMakeShapeBase::new(), my_cone }
    }

    /// `BRepPrimAPI_MakeCone(R1, R2, H)`; `BRepPrimAPIMakeCone::new(1.0, 0.0, 2.0)?`.
    pub fn new(r1: f64, r2: f64, h: f64) -> Result<Self, Failure> {
        Ok(Self::of(BRepPrimCone::with_axes_radii(xoy(), r1, r2, h)?))
    }

    /// `BRepPrimAPI_MakeCone(R1, R2, H, angle)`: the portion 0..`angle`;
    /// `BRepPrimAPIMakeCone::with_angle(1.0, 0.0, 2.0, 3.0)?`.
    pub fn with_angle(r1: f64, r2: f64, h: f64, angle: f64) -> Result<Self, Failure> {
        let mut cone = BRepPrimCone::from_radii(r1, r2, h)?;
        cone.set_angle(angle)?;
        Ok(Self::of(cone))
    }

    /// `BRepPrimAPI_MakeCone(Axes, R1, R2, H)`;
    /// `BRepPrimAPIMakeCone::with_axes(axes, 1.0, 0.0, 2.0)?`.
    pub fn with_axes(axes: Axis2, r1: f64, r2: f64, h: f64) -> Result<Self, Failure> {
        Ok(Self::of(BRepPrimCone::with_axes_radii(axes, r1, r2, h)?))
    }

    /// `BRepPrimAPI_MakeCone(Axes, R1, R2, H, angle)`;
    /// `BRepPrimAPIMakeCone::with_axes_angle(axes, 1.0, 0.0, 2.0, 3.0)?`.
    pub fn with_axes_angle(axes: Axis2, r1: f64, r2: f64, h: f64, angle: f64) -> Result<Self, Failure> {
        let mut cone = BRepPrimCone::with_axes_radii(axes, r1, r2, h)?;
        cone.set_angle(angle)?;
        Ok(Self::of(cone))
    }

    /// `Cone()`: the algorithm; `maker.cone()`.
    pub fn cone(&mut self) -> &mut BRepPrimCone {
        &mut self.my_cone
    }
}

one_axis_maker!(BRepPrimAPIMakeCone, my_cone);
