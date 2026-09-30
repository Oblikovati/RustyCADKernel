// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepApprox/BRepApprox_ApproxLine.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepApprox_ApproxLine`: the line of points that BRepApprox approximates, either the
//! points of an `IntSurf_LineOn2S` or the poles of three B-spline curves
//! (docs/topalgo-design.md section 12).
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::Geom2dBSplineCurve;
use rustycad_tkg3d::geom::GeomBSplineCurve;
use rustycad_tkgeomalgo::approx_int::ApproxIntLine;
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkmath::gp::{Point2, Point3};

/// `BRepApprox_ApproxLine`: the `TheWLine` of the BRepApprox instances. The four handles
/// are set by the constructors (each constructor leaves the others null) and never
/// change.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeomalgo::int_surf::IntSurfLineOn2S;
/// use rustycad_tktopalgo::brep_approx::BRepApproxApproxLine;
/// let line = BRepApproxApproxLine::from_line(&Handle::new(IntSurfLineOn2S::new(None)), false);
/// assert_eq!(line.nb_pnts()?, 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct BRepApproxApproxLine {
    parent: StandardTransient,
    curve_xyz: Handle<GeomBSplineCurve>,
    curve_uv1: Handle<Geom2dBSplineCurve>,
    curve_uv2: Handle<Geom2dBSplineCurve>,
    line_on_2s: Handle<IntSurfLineOn2S>,
}
impl_transient!(
    BRepApproxApproxLine,
    "BRepApprox_ApproxLine",
    StandardTransient,
    parent
);

impl BRepApproxApproxLine {
    /// `BRepApprox_ApproxLine(CurveXYZ, CurveUV1, CurveUV2)`: the poles of the curves
    /// (any of them may be null) are the points;
    /// `BRepApproxApproxLine::from_curves(&xyz, &uv1, &Handle::default())`.
    pub fn from_curves(
        curve_xyz: &Handle<GeomBSplineCurve>,
        curve_uv1: &Handle<Geom2dBSplineCurve>,
        curve_uv2: &Handle<Geom2dBSplineCurve>,
    ) -> Self {
        Self {
            curve_xyz: curve_xyz.clone(),
            curve_uv1: curve_uv1.clone(),
            curve_uv2: curve_uv2.clone(),
            ..Self::default()
        }
    }

    /// `BRepApprox_ApproxLine(lin, theTang)`: the points of `lin`; `tang` is unused (kept
    /// for the `IntPatch_WLine` signature); `BRepApproxApproxLine::from_line(&points, false)`.
    pub fn from_line(lin: &Handle<IntSurfLineOn2S>, _tang: bool) -> Self {
        Self {
            line_on_2s: lin.clone(),
            ..Self::default()
        }
    }

    /// `NbPnts()`: the pole count of the first non-null curve (3d, then on the first,
    /// then on the second surface), else the point count of the line; Standard_NullObject
    /// when all four are null (a null dereference natively); `line.nb_pnts()?`.
    pub fn nb_pnts(&self) -> Result<i32, Failure> {
        if let Some(c) = self.curve_xyz.get() {
            return Ok(c.nb_poles());
        }
        if let Some(c) = self.curve_uv1.get() {
            return Ok(c.nb_poles());
        }
        if let Some(c) = self.curve_uv2.get() {
            return Ok(c.nb_poles());
        }
        match self.line_on_2s.get() {
            Some(l) => Ok(l.nb_points()),
            None => Err(null_object(
                "BRepApprox_ApproxLine::NbPnts: no curve and a null IntSurf_LineOn2S; expected a curve or a line",
            )),
        }
    }

    /// `Point(Index)`: the point of a non-empty line (Standard_OutOfRange outside it);
    /// otherwise the poles of rank `index` of the curves, the origin for a null curve
    /// (Standard_OutOfRange outside the poles of a curve); `line.point(1)?`.
    pub fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        if let Some(l) = self.line_on_2s.get()
            && l.nb_points() != 0
        {
            return l.value(index);
        }
        let p: Point3 = match self.curve_xyz.get() {
            Some(c) => c.pole(index)?,
            None => Point3::new(0.0, 0.0, 0.0),
        };
        let p1: Point2 = pole_2d(&self.curve_uv1, index)?;
        let p2: Point2 = pole_2d(&self.curve_uv2, index)?;
        let mut pnt_on_2s = IntSurfPntOn2S::new();
        pnt_on_2s.set_value_all(p, p1.x(), p1.y(), p2.x(), p2.y());
        Ok(pnt_on_2s)
    }
}

/// The pole `index` of a 2d curve, or the default `gp_Pnt2d` (the origin) of a null one.
fn pole_2d(curve: &Handle<Geom2dBSplineCurve>, index: i32) -> Result<Point2, Failure> {
    match curve.get() {
        Some(c) => c.pole(index),
        None => Ok(Point2::new(0.0, 0.0)),
    }
}

impl ApproxIntLine for BRepApproxApproxLine {
    fn nb_pnts(&self) -> Result<i32, Failure> {
        BRepApproxApproxLine::nb_pnts(self)
    }
    fn point(&self, index: i32) -> Result<IntSurfPntOn2S, Failure> {
        BRepApproxApproxLine::point(self, index)
    }
    fn from_points(line: &Handle<IntSurfLineOn2S>, tang: bool) -> Self {
        Self::from_line(line, tang)
    }
}
