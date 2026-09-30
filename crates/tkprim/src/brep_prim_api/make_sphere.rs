// Rust translation of BRepPrimAPI_MakeSphere.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeSphere`: the maker of spheres.
use super::make_one_axis::one_axis_maker;
use crate::brep_prim::{BRepPrimOneAxis, BRepPrimSphere, standard_frame};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3, Vector3, dx, dz, origin, xoy};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `BRepPrimAPI_MakeSphere`: a sphere of radius R at the origin, at `Center` or on
/// `Axis`, possibly limited in latitude (`angle1..angle2`, the spherical segment) and in
/// longitude (`0..angle`, the spherical wedge) (docs/prim-design.md section 4).
///
/// ```
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeSphere;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let mut maker = BRepPrimAPIMakeSphere::new(5.0)?;
/// assert!(!maker.shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimAPIMakeSphere {
    base: BRepBuilderAPIMakeShapeBase,
    my_sphere: BRepPrimSphere,
}

/// The settings of a sphere after its construction: the latitude range and the angle.
#[derive(Clone, Copy)]
struct SphereLimits {
    latitudes: Option<(f64, f64)>,
    angle: Option<f64>,
}

impl BRepPrimAPIMakeSphere {
    /// The maker of `sphere` with its latitude range and angle set.
    fn limited(mut sphere: BRepPrimSphere, limits: SphereLimits) -> Result<Self, Failure> {
        if let Some((angle1, angle2)) = limits.latitudes {
            sphere.set_vmin(angle1)?;
            sphere.set_vmax(angle2)?;
        }
        if let Some(angle) = limits.angle {
            sphere.set_angle(angle)?;
        }
        Ok(Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_sphere: sphere,
        })
    }

    /// `BRepPrimAPI_MakeSphere(R)`; `BRepPrimAPIMakeSphere::new(1.0)?`.
    pub fn new(r: f64) -> Result<Self, Failure> {
        Self::limited(
            BRepPrimSphere::with_axes(xoy(), r)?,
            SphereLimits {
                latitudes: None,
                angle: None,
            },
        )
    }

    /// `BRepPrimAPI_MakeSphere(R, angle)`: the spherical wedge 0..|angle|, turning around
    /// -Z for a negative angle; `BRepPrimAPIMakeSphere::with_angle(1.0, 3.0)?`.
    pub fn with_angle(r: f64, angle: f64) -> Result<Self, Failure> {
        let sign = if angle < 0.0 { -1.0 } else { 1.0 };
        let normal = Direction3::from_vector(sign * Vector3::from(dz()))?;
        let axes = Axis2::with_x_direction(origin(), normal, dx())?;
        let limits = SphereLimits {
            latitudes: None,
            angle: Some(angle.abs()),
        };
        Self::limited(BRepPrimSphere::with_axes(axes, r)?, limits)
    }

    /// `BRepPrimAPI_MakeSphere(R, angle1, angle2)`: the segment between two latitudes;
    /// `BRepPrimAPIMakeSphere::with_latitudes(1.0, 0.0, 1.0)?`.
    pub fn with_latitudes(r: f64, angle1: f64, angle2: f64) -> Result<Self, Failure> {
        let limits = SphereLimits {
            latitudes: Some((angle1, angle2)),
            angle: None,
        };
        Self::limited(BRepPrimSphere::new(r)?, limits)
    }

    /// `BRepPrimAPI_MakeSphere(R, angle1, angle2, angle3)`;
    /// `BRepPrimAPIMakeSphere::with_latitudes_angle(1.0, 0.0, 1.0, 3.0)?`.
    pub fn with_latitudes_angle(
        r: f64,
        angle1: f64,
        angle2: f64,
        angle3: f64,
    ) -> Result<Self, Failure> {
        let limits = SphereLimits {
            latitudes: Some((angle1, angle2)),
            angle: Some(angle3),
        };
        Self::limited(BRepPrimSphere::new(r)?, limits)
    }

    /// `BRepPrimAPI_MakeSphere(Center, R)`; `BRepPrimAPIMakeSphere::at_center(c, 1.0)?`.
    pub fn at_center(center: Point3, r: f64) -> Result<Self, Failure> {
        Self::on_axis_limited(standard_frame(center)?, r, None, None)
    }

    /// `BRepPrimAPI_MakeSphere(Center, R, angle)` (the angle not made positive, unlike
    /// the constructor at the origin); `BRepPrimAPIMakeSphere::at_center_angle(c, 1.0, 3.0)?`.
    pub fn at_center_angle(center: Point3, r: f64, angle: f64) -> Result<Self, Failure> {
        Self::on_axis_limited(standard_frame(center)?, r, None, Some(angle))
    }

    /// `BRepPrimAPI_MakeSphere(Center, R, angle1, angle2)`;
    /// `BRepPrimAPIMakeSphere::at_center_latitudes(c, 1.0, 0.0, 1.0)?`.
    pub fn at_center_latitudes(
        center: Point3,
        r: f64,
        angle1: f64,
        angle2: f64,
    ) -> Result<Self, Failure> {
        Self::on_axis_limited(standard_frame(center)?, r, Some((angle1, angle2)), None)
    }

    /// `BRepPrimAPI_MakeSphere(Center, R, angle1, angle2, angle3)`;
    /// `BRepPrimAPIMakeSphere::at_center_latitudes_angle(c, 1.0, 0.0, 1.0, 3.0)?`.
    pub fn at_center_latitudes_angle(
        center: Point3,
        r: f64,
        angle1: f64,
        angle2: f64,
        angle3: f64,
    ) -> Result<Self, Failure> {
        Self::on_axis_limited(
            standard_frame(center)?,
            r,
            Some((angle1, angle2)),
            Some(angle3),
        )
    }

    /// `BRepPrimAPI_MakeSphere(Axis, R)`; `BRepPrimAPIMakeSphere::with_axis(axes, 1.0)?`.
    pub fn with_axis(axis: Axis2, r: f64) -> Result<Self, Failure> {
        Self::on_axis_limited(axis, r, None, None)
    }

    /// `BRepPrimAPI_MakeSphere(Axis, R, angle)`;
    /// `BRepPrimAPIMakeSphere::with_axis_angle(axes, 1.0, 3.0)?`.
    pub fn with_axis_angle(axis: Axis2, r: f64, angle: f64) -> Result<Self, Failure> {
        Self::on_axis_limited(axis, r, None, Some(angle))
    }

    /// `BRepPrimAPI_MakeSphere(Axis, R, angle1, angle2)`;
    /// `BRepPrimAPIMakeSphere::with_axis_latitudes(axes, 1.0, 0.0, 1.0)?`.
    pub fn with_axis_latitudes(
        axis: Axis2,
        r: f64,
        angle1: f64,
        angle2: f64,
    ) -> Result<Self, Failure> {
        Self::on_axis_limited(axis, r, Some((angle1, angle2)), None)
    }

    /// `BRepPrimAPI_MakeSphere(Axis, R, angle1, angle2, angle3)`;
    /// `BRepPrimAPIMakeSphere::with_axis_latitudes_angle(axes, 1.0, 0.0, 1.0, 3.0)?`.
    pub fn with_axis_latitudes_angle(
        axis: Axis2,
        r: f64,
        angle1: f64,
        angle2: f64,
        angle3: f64,
    ) -> Result<Self, Failure> {
        Self::on_axis_limited(axis, r, Some((angle1, angle2)), Some(angle3))
    }

    /// The sphere on `axis` with its limits.
    fn on_axis_limited(
        axis: Axis2,
        r: f64,
        latitudes: Option<(f64, f64)>,
        angle: Option<f64>,
    ) -> Result<Self, Failure> {
        Self::limited(
            BRepPrimSphere::with_axes(axis, r)?,
            SphereLimits { latitudes, angle },
        )
    }

    /// `Sphere()`: the algorithm; `maker.sphere()`.
    pub fn sphere(&mut self) -> &mut BRepPrimSphere {
        &mut self.my_sphere
    }
}

one_axis_maker!(BRepPrimAPIMakeSphere, my_sphere);
