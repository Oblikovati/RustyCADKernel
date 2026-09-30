// Rust translation of BRepPrim_Cylinder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Cylinder`: the cylinder primitive.
use super::revolution_derived::lateral_face_on;
use super::{BRepPrimOneAxis, BRepPrimRevolution, BRepPrimRevolutionDerived};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomCurve, GeomCylindricalSurface, GeomLine};
use rustycad_tkmath::gp::{
    Axis2, Direction2, Direction3, Point2, Point3, StandardDirection2, StandardDirection3, Vector3,
    xoy,
};

/// `BRepPrim_Cylinder`: the revolution of a line parallel to the axis at `Radius`
/// (docs/prim-design.md section 2). The infinite constructors span RealFirst..RealLast.
///
/// ```
/// use rustycad_tkprim::brep_prim::{BRepPrimCylinder, BRepPrimOneAxis};
/// let mut cylinder = BRepPrimCylinder::from_radius_height(2.0, 3.0);
/// assert!(cylinder.has_top()? && cylinder.has_bottom()?);
/// assert!(!cylinder.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimCylinder {
    revolution: BRepPrimRevolution,
    my_radius: f64,
}

impl BRepPrimCylinder {
    /// The cylinder of `radius` on `axes` over `vmin..vmax`, with its meridian.
    fn build(axes: Axis2, vmin: f64, vmax: f64, radius: f64) -> Self {
        let mut cylinder = Self {
            revolution: BRepPrimRevolution::without_meridian(axes, vmin, vmax),
            my_radius: radius,
        };
        cylinder.set_meridian();
        cylinder
    }

    /// `BRepPrim_Cylinder(Position, Radius, Height)`: the STEP cylinder;
    /// `BRepPrimCylinder::new(axes, 1.0, 2.0)`.
    pub fn new(position: Axis2, radius: f64, height: f64) -> Self {
        Self::build(position, 0.0, height, radius)
    }

    /// `BRepPrim_Cylinder(Radius)`: the infinite cylinder on the reference frame;
    /// `BRepPrimCylinder::infinite(1.0)`.
    pub fn infinite(radius: f64) -> Self {
        Self::build(xoy(), real_first(), real_last(), radius)
    }

    /// `BRepPrim_Cylinder(Center, Radius)`: the infinite cylinder at `center`;
    /// `BRepPrimCylinder::infinite_at(center, 1.0)?`.
    pub fn infinite_at(center: Point3, radius: f64) -> Result<Self, Failure> {
        Ok(Self::build(
            standard_frame(center)?,
            real_first(),
            real_last(),
            radius,
        ))
    }

    /// `BRepPrim_Cylinder(Axes, Radius)`: the infinite cylinder on `axes`;
    /// `BRepPrimCylinder::infinite_on(axes, 1.0)`.
    pub fn infinite_on(axes: Axis2, radius: f64) -> Self {
        Self::build(axes, real_first(), real_last(), radius)
    }

    /// `BRepPrim_Cylinder(R, H)`: the cylinder on the reference frame from 0 to `height`;
    /// `BRepPrimCylinder::from_radius_height(1.0, 2.0)`.
    pub fn from_radius_height(radius: f64, height: f64) -> Self {
        Self::build(xoy(), 0.0, height, radius)
    }

    /// `BRepPrim_Cylinder(Center, R, H)`: the cylinder at `center` from 0 to `height`;
    /// `BRepPrimCylinder::at_center(center, 1.0, 2.0)?`.
    pub fn at_center(center: Point3, radius: f64, height: f64) -> Result<Self, Failure> {
        Ok(Self::build(standard_frame(center)?, 0.0, height, radius))
    }

    /// `SetMeridian()`: the axis translated by `Radius` along X, and its 2d image
    /// through `(Radius, 0)` along +Y.
    fn set_meridian(&mut self) {
        let axes = self.revolution.axes();
        let mut v = Vector3::from(axes.x_direction());
        v.scale(self.my_radius);
        let mut axis = axes.axis();
        axis.translate(v);
        let meridian: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
        let pmeridian: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
            Point2::new(self.my_radius, 0.0),
            Direction2::from_standard(StandardDirection2::Y),
        ))
        .cast();
        self.revolution.set_meridian(meridian, pmeridian);
    }
}

/// `gp_Ax2(Center, gp_Dir(gp_Dir::D::Z), gp_Dir(gp_Dir::D::X))`: the general
/// constructor on two `gp_Dir` values (not the constexpr one on `gp_Dir::D` values),
/// whose cross products leave a signed zero in the X direction, as natively.
pub(crate) fn standard_frame(center: Point3) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(
        center,
        Direction3::from_standard(StandardDirection3::Z),
        Direction3::from_standard(StandardDirection3::X),
    )
}

impl BRepPrimRevolutionDerived for BRepPrimCylinder {
    fn revolution(&self) -> &BRepPrimRevolution {
        &self.revolution
    }

    fn revolution_mut(&mut self) -> &mut BRepPrimRevolution {
        &mut self.revolution
    }

    /// `MakeEmptyLateralFace()`: a face on the `Geom_CylindricalSurface` of the axes and
    /// the radius; Standard_ConstructionError for a negative radius.
    fn derived_lateral_face(&self) -> Result<Face, Failure> {
        let surface = GeomCylindricalSurface::new(self.revolution.axes().axis3(), self.my_radius)?;
        lateral_face_on(&self.revolution, Handle::new(surface).cast())
    }
}
