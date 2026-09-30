// Rust translation of ModelingData/TKGeomBase/GeomLib/GeomLib.cxx
// (GeomLib_CurveOnSurfaceEvaluator, BuildCurve3d) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomLib::BuildCurve3d`: the 3D curve of a curve on a surface, exact on planes
//! and isolines, approximated by AdvApprox otherwise.
use super::iso_line::{build_c3d_on_iso_line, is_iso_line};
use super::make_curve_from_approx::MakeCurvefromApprox;
use super::to_3d::to_3d;
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::adv_approx::{
    AdvApproxApproxAFunction, AdvApproxEvaluatorFunction, AdvApproxPrefAndRec,
};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomElementarySurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;

/// `GeomLib::BuildCurve3d(Tolerance, CurvePtr, FirstParameter, LastParameter,
/// NewCurvePtr, MaxDeviation, AverageDeviation, Continuity = GeomAbs_C1, MaxDegree =
/// 14, MaxSegment = 30)`: the 3D curve of `curve` on `[first_parameter,
/// last_parameter]` into `new_curve_ptr`, and the maximum and average deviations
/// of the approximation (0 for the exact cases).
///
/// When the curve on surface is a Geom2dAdaptor_Curve on a GeomAdaptor_Surface: on a
/// plane (or a trimmed plane) the pcurve is placed in the plane (To3d of the whole
/// pcurve); on an isoline the surface isoline is used when within the tolerance (a
/// null isoline result is written to `new_curve_ptr` before the fallback, as
/// natively). Otherwise the curve is approximated with AdvApprox (one 3D subspace
/// within `tolerance`, cut preferably at its C2 then C3 breaks); without an
/// approximation result `new_curve_ptr` keeps its value;
/// `geom_lib::build_curve3d(1e-7, &cos, (0.0, 1.0), &mut curve, Shape::C1, 14, 30)?`.
pub fn build_curve3d(
    tolerance: f64,
    curve: &Adaptor3dCurveOnSurface,
    (first_parameter, last_parameter): (f64, f64),
    new_curve_ptr: &mut Handle<dyn GeomCurve>,
    continuity: Shape,
    max_degree: i32,
    max_segment: i32,
) -> Result<(f64, f64), Failure> {
    if exact_curve(
        tolerance,
        curve,
        (first_parameter, last_parameter),
        new_curve_ptr,
    )? {
        return Ok((0.0, 0.0));
    }
    approximated_curve(
        tolerance,
        curve,
        [first_parameter, last_parameter],
        (continuity, max_degree, max_segment),
        new_curve_ptr,
    )
}

// The planar and isoline cases of a Geom2dAdaptor_Curve on a GeomAdaptor_Surface.
// True when the result is final.
fn exact_curve(
    tolerance: f64,
    curve: &Adaptor3dCurveOnSurface,
    (first_parameter, last_parameter): (f64, f64),
    new_curve_ptr: &mut Handle<dyn GeomCurve>,
) -> Result<bool, Failure> {
    let surface_handle: Handle<dyn Adaptor3dSurface> = curve.get_surface();
    let curve_handle: Handle<dyn Adaptor2dCurve2d> = curve.get_curve();
    let geom_surface: Handle<GeomAdaptorSurface> = surface_handle.cast();
    let geom_curve: Handle<Geom2dAdaptorCurve> = curve_handle.cast();
    let (Some(geom_surface), Some(geom_curve)) = (geom_surface.get(), geom_curve.get()) else {
        return Ok(false);
    };
    let surface: Handle<dyn GeomSurface> = geom_surface.surface();
    let rt: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    let plane: Handle<GeomPlane> = match rt.get() {
        Some(trimmed) => trimmed.basis_surface().cast(),
        None => surface.cast(),
    };
    if let Some(p) = plane.get() {
        // Compute the 3d curve.
        *new_curve_ptr = to_3d(p.position().ax2()?, &geom_curve.curve())?;
        return Ok(true);
    }
    let trimmed_c2d: Handle<dyn Adaptor2dCurve2d> =
        geom_curve.trim(first_parameter, last_parameter, precision::p_confusion())?;
    if let Some(iso) = is_iso_line(&trimmed_c2d)? {
        *new_curve_ptr = build_c3d_on_iso_line(
            &trimmed_c2d,
            &surface_handle,
            (first_parameter, last_parameter),
            tolerance,
            iso,
        )?;
        if !new_curve_ptr.is_null() {
            return Ok(true);
        }
    }
    Ok(false)
}

// The AdvApprox approximation with preferential cutting at the discontinuities.
fn approximated_curve(
    tolerance: f64,
    curve: &Adaptor3dCurveOnSurface,
    [first_parameter, last_parameter]: [f64; 2],
    (continuity, max_degree, max_segment): (Shape, i32, i32),
    new_curve_ptr: &mut Handle<dyn GeomCurve>,
) -> Result<(f64, f64), Failure> {
    let mut tolerance_3d: Array1<'static, f64> = one_based(1)?;
    tolerance_3d.init(tolerance);
    // Search for discontinuities.
    let cut_c2: Array1<'static, f64> = cut_points(curve, Shape::C2)?;
    let cut_c3: Array1<'static, f64> = cut_points(curve, Shape::C3)?;
    let mut ev = CurveOnSurfaceEvaluator::new(curve, first_parameter, last_parameter)?;
    // Approximation with preferential cutting.
    let preferentiel = AdvApproxPrefAndRec::new(&cut_c2, &cut_c3, 5.0)?;
    let approximator = AdvApproxApproxAFunction::with_cutting(
        0,
        0,
        1,
        None,
        None,
        Some(&tolerance_3d),
        first_parameter,
        last_parameter,
        continuity,
        max_degree,
        max_segment,
        &mut ev,
        &preferentiel,
    );
    ev.capture.take()?;
    let approximator = approximator?;
    if !approximator.has_result() {
        return Ok((0.0, 0.0));
    }
    let curve_builder = MakeCurvefromApprox::new(&approximator);
    let curve_ptr = curve_builder.curve(1)?;
    // Return the approximation results.
    let deviations = (
        approximator.max_error(3, 1)?,
        approximator.average_error(3, 1)?,
    );
    *new_curve_ptr = curve_ptr.cast();
    Ok(deviations)
}

fn cut_points(
    curve: &Adaptor3dCurveOnSurface,
    shape: Shape,
) -> Result<Array1<'static, f64>, Failure> {
    let mut points: Array1<'static, f64> = one_based(curve.nb_intervals(shape)? + 1)?;
    curve.intervals(&mut points, shape)?;
    Ok(points)
}

/// `GeomLib_CurveOnSurfaceEvaluator`: the value, first or second derivative of the
/// curve on surface, trimmed again whenever AdvApprox moves to another interval;
/// a higher derivative request writes nothing, and the return code is always 0.
struct CurveOnSurfaceEvaluator<'c> {
    curve_on_surface: &'c Adaptor3dCurveOnSurface,
    first_param: f64,
    last_param: f64,
    trim_curve: Handle<dyn Adaptor3dCurve>,
    // An adaptor exception unwinds natively; it is carried to the end of AdvApprox.
    capture: FailureCapture,
}

impl<'c> CurveOnSurfaceEvaluator<'c> {
    fn new(
        curve_on_surface: &'c Adaptor3dCurveOnSurface,
        first: f64,
        last: f64,
    ) -> Result<Self, Failure> {
        let trim_curve = curve_on_surface.trim(first, last, precision::p_confusion())?;
        Ok(Self {
            curve_on_surface,
            first_param: first,
            last_param: last,
            trim_curve,
            capture: FailureCapture::default(),
        })
    }

    // The derivative `request` at `parameter`, None when nothing is written.
    fn derivative(&self, parameter: f64, request: i32) -> Result<Option<Point3>, Failure> {
        let curve = self.trim_curve.get().ok_or_else(|| {
            crate::native_failure::failure(
                "Standard_NullObject",
                "GeomLib_CurveOnSurfaceEvaluator: null trimmed curve",
            )
        })?;
        Ok(match request {
            0 => Some(curve.eval_d0(parameter)?),
            1 => Some(Point3::from(*curve.eval_d1(parameter)?.d1.xyz())),
            2 => Some(Point3::from(*curve.eval_d2(parameter)?.d2.xyz())),
            _ => None,
        })
    }
}

impl AdvApproxEvaluatorFunction for CurveOnSurfaceEvaluator<'_> {
    fn evaluate(
        &mut self,
        _dimension: i32,
        debut_fin: [f64; 2],
        parameter: f64,
        request: i32,
        result: &mut [f64],
    ) -> i32 {
        // Handle left / right positioning.
        if debut_fin[0] != self.first_param || debut_fin[1] != self.last_param {
            let trimmed =
                self.curve_on_surface
                    .trim(debut_fin[0], debut_fin[1], precision::p_confusion());
            let Some(trimmed) = self.capture.keep(trimmed) else {
                return 1;
            };
            self.trim_curve = trimmed;
            self.first_param = debut_fin[0];
            self.last_param = debut_fin[1];
        }
        match self.capture.keep(self.derivative(parameter, request)) {
            Some(Some(value)) => {
                (result[0], result[1], result[2]) = (value.x(), value.y(), value.z());
                0
            }
            Some(None) => 0,
            None => 1,
        }
    }
}
