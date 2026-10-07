// Rust translation of the static ValueOnFace and of BRepFill_MultiLine::Value, ValueOnF1,
// ValueOnF2 and Value3dOnF1OnF2 (ModelingAlgorithms/TKBool/BRepFill/BRepFill_MultiLine.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of the multi-line on each face: the point of the bisector projected on
//! the iso of the edge (its parameter and distance), then the transverse parameter at
//! that distance.
use super::{BRepFillMultiLine, eps};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector2};
use std::f64::consts::PI;

/// The parameter and distance of the point on the iso of the edge: the projection,
/// replaced by an end at the same or a smaller distance (at equal ends, the angle of
/// the bisector start on a circle; Standard_ConstructionError on another curve).
fn on_edge_iso(
    p: Point2,
    the_bis: &Geom2dAdaptorCurve,
    the_u: &Geom2dAdaptorCurve,
) -> Result<(f64, f64), Failure> {
    let (uf, ul) = (the_u.first_parameter()?, the_u.last_parameter()?);
    let ext = ProjectPointOnCurve::with_range(p, &the_u.curve(), uf, ul)?;
    let (mut uu, mut dist): (f64, f64) = (0., precision::infinite());
    if ext.nb_points() != 0 {
        (uu, dist) = (ext.lower_distance_parameter()?, ext.lower_distance()?);
    }
    // Control with the ends.
    let d1: f64 = p.distance(the_u.value(uf)?);
    let d2: f64 = p.distance(the_u.value(ul)?);
    if !(d1 < dist || d2 < dist || (d1 - dist).abs() < eps() || (d2 - dist).abs() < eps()) {
        return Ok((uu, dist));
    }
    if (d1 - d2).abs() < eps() {
        return circle_start(the_bis, the_u);
    }
    Ok(if d1 < d2 { (uf, d1) } else { (ul, d2) })
}

/// The angle of the start tangent of the bisector on a circle iso, in its period, and
/// the radius.
fn circle_start(
    the_bis: &Geom2dAdaptorCurve,
    the_u: &Geom2dAdaptorCurve,
) -> Result<(f64, f64), Failure> {
    if the_u.get_type()? != CurveType::Circle {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            "BRepFill_MultiLine: ValueOnFace",
            "BRepFill_MultiLine",
        ));
    }
    let circle = the_u.circle()?;
    let axis = Vector2::from(circle.x_axis().direction());
    let d12d: Vector2 = the_bis.dn(the_bis.first_parameter()?, 1)?;
    let mut ang: f64 = axis.angle(d12d)?;
    if !circle.is_direct() {
        ang = -ang;
    }
    let uf: f64 = the_u.first_parameter()?;
    Ok((elclib::in_period(ang, uf, uf + 2. * PI), circle.radius()))
}

/// The parameter on the transverse iso at the distance: an end at that height, else
/// the intersection with the line y = dist (negated below the axis), a segment
/// interpolated, the nearer end without intersection; `None` when the intersection
/// fails (the point is then the origin, natively).
fn on_transverse_iso(the_v: &Geom2dAdaptorCurve, mut dist: f64) -> Result<Option<f64>, Failure> {
    let tol: f64 = precision::confusion();
    let (vf, vl) = (the_v.first_parameter()?, the_v.last_parameter()?);
    let (pf, pl) = (the_v.value(vf)?, the_v.value(vl)?);
    if (dist - pf.y().abs()).abs() < tol {
        return Ok(Some(vf));
    }
    if (dist - pl.y().abs()).abs() < tol {
        return Ok(Some(vl));
    }
    // Test if the curve is at the side `negative Y`.
    if pf.y().min(pl.y()) < -tol {
        dist = -dist;
    }
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::new(0., dist), Direction2::X)).cast();
    let cu2 = Geom2dAdaptorCurve::with_curve(&line)?;
    let intersector = GInter::between(the_v, &cu2, 0., tol)?;
    if !intersector.is_done() {
        return Ok(None);
    }
    if intersector.nb_points()? > 0 {
        return Ok(Some(intersector.point(1)?.param_on_first()));
    }
    if intersector.nb_segments()? > 0 {
        let seg = intersector.segment(1)?;
        let (vs1, vs2) = (
            seg.first_point()?.param_on_first(),
            seg.last_point()?.param_on_first(),
        );
        let (ps1, ps2) = (the_v.value(vs1)?, the_v.value(vs2)?);
        let alp: f64 = (dist - ps1.y()) / (ps2.y() - ps1.y());
        return Ok(Some(alp * (vs2 - vs1) + vs1));
    }
    Ok(Some(if (dist - pl.y()).abs() < (dist - pf.y()).abs() {
        vl
    } else {
        vf
    }))
}

/// `ValueOnFace(U, TheBis, TheU, TheV, IsIsoU)`: the point on a face of the bisector
/// point at `u`.
fn value_on_face(
    u: f64,
    the_bis: &Geom2dAdaptorCurve,
    the_u: &Geom2dAdaptorCurve,
    the_v: &Geom2dAdaptorCurve,
    is_iso_u: bool,
) -> Result<Point2, Failure> {
    let p: Point2 = the_bis.value(u)?;
    let (uu, dist) = on_edge_iso(p, the_bis, the_u)?;
    let Some(vv) = on_transverse_iso(the_v, dist)? else {
        return Ok(Point2::new(0., 0.));
    };
    Ok(if is_iso_u {
        Point2::new(vv, uu)
    } else {
        Point2::new(uu, vv)
    })
}

/// The point of the surface of a face at UV, moved by its location.
fn on_surface(face: &Face, p2d: Point2) -> Result<Point3, Failure> {
    let (s, l) = BRepTool::surface_with_location(face)?;
    let surface = s.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "BRepFill_MultiLine: face without surface; expected a surface",
            "BRepFill_MultiLine",
        )
    })?;
    Ok(surface
        .value(p2d.x(), p2d.y())?
        .transformed(l.transformation()))
}

impl BRepFillMultiLine {
    /// `ValueOnF1(U)`: the point on the first face; `ml.value_on_f1(0.5)?`.
    pub fn value_on_f1(&self, u: f64) -> Result<Point2, Failure> {
        value_on_face(u, &self.my_bis, &self.my_u1, &self.my_v1, self.my_iso_u1)
    }

    /// `ValueOnF2(U)`: the point on the second face; `ml.value_on_f2(0.5)?`.
    pub fn value_on_f2(&self, u: f64) -> Result<Point2, Failure> {
        value_on_face(u, &self.my_bis, &self.my_u2, &self.my_v2, self.my_iso_u2)
    }

    /// `Value(U)`: the 3d point on the first face; `ml.value_3d(0.5)?`.
    pub fn value_3d(&self, u: f64) -> Result<Point3, Failure> {
        on_surface(&self.my_face1, self.value_on_f1(u)?)
    }

    /// `Value3dOnF1OnF2(U, P3d, PF1, PF2)`: the 3d point and the points on both faces;
    /// `let (p3d, pf1, pf2) = ml.value_3d_on_f1_on_f2(0.5)?;`.
    pub fn value_3d_on_f1_on_f2(&self, u: f64) -> Result<(Point3, Point2, Point2), Failure> {
        let pf1: Point2 = self.value_on_f1(u)?;
        let pf2: Point2 = self.value_on_f2(u)?;
        Ok((on_surface(&self.my_face1, pf1)?, pf1, pf2))
    }
}
