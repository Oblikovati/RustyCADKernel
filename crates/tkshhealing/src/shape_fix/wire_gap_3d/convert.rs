use super::{Failure, GapCurve, GeomBSplineCurve, GeomCurve, Handle, Point3};
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkernel::precision;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{
        GeomCircle, GeomEllipse, GeomHyperbola, GeomOffsetCurve, GeomParabola, GeomTrimmedCurve,
    },
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkgeombase::{approx::Curve3d, geom_convert};
use rustycad_tkmath::{
    convert::ParameterisationType,
    geom_abs::Shape,
    gp::{Direction3, Xyz},
};
pub(super) fn convert_gap_curve(
    entry: &GapCurve,
    target: Point3,
    precision: f64,
    approximation: f64,
    same: bool,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if entry.point.distance(&target) < precision && !same {
        return Ok(Handle::default());
    }
    let curve = basic_gap_curve(&entry.curve)?;
    if !curve.cast::<GeomBSplineCurve>().is_null() {
        return segment_gap_spline(&curve, entry.first, entry.last);
    }
    if is_conic(&curve) {
        let adaptor: Handle<dyn Adaptor3dCurve> = Handle::new(GeomAdaptorCurve::with_range(
            &curve,
            entry.first,
            entry.last,
        )?)
        .cast();
        let approximation = Curve3d::new(&adaptor, approximation, Shape::C1, 9, 1000)?;
        return Ok(if approximation.is_done() || approximation.has_result() {
            approximation.curve()
        } else {
            Handle::default()
        });
    }
    // The native generic conversion branch catches Standard_Failure and retains a null result.
    Ok(trim_and_convert(&curve, entry.first, entry.last).unwrap_or_default())
}
fn basic_gap_curve(source: &Handle<dyn GeomCurve>) -> Result<Handle<dyn GeomCurve>, Failure> {
    let mut curve = source.clone();
    let mut offset = Xyz::new(0., 0., 0.);
    let mut has_offset = false;
    loop {
        let trim = curve.cast::<GeomTrimmedCurve>();
        if let Some(trim) = trim.get() {
            curve = trim.basis_curve();
            continue;
        }
        let shifted = curve.cast::<GeomOffsetCurve>();
        let Some(shifted) = shifted.get() else {
            break;
        };
        offset += shifted.direction().xyz() * shifted.offset();
        has_offset = true;
        curve = shifted.basis_curve();
    }
    if !has_offset {
        return Ok(curve);
    }
    Ok(Handle::new(GeomOffsetCurve::new(
        &curve,
        offset.modulus(),
        Direction3::from_xyz(offset)?,
        false,
    )?)
    .cast())
}
fn is_conic(curve: &Handle<dyn GeomCurve>) -> bool {
    !curve.cast::<GeomCircle>().is_null()
        || !curve.cast::<GeomEllipse>().is_null()
        || !curve.cast::<GeomHyperbola>().is_null()
        || !curve.cast::<GeomParabola>().is_null()
}
fn segment_gap_spline(
    curve: &Handle<dyn GeomCurve>,
    first: f64,
    last: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    let spline = curve.copy()?.cast::<GeomBSplineCurve>();
    let begin = spline.first_parameter();
    let end = spline.last_parameter();
    if first > begin || last < end {
        return geom_convert::split_bspline_curve(
            &spline,
            first.max(begin),
            last.min(end),
            precision::confusion(),
            true,
        );
    }
    Ok(spline)
}
fn trim_and_convert(
    curve: &Handle<dyn GeomCurve>,
    mut first: f64,
    mut last: f64,
) -> Result<Handle<GeomBSplineCurve>, Failure> {
    if !ShapeAnalysisCurve::is_periodic(curve)? {
        first = first.max(curve.first_parameter());
        last = last.min(curve.last_parameter());
    }
    let trimmed: Handle<dyn GeomCurve> =
        Handle::new(GeomTrimmedCurve::new(curve, first, last, true, true)?).cast();
    geom_convert::curve_to_bspline_curve(&trimmed, ParameterisationType::TgtThetaOver2)
}
pub(super) fn move_gap_endpoint(
    curve: &Handle<GeomBSplineCurve>,
    target: Point3,
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
