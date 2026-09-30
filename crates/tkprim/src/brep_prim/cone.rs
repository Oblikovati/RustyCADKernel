// Rust translation of BRepPrim_Cone.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrim_Cone`: the cone primitive.
use super::cylinder::standard_frame;
use super::revolution_derived::lateral_face_on;
use super::{BRepPrimOneAxis, BRepPrimRevolution, BRepPrimRevolutionDerived};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomConicalSurface, GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction2, Point2, Point3, Vector3, xoy};
use std::f64::consts::PI;

/// The Standard_DomainError of the cone constructors.
fn cone_error(detail: &str) -> Failure {
    Failure::with_message("Standard_DomainError", &format!("BRepPrim_Cone: {detail}"))
}

/// `BRepPrim_Cone`: the revolution of a line at the half angle `Angle` to the axis,
/// through the radius `Radius` at Z = 0 (docs/prim-design.md section 2).
///
/// ```
/// use rustycad_tkprim::brep_prim::{BRepPrimCone, BRepPrimOneAxis};
/// let mut cone = BRepPrimCone::from_radii(2.0, 0.0, 3.0)?;
/// assert!(!cone.has_top()?);
/// assert!(!cone.shell()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimCone {
    revolution: BRepPrimRevolution,
    my_half_angle: f64,
    my_radius: f64,
}

impl BRepPrimCone {
    /// The cone on `axes` over `vmin..vmax` before its checks and its meridian.
    fn unset(axes: Axis2, vmin: f64, vmax: f64, half_angle: f64, radius: f64) -> Self {
        Self {
            revolution: BRepPrimRevolution::without_meridian(axes, vmin, vmax),
            my_half_angle: half_angle,
            my_radius: radius,
        }
    }

    /// `BRepPrim_Cone(Angle, Position, Height, Radius)`: the STEP cone, cut at Z =
    /// `height`; Standard_DomainError for a null height, a null angle or an angle
    /// beyond PI/2 (each relative to Resolution / Height);
    /// `BRepPrimCone::new(0.5, axes, 2.0, 1.0)?`.
    pub fn new(angle: f64, position: Axis2, height: f64, radius: f64) -> Result<Self, Failure> {
        let mut cone = Self::unset(position, 0.0, 0.0, angle, radius);
        if height < precision::confusion() {
            return Err(cone_error(&format!(
                "cone with null height {height}; expected a height >= Precision::Confusion()"
            )));
        }
        if cone.my_half_angle * height < precision::confusion() {
            return Err(cone_error(&format!(
                "cone with null angle {angle} for height {height}"
            )));
        }
        if (PI / 2.0 - cone.my_half_angle) * height < precision::confusion() {
            return Err(cone_error(&format!(
                "cone with angle {angle} > PI/2 for height {height}"
            )));
        }
        cone.revolution
            .set_vmax(height / cone.my_half_angle.cos())?;
        cone.revolution.set_vmin(0.0)?;
        cone.set_meridian()?;
        Ok(cone)
    }

    /// The infinite cones: the angle check, VMin = 0 and the meridian.
    fn infinite_on(axes: Axis2, angle: f64) -> Result<Self, Failure> {
        let mut cone = Self::unset(axes, 0.0, real_last(), angle, 0.0);
        if angle < 0.0 || angle > PI / 2.0 {
            return Err(cone_error(&format!("cone with angle {angle} <0 or > PI/2")));
        }
        cone.revolution.set_vmin(0.0)?;
        cone.set_meridian()?;
        Ok(cone)
    }

    /// `BRepPrim_Cone(Angle)`: the infinite cone at the origin; Standard_DomainError for
    /// an angle outside 0..PI/2; `BRepPrimCone::infinite(0.5)?`.
    pub fn infinite(angle: f64) -> Result<Self, Failure> {
        Self::infinite_on(xoy(), angle)
    }

    /// `BRepPrim_Cone(Angle, Apex)`: the infinite cone at `apex`;
    /// `BRepPrimCone::infinite_at(0.5, apex)?`.
    pub fn infinite_at(angle: f64, apex: Point3) -> Result<Self, Failure> {
        Self::infinite_on(standard_frame(apex)?, angle)
    }

    /// `BRepPrim_Cone(Angle, Axes)`: the infinite cone on `axes`. The native constructor
    /// leaves the radius uninitialized; it is 0 here (docs/tkprim-progress.md);
    /// `BRepPrimCone::infinite_with_axes(0.5, axes)?`.
    pub fn infinite_with_axes(angle: f64, axes: Axis2) -> Result<Self, Failure> {
        Self::infinite_on(axes, angle)
    }

    /// `BRepPrim_Cone(R1, R2, H)`: the cone on the reference frame of radius `r1` at
    /// Z = 0 and `r2` at Z = `h`; Standard_DomainError for a negative or tiny radius,
    /// equal radii or a null height; `BRepPrimCone::from_radii(1.0, 0.0, 2.0)?`.
    pub fn from_radii(r1: f64, r2: f64, h: f64) -> Result<Self, Failure> {
        Self::with_axes_radii(xoy(), r1, r2, h)
    }

    /// `BRepPrim_Cone(Center, R1, R2, H)`: the same at `center`;
    /// `BRepPrimCone::at_center(center, 1.0, 0.0, 2.0)?`.
    pub fn at_center(center: Point3, r1: f64, r2: f64, h: f64) -> Result<Self, Failure> {
        Self::with_axes_radii(standard_frame(center)?, r1, r2, h)
    }

    /// `BRepPrim_Cone(Axes, R1, R2, H)`: the same on `axes`;
    /// `BRepPrimCone::with_axes_radii(axes, 1.0, 0.0, 2.0)?`.
    pub fn with_axes_radii(axes: Axis2, r1: f64, r2: f64, h: f64) -> Result<Self, Failure> {
        let mut cone = Self::unset(axes, 0.0, 0.0, 0.0, 0.0);
        cone.set_parameters(r1, r2, h)?;
        cone.set_meridian()?;
        Ok(cone)
    }

    /// `SetMeridian()`: the axis rotated by the half angle around Y and translated by
    /// the radius along X, and its 2d image through `(Radius, 0)` along
    /// `(sin, cos)` of the half angle.
    fn set_meridian(&mut self) -> Result<(), Failure> {
        let axes = self.revolution.axes();
        let mut axis = axes.axis();
        axis.rotate(
            Axis1::new(axes.location(), axes.y_direction()),
            self.my_half_angle,
        )?;
        let mut v = Vector3::from(axes.x_direction());
        v *= self.my_radius;
        axis.translate(v);
        let meridian: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(axis)).cast();
        let direction = Direction2::new(self.my_half_angle.sin(), self.my_half_angle.cos())?;
        let pmeridian: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
            Point2::new(self.my_radius, 0.0),
            direction,
        ))
        .cast();
        self.revolution.set_meridian(meridian, pmeridian);
        Ok(())
    }

    /// `SetParameters(R1, R2, H)`: the radius at Z = 0, the half angle and the meridian
    /// range of the truncated cone, after the checks of the radii and the height.
    fn set_parameters(&mut self, r1: f64, r2: f64, h: f64) -> Result<(), Failure> {
        let conf = precision::confusion();
        if (r1 != 0.0 && r1 < conf) || (r2 != 0.0 && r2 < conf) {
            return Err(cone_error(&format!(
                "cone with negative or too small radius ({r1}, {r2})"
            )));
        }
        if (r1 - r2).abs() < conf {
            return Err(cone_error(&format!(
                "cone with two identic radii ({r1}, {r2})"
            )));
        }
        if h < conf {
            return Err(cone_error(&format!(
                "cone with negative or null height {h}"
            )));
        }
        self.my_radius = r1;
        self.my_half_angle = ((r2 - r1) / h).atan();
        self.revolution.set_vmin(0.0)?;
        self.revolution
            .set_vmax((h * h + (r2 - r1) * (r2 - r1)).sqrt())
    }
}

impl BRepPrimRevolutionDerived for BRepPrimCone {
    fn revolution(&self) -> &BRepPrimRevolution {
        &self.revolution
    }

    fn revolution_mut(&mut self) -> &mut BRepPrimRevolution {
        &mut self.revolution
    }

    /// `MakeEmptyLateralFace()`: a face on the `Geom_ConicalSurface` of the axes, the
    /// half angle and the radius; Standard_ConstructionError for a radius below 0 or a
    /// half angle outside (0, PI/2).
    fn derived_lateral_face(&self) -> Result<Face, Failure> {
        let axes = self.revolution.axes().axis3();
        let surface = GeomConicalSurface::new(axes, self.my_half_angle, self.my_radius)?;
        lateral_face_on(&self.revolution, Handle::new(surface).cast())
    }
}
