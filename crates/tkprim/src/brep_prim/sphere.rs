// Rust translation of BRepPrim_Sphere.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Sphere`: the sphere primitive.
use super::cylinder::standard_frame;
use super::revolution_derived::lateral_face_on;
use super::{BRepPrimOneAxis, BRepPrimRevolution, BRepPrimRevolutionDerived};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomSphericalSurface};
use rustycad_tkmath::gp::{Axis2, Axis2d, Direction2, Point2, Point3, StandardDirection2, xoy};
use std::f64::consts::PI;

/// `PMIN`, `PMAX`: the parameters of the poles on the meridian.
const PMIN: f64 = -0.5 * PI;
const PMAX: f64 = 0.5 * PI;

/// `BRepPrim_Sphere`: the revolution of a half circle of `Radius` from the south pole
/// (-PI/2) to the north pole (PI/2); the meridian edges are trimmed in 3PI/2..5PI/2 by a
/// meridian offset of 2PI (docs/prim-design.md section 2).
///
/// ```
/// use rustycad_tkprim::brep_prim::{BRepPrimOneAxis, BRepPrimSphere};
/// let mut sphere = BRepPrimSphere::new(2.0)?;
/// assert!(!sphere.has_top()? && !sphere.has_sides());
/// assert!(!sphere.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimSphere {
    revolution: BRepPrimRevolution,
    my_radius: f64,
}

impl BRepPrimSphere {
    /// `BRepPrim_Sphere(Radius)`: the sphere at the origin on the reference axes;
    /// Standard_ConstructionError for a negative radius; `BRepPrimSphere::new(1.0)?`.
    pub fn new(radius: f64) -> Result<Self, Failure> {
        Self::with_axes(xoy(), radius)
    }

    /// `BRepPrim_Sphere(Center, Radius)`: the STEP sphere;
    /// `BRepPrimSphere::at_center(center, 1.0)?`.
    pub fn at_center(center: Point3, radius: f64) -> Result<Self, Failure> {
        Self::with_axes(standard_frame(center)?, radius)
    }

    /// `BRepPrim_Sphere(Axes, Radius)`: the sphere on `axes`;
    /// `BRepPrimSphere::with_axes(axes, 1.0)?`.
    pub fn with_axes(axes: Axis2, radius: f64) -> Result<Self, Failure> {
        let mut sphere = Self {
            revolution: BRepPrimRevolution::without_meridian(axes, PMIN, PMAX),
            my_radius: radius,
        };
        sphere.set_meridian()?;
        Ok(sphere)
    }

    /// `SetMeridian()`: the meridian offset 2PI, the circle of the XZ plane (normal
    /// -Y) and its 2d image centered at the origin.
    fn set_meridian(&mut self) -> Result<(), Failure> {
        self.revolution.set_meridian_offset(2.0 * PI);
        let axes = self.revolution.axes();
        let normal = axes.y_direction().reversed();
        let frame = Axis2::with_x_direction(axes.location(), normal, axes.x_direction())?;
        let meridian: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(frame, self.my_radius)?).cast();
        let x_axis = Axis2d::new(
            Point2::new(0.0, 0.0),
            Direction2::from_standard(StandardDirection2::X),
        );
        let pmeridian: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dCircle::from_axis(x_axis, self.my_radius, true)?).cast();
        self.revolution.set_meridian(meridian, pmeridian);
        Ok(())
    }
}

impl BRepPrimRevolutionDerived for BRepPrimSphere {
    fn revolution(&self) -> &BRepPrimRevolution {
        &self.revolution
    }

    fn revolution_mut(&mut self) -> &mut BRepPrimRevolution {
        &mut self.revolution
    }

    /// `MakeEmptyLateralFace()`: a face on the `Geom_SphericalSurface` of the axes and
    /// the radius.
    fn derived_lateral_face(&self) -> Result<Face, Failure> {
        let surface = GeomSphericalSurface::new(self.revolution.axes().axis3(), self.my_radius)?;
        lateral_face_on(&self.revolution, Handle::new(surface).cast())
    }
}
