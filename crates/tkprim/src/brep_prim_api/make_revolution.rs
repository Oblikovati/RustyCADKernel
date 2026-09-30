// Rust translation of BRepPrimAPI_MakeRevolution.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepPrimAPI_MakeRevolution`: the maker of bodies of revolution of a curve.
use super::make_one_axis::one_axis_maker;
use crate::brep_prim::{BRepPrimOneAxis, BRepPrimRevolution, standard_frame};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::{Axis2, Point3, dx, dy, origin, xoy};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShapeBase;

/// `Project(M)`: the meridian projected on the plane of the frame `gp_Ax2(gp::Origin(),
/// -gp::DY(), gp::DX())` (the XZ plane, normal -Y), which is the 2d meridian.
fn project(meridian: &Handle<dyn GeomCurve>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let frame = Axis2::with_x_direction(origin(), dy().reversed(), dx())?;
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame.axis3())).cast();
    geom_proj_lib::curve2d(meridian, &plane)
}

/// The parameter range of the meridian (`FirstParameter`, `LastParameter`).
fn meridian_range(meridian: &Handle<dyn GeomCurve>) -> Result<(f64, f64), Failure> {
    let curve = meridian.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "BRepPrimAPI_MakeRevolution: the meridian is null; expected a curve",
        )
    })?;
    Ok((curve.first_parameter(), curve.last_parameter()))
}

/// `BRepPrimAPI_MakeRevolution`: the body of revolution of a meridian curve in the XZ
/// plane of the reference frame or of `Axes`, over its whole range or `VMin..VMax`,
/// possibly limited to the angles `0..angle`; the 2d meridian is its projection on the XZ
/// plane (docs/prim-design.md section 4).
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
/// use rustycad_tkmath::gp::{Direction3, Point3};
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeRevolution;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(Point3::new(1.0, 0.0, 0.0), Direction3::new(0.0, 0.0, 1.0)?)).cast();
/// let mut maker = BRepPrimAPIMakeRevolution::with_range(&line, 0.0, 2.0)?;
/// assert!(!maker.shape()?.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone)]
pub struct BRepPrimAPIMakeRevolution {
    base: BRepBuilderAPIMakeShapeBase,
    my_revolution: BRepPrimRevolution,
}

impl BRepPrimAPIMakeRevolution {
    /// The maker of the revolution of `meridian` on `axes` over `range`, limited to
    /// `angle` when given.
    fn build_on(
        axes: Axis2,
        range: (f64, f64),
        meridian: &Handle<dyn GeomCurve>,
        angle: Option<f64>,
    ) -> Result<Self, Failure> {
        let mut revolution =
            BRepPrimRevolution::new(axes, range.0, range.1, meridian.clone(), project(meridian)?);
        if let Some(angle) = angle {
            revolution.set_angle(angle)?;
        }
        Ok(Self {
            base: BRepBuilderAPIMakeShapeBase::new(),
            my_revolution: revolution,
        })
    }

    /// `BRepPrimAPI_MakeRevolution(Meridian)`: on the reference frame (`gp::XOY()`), over
    /// the whole meridian; `BRepPrimAPIMakeRevolution::new(&m)?`.
    pub fn new(meridian: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        Self::build_on(xoy(), meridian_range(meridian)?, meridian, None)
    }

    /// `BRepPrimAPI_MakeRevolution(Meridian, angle)`: on the frame at the origin built
    /// from two `gp_Dir` (not `gp::XOY()`); `BRepPrimAPIMakeRevolution::with_angle(&m, 3.0)?`.
    pub fn with_angle(meridian: &Handle<dyn GeomCurve>, angle: f64) -> Result<Self, Failure> {
        let axes = standard_frame(Point3::new(0.0, 0.0, 0.0))?;
        Self::build_on(axes, meridian_range(meridian)?, meridian, Some(angle))
    }

    /// `BRepPrimAPI_MakeRevolution(Meridian, VMin, VMax)`;
    /// `BRepPrimAPIMakeRevolution::with_range(&m, 0.0, 1.0)?`.
    pub fn with_range(
        meridian: &Handle<dyn GeomCurve>,
        vmin: f64,
        vmax: f64,
    ) -> Result<Self, Failure> {
        let axes = standard_frame(Point3::new(0.0, 0.0, 0.0))?;
        Self::build_on(axes, (vmin, vmax), meridian, None)
    }

    /// `BRepPrimAPI_MakeRevolution(Meridian, VMin, VMax, angle)`;
    /// `BRepPrimAPIMakeRevolution::with_range_angle(&m, 0.0, 1.0, 3.0)?`.
    pub fn with_range_angle(
        meridian: &Handle<dyn GeomCurve>,
        vmin: f64,
        vmax: f64,
        angle: f64,
    ) -> Result<Self, Failure> {
        let axes = standard_frame(Point3::new(0.0, 0.0, 0.0))?;
        Self::build_on(axes, (vmin, vmax), meridian, Some(angle))
    }

    /// `BRepPrimAPI_MakeRevolution(Axes, Meridian)`;
    /// `BRepPrimAPIMakeRevolution::with_axes(axes, &m)?`.
    pub fn with_axes(axes: Axis2, meridian: &Handle<dyn GeomCurve>) -> Result<Self, Failure> {
        Self::build_on(axes, meridian_range(meridian)?, meridian, None)
    }

    /// `BRepPrimAPI_MakeRevolution(Axes, Meridian, angle)`;
    /// `BRepPrimAPIMakeRevolution::with_axes_angle(axes, &m, 3.0)?`.
    pub fn with_axes_angle(
        axes: Axis2,
        meridian: &Handle<dyn GeomCurve>,
        angle: f64,
    ) -> Result<Self, Failure> {
        Self::build_on(axes, meridian_range(meridian)?, meridian, Some(angle))
    }

    /// `BRepPrimAPI_MakeRevolution(Axes, Meridian, VMin, VMax)`;
    /// `BRepPrimAPIMakeRevolution::with_axes_range(axes, &m, 0.0, 1.0)?`.
    pub fn with_axes_range(
        axes: Axis2,
        meridian: &Handle<dyn GeomCurve>,
        vmin: f64,
        vmax: f64,
    ) -> Result<Self, Failure> {
        Self::build_on(axes, (vmin, vmax), meridian, None)
    }

    /// `BRepPrimAPI_MakeRevolution(Axes, Meridian, VMin, VMax, angle)`;
    /// `BRepPrimAPIMakeRevolution::with_axes_range_angle(axes, &m, 0.0, 1.0, 3.0)?`.
    pub fn with_axes_range_angle(
        axes: Axis2,
        meridian: &Handle<dyn GeomCurve>,
        vmin: f64,
        vmax: f64,
        angle: f64,
    ) -> Result<Self, Failure> {
        Self::build_on(axes, (vmin, vmax), meridian, Some(angle))
    }

    /// `Revolution()`: the algorithm; `maker.revolution()`.
    pub fn revolution(&mut self) -> &mut BRepPrimRevolution {
        &mut self.my_revolution
    }
}

one_axis_maker!(BRepPrimAPIMakeRevolution, my_revolution);
