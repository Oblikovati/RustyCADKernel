// Rust translation of BRepPrim_Wedge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Wedge`: the generic wedge with the default builder.
use super::{BRepPrimBuilder, BRepPrimGWedge};
use rustycad_tkmath::gp::Axis2;
use std::ops::{Deref, DerefMut};

/// `BRepPrim_Wedge`: a [`BRepPrimGWedge`] built with a default `BRepPrim_Builder`; it
/// dereferences to the wedge, as the native class derives from it
/// (docs/prim-design.md section 2).
///
/// ```
/// use rustycad_tkmath::gp::Axis2;
/// use rustycad_tkprim::brep_prim::BRepPrimWedge;
/// let mut wedge = BRepPrimWedge::with_ltx(Axis2::default(), 10.0, 10.0, 10.0, 5.0);
/// assert_eq!(wedge.get_x2max(), 5.0);
/// assert!(!wedge.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct BRepPrimWedge(BRepPrimGWedge);

impl Deref for BRepPrimWedge {
    type Target = BRepPrimGWedge;
    fn deref(&self) -> &BRepPrimGWedge {
        &self.0
    }
}

impl DerefMut for BRepPrimWedge {
    fn deref_mut(&mut self) -> &mut BRepPrimGWedge {
        &mut self.0
    }
}

impl BRepPrimWedge {
    /// `BRepPrim_Wedge()`: the default wedge; `BRepPrimWedge::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepPrim_Wedge(Axes, dx, dy, dz)`: the box 0..dx, 0..dy, 0..dz;
    /// `BRepPrimWedge::with_box(axes, 1.0, 2.0, 3.0)`.
    pub fn with_box(axes: Axis2, dx: f64, dy: f64, dz: f64) -> Self {
        Self(BRepPrimGWedge::with_box(
            BRepPrimBuilder::new(),
            axes,
            dx,
            dy,
            dz,
        ))
    }

    /// `BRepPrim_Wedge(Axes, dx, dy, dz, ltx)`: the right angular wedge;
    /// `BRepPrimWedge::with_ltx(axes, 1.0, 2.0, 3.0, 0.5)`.
    pub fn with_ltx(axes: Axis2, dx: f64, dy: f64, dz: f64, ltx: f64) -> Self {
        Self(BRepPrimGWedge::with_ltx(
            BRepPrimBuilder::new(),
            axes,
            dx,
            dy,
            dz,
            ltx,
        ))
    }

    /// `BRepPrim_Wedge(Axes, xmin, ymin, zmin, z2min, x2min, xmax, ymax, zmax, z2max,
    /// x2max)`, the minima as `[xmin, ymin, zmin, z2min, x2min]` and the maxima as
    /// `[xmax, ymax, zmax, z2max, x2max]`; `BRepPrimWedge::with_limits(axes, [0.0; 5], [1.0; 5])`.
    pub fn with_limits(axes: Axis2, mins: [f64; 5], maxs: [f64; 5]) -> Self {
        Self(BRepPrimGWedge::with_limits(
            BRepPrimBuilder::new(),
            axes,
            mins,
            maxs,
        ))
    }
}
