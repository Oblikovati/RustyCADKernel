//! HLRBRep_CLPropsATool.hxx/.lxx and HLRBRep_CLProps.hxx, OCCT
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::Curve;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::geom_lprop::CLPropsBase;
use rustycad_tkgeombase::lprop::curve_access::{CurveD1, CurveD2, CurveD3, CurveTool, ToolAccess};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Vector2};

/// HLRBRep_CLPropsATool: the static evaluation of a projected [`Curve`] for the
/// local properties; `CLPropsATool::value(&curve, 0.5)?`.
pub struct CLPropsATool;

/// HLRBRep_CLProps: `GeomLProp_CLPropsBase` on a projected curve through
/// [`CLPropsATool`]; `CLProps::new(&curve, 2, 1e-7)?`.
pub type CLProps<'a> = CLPropsBase<Vector2, &'a Curve, ToolAccess<CLPropsATool>>;

impl CLPropsATool {
    /// Value(A, U, P); `CLPropsATool::value(&curve, 0.5)?`.
    pub fn value(a: &Curve, u: f64) -> Result<Point2, Failure> {
        a.value(u)
    }
    /// D1(A, U, P, V1); `CLPropsATool::d1(&curve, 0.5)?`.
    pub fn d1(a: &Curve, u: f64) -> Result<(Point2, Vector2), Failure> {
        let (mut p, mut v1) = (Point2::default(), Vector2::default());
        a.d1(u, &mut p, &mut v1)?;
        Ok((p, v1))
    }
    /// D2(A, U, P, V1, V2); `CLPropsATool::d2(&curve, 0.5)?`.
    pub fn d2(a: &Curve, u: f64) -> Result<(Point2, Vector2, Vector2), Failure> {
        let (mut p, mut v1, mut v2) = (Point2::default(), Vector2::default(), Vector2::default());
        a.d2(u, &mut p, &mut v1, &mut v2)?;
        Ok((p, v1, v2))
    }
    /// D3(A, U, P, V1, V2, V3): the native curve leaves its outputs unchanged; the
    /// tool starts them at the origin and null vectors;
    /// `CLPropsATool::d3(&curve, 0.5)`.
    pub fn d3(a: &Curve, u: f64) -> (Point2, Vector2, Vector2, Vector2) {
        let mut out = (
            Point2::default(),
            Vector2::default(),
            Vector2::default(),
            Vector2::default(),
        );
        a.d3(u, &mut out.0, &mut out.1, &mut out.2, &mut out.3);
        out
    }
    /// Continuity(A): always C2 (`GeomAbs_C2` as an integer);
    /// `CLPropsATool::continuity(&curve)`.
    pub fn continuity(_a: &Curve) -> i32 {
        Shape::C2 as i32
    }
    /// FirstParameter(A); `CLPropsATool::first_parameter(&curve)?`.
    pub fn first_parameter(a: &Curve) -> Result<f64, Failure> {
        a.first_parameter()
    }
    /// LastParameter(A); `CLPropsATool::last_parameter(&curve)?`.
    pub fn last_parameter(a: &Curve) -> Result<f64, Failure> {
        a.last_parameter()
    }
}

impl<'a> CurveTool<&'a Curve> for CLPropsATool {
    type Vector = Vector2;
    fn value(curve: &&'a Curve, u: f64) -> Result<Point2, Failure> {
        CLPropsATool::value(curve, u)
    }
    fn d1(curve: &&'a Curve, u: f64) -> Result<CurveD1<Vector2>, Failure> {
        CLPropsATool::d1(curve, u)
    }
    fn d2(curve: &&'a Curve, u: f64) -> Result<CurveD2<Vector2>, Failure> {
        CLPropsATool::d2(curve, u)
    }
    fn d3(curve: &&'a Curve, u: f64) -> Result<CurveD3<Vector2>, Failure> {
        Ok(CLPropsATool::d3(curve, u))
    }
    fn first_parameter(curve: &&'a Curve) -> Result<f64, Failure> {
        CLPropsATool::first_parameter(curve)
    }
    fn last_parameter(curve: &&'a Curve) -> Result<f64, Failure> {
        CLPropsATool::last_parameter(curve)
    }
}
