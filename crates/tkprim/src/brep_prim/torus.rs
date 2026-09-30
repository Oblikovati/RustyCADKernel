// Rust translation of BRepPrim_Torus.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Torus`: the torus primitive.
use super::cylinder::standard_frame;
use super::revolution_derived::lateral_face_on;
use super::{BRepPrimOneAxis, BRepPrimRevolution, BRepPrimRevolutionDerived};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomToroidalSurface};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Direction2, Point2, Point3, StandardDirection2, Vector3, xoy,
};
use std::f64::consts::PI;

/// `BRepPrim_Torus`: the revolution of the circle of radius `Minor` centered at
/// `Major` on the X axis, over the whole circle (0..2PI) (docs/prim-design.md
/// section 2).
///
/// ```
/// use rustycad_tkprim::brep_prim::{BRepPrimOneAxis, BRepPrimTorus};
/// let mut torus = BRepPrimTorus::new(3.0, 1.0)?;
/// assert!(torus.meridian_closed()?);
/// assert!(!torus.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimTorus {
    revolution: BRepPrimRevolution,
    my_major: f64,
    my_minor: f64,
}

impl BRepPrimTorus {
    /// `BRepPrim_Torus(Position, Major, Minor)`: the STEP torus;
    /// Standard_ConstructionError for a negative minor radius;
    /// `BRepPrimTorus::with_axes(axes, 3.0, 1.0)?`.
    pub fn with_axes(position: Axis2, major: f64, minor: f64) -> Result<Self, Failure> {
        let mut torus = Self {
            revolution: BRepPrimRevolution::without_meridian(position, 0.0, 2.0 * PI),
            my_major: major,
            my_minor: minor,
        };
        torus.set_meridian()?;
        Ok(torus)
    }

    /// `BRepPrim_Torus(Major, Minor)`: the torus centered at the origin;
    /// `BRepPrimTorus::new(3.0, 1.0)?`.
    pub fn new(major: f64, minor: f64) -> Result<Self, Failure> {
        Self::with_axes(xoy(), major, minor)
    }

    /// `BRepPrim_Torus(Center, Major, Minor)`: the torus at `center`;
    /// `BRepPrimTorus::at_center(center, 3.0, 1.0)?`.
    pub fn at_center(center: Point3, major: f64, minor: f64) -> Result<Self, Failure> {
        Self::with_axes(standard_frame(center)?, major, minor)
    }

    /// `SetMeridian()`: the circle of the XZ plane (normal -Y) translated by `Major`
    /// along X, and its 2d image centered at `(Major, 0)`.
    fn set_meridian(&mut self) -> Result<(), Failure> {
        let axes = self.revolution.axes();
        let normal = axes.y_direction().reversed();
        let mut frame = Axis2::with_x_direction(axes.location(), normal, axes.x_direction())?;
        let mut v = Vector3::from(axes.x_direction());
        v.scale(self.my_major);
        frame.translate(v);
        let meridian: Handle<dyn GeomCurve> =
            Handle::new(GeomCircle::new(frame, self.my_minor)?).cast();
        let x_axis = Axis2d::new(
            Point2::new(self.my_major, 0.0),
            Direction2::from_standard(StandardDirection2::X),
        );
        let pmeridian: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dCircle::from_axis(x_axis, self.my_minor, true)?).cast();
        self.revolution.set_meridian(meridian, pmeridian);
        Ok(())
    }
}

impl BRepPrimRevolutionDerived for BRepPrimTorus {
    fn revolution(&self) -> &BRepPrimRevolution {
        &self.revolution
    }

    fn revolution_mut(&mut self) -> &mut BRepPrimRevolution {
        &mut self.revolution
    }

    /// `MakeEmptyLateralFace()`: a face on the `Geom_ToroidalSurface` of the axes and
    /// the radii.
    fn derived_lateral_face(&self) -> Result<Face, Failure> {
        let axes = self.revolution.axes().axis3();
        let surface = GeomToroidalSurface::new(axes, self.my_major, self.my_minor)?;
        lateral_face_on(&self.revolution, Handle::new(surface).cast())
    }
}
