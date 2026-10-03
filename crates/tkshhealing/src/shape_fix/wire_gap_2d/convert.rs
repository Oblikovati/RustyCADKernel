use super::ShapeFixWire;
use super::{Failure, Geom2dBSplineCurve, Geom2dCurve, Handle, PcurveGap, Point2};
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::precision;
use rustycad_tkg2d::{
    adaptor2d::Adaptor2dCurve2d,
    geom2d::{
        Geom2dCircle, Geom2dEllipse, Geom2dHyperbola, Geom2dOffsetCurve, Geom2dParabola,
        Geom2dTrimmedCurve,
    },
    geom2d_adaptor::Geom2dAdaptorCurve,
};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkgeombase::{approx::Curve2d, geom2d_convert};
use rustycad_tkmath::{convert::ParameterisationType, geom_abs::Shape};
pub(super) fn convert_gap_curve(
    tool: &ShapeFixWire,
    entry: &PcurveGap,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let curve = basic_gap_curve(&entry.curve)?;
    if !curve.cast::<Geom2dBSplineCurve>().is_null() {
        return segment_gap_spline(&curve, entry.first, entry.last);
    }
    if is_conic(&curve) {
        let adaptor: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_range(
            &curve,
            entry.first,
            entry.last,
        )?)
        .cast();
        let surface = tool.working_analyzer().surface().adaptor3d();
        let tolerance = tool.working_analyzer().precision();
        let tolerances = (
            surface.u_resolution(tolerance)?,
            surface.v_resolution(tolerance)?,
        );
        let approximation = Curve2d::new(
            &adaptor,
            entry.first,
            entry.last,
            tolerances.0,
            tolerances.1,
            Shape::C1,
            9,
            1000,
        )?;
        return Ok(if approximation.is_done() || approximation.has_result() {
            approximation.curve()
        } else {
            Handle::default()
        });
    }
    // The native generic conversion branch catches Standard_Failure and retains a null result.
    Ok(trim_and_convert(&curve, entry.first, entry.last).unwrap_or_default())
}
pub(super) fn basic_gap_curve(
    source: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let mut curve = source.clone();
    let mut offset = 0.;
    let mut has_offset = false;
    loop {
        let trim = curve.cast::<Geom2dTrimmedCurve>();
        if let Some(trim) = trim.get() {
            curve = trim.basis_curve();
            continue;
        }
        let shifted = curve.cast::<Geom2dOffsetCurve>();
        let Some(shifted) = shifted.get() else {
            break;
        };
        offset += shifted.offset();
        has_offset = true;
        curve = shifted.basis_curve();
    }
    if !has_offset {
        return Ok(curve);
    }
    Ok(Handle::new(Geom2dOffsetCurve::new(&curve, offset, false)?).cast())
}
fn is_conic(curve: &Handle<dyn Geom2dCurve>) -> bool {
    !curve.cast::<Geom2dCircle>().is_null()
        || !curve.cast::<Geom2dEllipse>().is_null()
        || !curve.cast::<Geom2dHyperbola>().is_null()
        || !curve.cast::<Geom2dParabola>().is_null()
}
fn segment_gap_spline(
    curve: &Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    let spline = curve.copy()?.cast::<Geom2dBSplineCurve>();
    let begin = spline.first_parameter();
    let end = spline.last_parameter();
    if first > begin || last < end {
        return geom2d_convert::split_bspline_curve(
            &spline,
            first.max(begin),
            last.min(end),
            precision::p_confusion(),
            true,
        );
    }
    Ok(spline)
}
fn trim_and_convert(
    curve: &Handle<dyn Geom2dCurve>,
    mut first: f64,
    mut last: f64,
) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
    if !ShapeAnalysisCurve::is_periodic_2d(curve)? {
        first = first.max(curve.first_parameter());
        last = last.min(curve.last_parameter());
    }
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(curve, first, last, true, true)?).cast();
    geom2d_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)
}
pub(super) fn move_gap_endpoint(
    curve: &Handle<Geom2dBSplineCurve>,
    target: Point2,
    first: bool,
    both: bool,
) -> Result<(), Failure> {
    let Some(curve) = curve.get() else {
        return Ok(());
    };
    if curve.degree() == 1 {
        curve.increase_degree(2)?;
    }
    if first || both {
        curve.set_pole(1, target)?;
    }
    if !first || both {
        curve.set_pole(curve.nb_poles(), target)?;
    }
    Ok(())
}
