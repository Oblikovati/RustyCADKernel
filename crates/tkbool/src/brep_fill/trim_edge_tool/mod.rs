// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimEdgeTool.hxx/.cxx
// (the constructors, AddOrConfuse and IsInside) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_TrimEdgeTool`: the intersections of a bisector with the parallels of the
//! two elements it separates, which trim the parallels of an offset wire.
mod intersect;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dCartesianPoint, Geom2dCurve, Geom2dGeometry, Geom2dPoint, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::bisector::{Bisec, BisecAna};

/// The trimming of the parallels of two elements by their bisector
/// (`BRepFill_TrimEdgeTool`); `let t = BRepFillTrimEdgeTool::new(&bisec, &s1, &s2, 1.0)?;`.
#[derive(Default)]
pub struct BRepFillTrimEdgeTool {
    is_point1: bool,
    is_point2: bool,
    my_p1: Point2,
    my_p2: Point2,
    my_c1: Handle<dyn Geom2dCurve>,
    my_c2: Handle<dyn Geom2dCurve>,
    my_offset: f64,
    my_bisec: Bisec,
    my_bis: Geom2dAdaptorCurve,
}

/// `SimpleExpression(B, Bis)`: the bisector, its analytic basis made a trimmed curve of
/// its plain geometric curve.
fn simple_expression(b: &Bisec) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let tr_bis: Handle<Geom2dTrimmedCurve> = b.value();
    let Some(trimmed) = tr_bis.get() else {
        return Ok(tr_bis.cast());
    };
    let analytic: Handle<BisecAna> = trimmed.basis_curve().cast();
    let Some(analytic) = analytic.get() else {
        return Ok(tr_bis.clone().cast());
    };
    let bis = analytic.geom2d_curve();
    let (first, last) = (trimmed.first_parameter(), trimmed.last_parameter());
    Ok(Handle::new(Geom2dTrimmedCurve::new(&bis, first, last, true, true)?).cast())
}

/// The point of a Geom2d_CartesianPoint geometry, `None` for a curve.
fn point_of(s: &Handle<dyn Geom2dGeometry>) -> Option<Point2> {
    let point: Handle<Geom2dCartesianPoint> = s.cast();
    point.get().map(|p| p.pnt2d())
}

/// The first curve on surface of an edge as an adaptor on its range, with the curve.
pub(super) fn pcurve_adaptor(
    edge: &Edge,
) -> Result<(Geom2dAdaptorCurve, Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let on_surface = BRepTool::curve_on_surface_by_index(edge, 1)?;
    let (f, l) = (on_surface.first, on_surface.last);
    let adaptor = Geom2dAdaptorCurve::with_range(&on_surface.pcurve, f, l)?;
    Ok((adaptor, on_surface.pcurve, f, l))
}

impl BRepFillTrimEdgeTool {
    /// `BRepFill_TrimEdgeTool(Bisec, S1, S2, Offset)`: the bisector of the two elements
    /// (points or curves) and the offset of their parallels.
    pub fn new(
        bisec: &Bisec,
        s1: &Handle<dyn Geom2dGeometry>,
        s2: &Handle<dyn Geom2dGeometry>,
        offset: f64,
    ) -> Result<Self, Failure> {
        let mut tool = Self {
            my_offset: offset,
            my_bisec: bisec.clone(),
            ..Self::default()
        };
        if let Some(p) = point_of(s1) {
            (tool.is_point1, tool.my_p1) = (true, p);
        } else {
            tool.my_c1 = s1.cast();
        }
        if let Some(p) = point_of(s2) {
            (tool.is_point2, tool.my_p2) = (true, p);
        } else {
            tool.my_c2 = s2.cast();
        }
        tool.my_bis = Geom2dAdaptorCurve::with_curve(&simple_expression(&tool.my_bisec)?)?;
        Ok(tool)
    }

    /// `AddOrConfuse(Start, Edge1, Edge2, Params)`: the first (or last) point of the
    /// bisector projected on both parallels and added to the parameters when it is not
    /// already their first (last) intersection; nothing when a projection fails or
    /// misses the point by 10 * Precision::Confusion().
    pub fn add_or_confuse(
        &self,
        start: bool,
        edge1: &Edge,
        edge2: &Edge,
        params: &mut Sequence<Point3>,
    ) -> Result<(), Failure> {
        let tol: f64 = 10. * precision::confusion();
        let (ac1, c1, f, l) = pcurve_adaptor(edge1)?;
        let bis_par: f64 = if start {
            self.my_bis.first_parameter()?
        } else {
            self.my_bis.last_parameter()?
        };
        let p_bis: Point2 = self.my_bis.value(bis_par)?;
        let known = if start { params.first() } else { params.last() };
        if let Some(known) = known
            && p_bis.is_equal(ac1.value(known.y())?, tol)
        {
            return Ok(());
        }
        let on_surface2 = BRepTool::curve_on_surface_by_index(edge2, 1)?;
        let projector1 = ProjectPointOnCurve::with_range(p_bis, &c1, f, l)?;
        let projector2 = ProjectPointOnCurve::with_range(
            p_bis,
            &on_surface2.pcurve,
            on_surface2.first,
            on_surface2.last,
        )?;
        if !projected_on(&projector1, p_bis, tol)? || !projected_on(&projector2, p_bis, tol)? {
            return Ok(());
        }
        let p_int = Point3::new(
            bis_par,
            projector1.lower_distance_parameter()?,
            projector2.lower_distance_parameter()?,
        );
        if start {
            params.prepend(p_int);
        } else {
            params.append(p_int);
        }
        Ok(())
    }

    /// `IsInside(P)`: whether the point is closer than |Offset| - Precision::Confusion()
    /// to the first element (the first point, else the second point, else the nearest of
    /// the projection on the first curve and its ends).
    pub fn is_inside(&self, p: Point2) -> Result<bool, Failure> {
        let dist: f64 = if self.is_point1 {
            p.distance(self.my_p1)
        } else if self.is_point2 {
            p.distance(self.my_p2)
        } else {
            self.distance_to_first_curve(p)?
        };
        Ok(dist < self.my_offset.abs() - precision::confusion())
    }

    /// The distance of a point to the first curve: its projection, bounded by its
    /// distances to the curve ends.
    fn distance_to_first_curve(&self, p: Point2) -> Result<f64, Failure> {
        let mut dist: f64 = f64::MAX;
        let projector = ProjectPointOnCurve::with_curve(p, &self.my_c1)?;
        if projector.nb_points() > 0 {
            dist = projector.lower_distance()?;
        }
        let c1 = self.my_c1.get().ok_or_else(null_curve)?;
        let pf: Point2 = c1.value(c1.first_parameter())?;
        let pl: Point2 = c1.value(c1.last_parameter())?;
        let a_dist_min: f64 = p.distance(pf).min(p.distance(pl));
        Ok(if dist > a_dist_min { a_dist_min } else { dist })
    }
}

/// Whether a projection has a solution at the point.
fn projected_on(projector: &ProjectPointOnCurve, p: Point2, tol: f64) -> Result<bool, Failure> {
    // A failed projection or an incorrect solution: nothing is added (debug messages
    // natively).
    Ok(projector.nb_points() > 0 && projector.nearest_point()?.is_equal(p, tol))
}

/// Standard_NullObject of a null first curve.
fn null_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_TrimEdgeTool: null first curve; expected a curve element",
        "BRepFill_TrimEdgeTool",
    )
}
