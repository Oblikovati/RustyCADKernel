//! Geometry operations differ by dimension; native Build policies are explicit.
use super::*;
use crate::shape_analysis::ShapeAnalysisCurve;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurve, Geom2dGeometry, Geom2dOffsetCurve,
    Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBezierCurve, GeomCurve, GeomGeometry, GeomOffsetCurve, GeomTrimmedCurve,
};
pub(in crate::shape_upgrade) struct CurveFamily2d;
pub(in crate::shape_upgrade) struct CurveFamily3d;
macro_rules! curve_geometry_family {
    ($family:ty,$curve:ty,$spline:ty,$bezier:ty,$trimmed:ty,$offset:ty,$name:literal,$clamp:expr,$single:expr,$periodic:ident,$offset_piece:ident) => {
        impl CurveGeometry for $family {
            type Curve = $curve;
            const NAME: &'static str = $name;
            const CLAMP_BUILD: bool = $clamp;
            const SEGMENT_SINGLE: bool = $single;
            fn copy(curve: &Handle<Self::Curve>) -> Result<Handle<Self::Curve>, Failure> {
                Ok(curve.copy()?.cast())
            }
            fn bounds(curve: &Handle<Self::Curve>) -> (f64, f64) {
                (curve.first_parameter(), curve.last_parameter())
            }
            fn periodic(curve: &Handle<Self::Curve>) -> Result<bool, Failure> {
                ShapeAnalysisCurve::$periodic(curve)
            }
            fn trimmed_basis(curve: &Handle<Self::Curve>) -> Option<Handle<Self::Curve>> {
                curve.cast::<$trimmed>().get().map(<$trimmed>::basis_curve)
            }
            fn offset_basis(curve: &Handle<Self::Curve>) -> Option<Handle<Self::Curve>> {
                curve.cast::<$offset>().get().map(<$offset>::basis_curve)
            }
            fn offset(
                curve: &Handle<Self::Curve>,
                basis: &Handle<Self::Curve>,
            ) -> Result<Handle<Self::Curve>, Failure> {
                $offset_piece(curve, basis)
            }
            fn trim(
                curve: &Handle<Self::Curve>,
                first: f64,
                last: f64,
            ) -> Result<Handle<Self::Curve>, Failure> {
                Ok(Handle::new(<$trimmed>::new(curve, first, last, true, true)?).cast())
            }
            fn spline(curve: &Handle<Self::Curve>) -> bool {
                !curve.cast::<$spline>().is_null() || !curve.cast::<$bezier>().is_null()
            }
            fn segment(curve: &Handle<Self::Curve>, first: f64, last: f64) -> Result<(), Failure> {
                if let Some(spline) = curve.cast::<$spline>().get() {
                    return spline.segment(first, last, precision::p_confusion());
                }
                curve.cast::<$bezier>().segment(first, last)
            }
            fn knots(curve: &Handle<Self::Curve>) -> Result<Option<Vec<f64>>, Failure> {
                let spline = curve.cast::<$spline>();
                if spline.is_null() {
                    return Ok(None);
                }
                let copy: Handle<$spline> = spline.copy()?.cast();
                let knots = (copy.first_u_knot_index()?..=copy.last_u_knot_index()?)
                    .map(|i| copy.knot(i))
                    .collect::<Result<Vec<_>, Failure>>()?;
                Ok(Some(knots))
            }
        }
    };
}
curve_geometry_family!(
    CurveFamily2d,
    dyn Geom2dCurve,
    Geom2dBSplineCurve,
    Geom2dBezierCurve,
    Geom2dTrimmedCurve,
    Geom2dOffsetCurve,
    "ShapeUpgrade_SplitCurve2d",
    false,
    true,
    is_periodic_2d,
    offset_piece_2d
);
curve_geometry_family!(
    CurveFamily3d,
    dyn GeomCurve,
    GeomBSplineCurve,
    GeomBezierCurve,
    GeomTrimmedCurve,
    GeomOffsetCurve,
    "ShapeUpgrade_SplitCurve3d",
    true,
    false,
    is_periodic,
    offset_piece_3d
);
fn offset_piece_2d(
    curve: &Handle<dyn Geom2dCurve>,
    basis: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let offset = curve.cast::<Geom2dOffsetCurve>();
    Ok(Handle::new(Geom2dOffsetCurve::new(basis, offset.offset(), false)?).cast())
}
fn offset_piece_3d(
    curve: &Handle<dyn GeomCurve>,
    basis: &Handle<dyn GeomCurve>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let offset = curve.cast::<GeomOffsetCurve>();
    Ok(Handle::new(GeomOffsetCurve::new(
        basis,
        offset.offset(),
        offset.direction(),
        false,
    )?)
    .cast())
}
