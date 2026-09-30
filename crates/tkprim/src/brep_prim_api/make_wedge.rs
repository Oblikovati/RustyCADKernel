// Rust translation of BRepPrimAPI_MakeWedge.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeWedge`: the maker of wedges.
use super::make_one_axis::{add_shell_done, reset_solid};
use crate::brep_prim::{BRepPrimWedge, standard_frame};
use rustycad_tkbrep::topods::{self, Shell, Solid};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeShapeBase};

/// `BRepPrimAPI_MakeWedge`: a box with inclined faces: the STEP right angular wedge
/// (its YMax face spans 0..ltx in X), or the wedge whose YMax face is
/// `xmin..xmax` x `zmin..zmax` (docs/prim-design.md section 4).
///
/// ```
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeWedge;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut maker = BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0)?;
/// assert!(!maker.shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimAPIMakeWedge {
    base: BRepBuilderAPIMakeShapeBase,
    my_wedge: BRepPrimWedge,
}

impl BRepPrimAPIMakeWedge {
    /// The maker of a wedge.
    fn of(my_wedge: BRepPrimWedge) -> Self {
        Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_wedge,
        }
    }

    /// `BRepPrimAPI_MakeWedge(dx, dy, dz, ltx)`: the right angular wedge from the
    /// origin; `BRepPrimAPIMakeWedge::with_ltx(10.0, 10.0, 10.0, 5.0)?`.
    pub fn with_ltx(dx: f64, dy: f64, dz: f64, ltx: f64) -> Result<Self, Failure> {
        let axes = standard_frame(Point3::new(0.0, 0.0, 0.0))?;
        Ok(Self::with_axes_ltx(axes, dx, dy, dz, ltx))
    }

    /// `BRepPrimAPI_MakeWedge(Axes, dx, dy, dz, ltx)`: the same on `axes`;
    /// `BRepPrimAPIMakeWedge::with_axes_ltx(axes, 10.0, 10.0, 10.0, 5.0)`.
    pub fn with_axes_ltx(axes: Axis2, dx: f64, dy: f64, dz: f64, ltx: f64) -> Self {
        Self::of(BRepPrimWedge::with_ltx(axes, dx, dy, dz, ltx))
    }

    /// `BRepPrimAPI_MakeWedge(dx, dy, dz, xmin, zmin, xmax, zmax)`: the wedge from the
    /// origin whose face at `dy` is `xmin..xmax` x `zmin..zmax`, given as
    /// `[xmin, zmin, xmax, zmax]`; `BRepPrimAPIMakeWedge::with_limits(20.0, 10.0, 20.0, [5.0, 5.0, 15.0, 15.0])?`.
    pub fn with_limits(dx: f64, dy: f64, dz: f64, limits: [f64; 4]) -> Result<Self, Failure> {
        let axes = standard_frame(Point3::new(0.0, 0.0, 0.0))?;
        Ok(Self::with_axes_limits(axes, dx, dy, dz, limits))
    }

    /// `BRepPrimAPI_MakeWedge(Axes, dx, dy, dz, xmin, zmin, xmax, zmax)`: the same on
    /// `axes`; `BRepPrimAPIMakeWedge::with_axes_limits(axes, 20.0, 10.0, 20.0, [5.0, 5.0, 15.0, 15.0])`.
    pub fn with_axes_limits(axes: Axis2, dx: f64, dy: f64, dz: f64, limits: [f64; 4]) -> Self {
        let [xmin, zmin, xmax, zmax] = limits;
        Self::of(BRepPrimWedge::with_limits(
            axes,
            [0.0, 0.0, 0.0, zmin, xmin],
            [dx, dy, dz, zmax, xmax],
        ))
    }

    /// `Wedge()`: the wedge; `maker.wedge()`.
    pub fn wedge(&mut self) -> &mut BRepPrimWedge {
        &mut self.my_wedge
    }

    /// `Shell()` and `operator TopoDS_Shell()`: builds, then returns the shell of the
    /// wedge; `maker.shell()?`.
    pub fn shell(&mut self) -> Result<Shell, Failure> {
        self.build(&ProgressRange::default())?;
        self.my_wedge.shell()
    }

    /// `Solid()` and `operator TopoDS_Solid()`: builds, then returns the solid;
    /// `maker.solid()?`.
    pub fn solid(&mut self) -> Result<Solid, Failure> {
        self.build(&ProgressRange::default())?;
        topods::solid(&self.base.my_shape)
    }
}

impl BRepBuilderAPIMakeShape for BRepPrimAPIMakeWedge {
    fn make_shape_base(&self) -> &BRepBuilderAPIMakeShapeBase {
        &self.base
    }

    fn make_shape_base_mut(&mut self) -> &mut BRepBuilderAPIMakeShapeBase {
        &mut self.base
    }

    /// `Build()`: a new solid holding the shell of the wedge becomes the shape.
    fn build(&mut self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
        reset_solid(&mut self.base)?;
        let shell = self.my_wedge.shell()?;
        add_shell_done(&mut self.base, &shell)?;
        Ok(())
    }
}
