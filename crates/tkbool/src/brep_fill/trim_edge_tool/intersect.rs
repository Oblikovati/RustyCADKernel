// Rust translation of BRepFill_TrimEdgeTool::IntersectWith and of the static
// EvalParameters, EvalParametersBis and Bubble
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimEdgeTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersections of the bisector with the two parallels, made coherent: retried
//! with growing tolerances, completed by the nearest end, kept where both parallels
//! cross the bisector at the same parameter.
use super::{BRepFillTrimEdgeTool, pcurve_adaptor};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::{CurveType, JoinType};
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::PI;

/// `Bubble(Seq)`: the points ordered by increasing X (a stable bubble sort).
fn bubble(seq: &mut [Point3]) {
    let mut invert = true;
    while invert {
        invert = false;
        for i in 1..seq.len() {
            if seq[i].x() < seq[i - 1].x() {
                seq.swap(i - 1, i);
                invert = true;
            }
        }
    }
}

/// `EvalParameters(Bis, AC, Params)` with the tolerances `(TolConf, Tol)` (Confusion for
/// both, or the growing tolerance of `EvalParametersBis`): the intersections of the
/// curve and the bisector as (parameter on the bisector, on the curve, 0), ordered on
/// the bisector; a segment covering the bisector gives its two ends, another its middle.
fn eval_parameters(
    bis: &Geom2dAdaptorCurve,
    ac: &Geom2dAdaptorCurve,
    tol: f64,
) -> Result<Vec<Point3>, Failure> {
    let intersector = GInter::between(ac, bis, tol, tol)?;
    if !intersector.is_done() {
        return Err(Failure::with_type(
            "StdFail_NotDone",
            &format!("BRepFill_TrimSurfaceTool::IntersectWith; tolerance {tol}"),
            "BRepFill_TrimEdgeTool",
        ));
    }
    let mut params: Vec<Point3> = Vec::new();
    for i in 1..=intersector.nb_points()? {
        let point = intersector.point(i)?;
        params.push(Point3::new(
            point.param_on_second(),
            point.param_on_first(),
            0.,
        ));
    }
    let (bis_first, bis_last) = (bis.first_parameter()?, bis.last_parameter()?);
    for i in 1..=intersector.nb_segments()? {
        let seg = intersector.segment(i)?;
        let (fp, lp) = (seg.first_point()?, seg.last_point()?);
        let (u1, ulast) = (fp.param_on_second(), lp.param_on_second());
        if (u1 - bis_first).abs() <= tol && (ulast - bis_last).abs() <= tol {
            params.push(Point3::new(u1, fp.param_on_first(), 0.));
            params.push(Point3::new(ulast, lp.param_on_first(), 0.));
        } else {
            params.push(Point3::new(
                (u1 + ulast) / 2.,
                (fp.param_on_first() + lp.param_on_first()) / 2.,
                0.,
            ));
        }
    }
    bubble(&mut params);
    Ok(params)
}

/// The square distance to the line of the bisector of an end of the curve whose
/// projection is inside the bisector range (RealLast for none).
fn end_square_distance(bis: &Geom2dAdaptorCurve, ac: &Geom2dAdaptorCurve) -> Result<f64, Failure> {
    let an_l = bis.line()?;
    let mut d: f64 = f64::MAX;
    for (k, u) in [ac.first_parameter()?, ac.last_parameter()?]
        .into_iter()
        .enumerate()
    {
        let a_p: Point2 = ac.d0(u)?;
        let par: f64 = elclib::line_parameter_2d(an_l.position(), a_p);
        if par >= bis.first_parameter()? && par <= bis.last_parameter()? {
            d = if k == 0 {
                an_l.square_distance(a_p)
            } else {
                an_l.square_distance(a_p).min(d)
            };
        }
    }
    Ok(d)
}

/// The retry tolerance and count: 1e-9 and 7 between lines and circles, else 1e-8 and 6.
fn retry_bounds(ac1: &Geom2dAdaptorCurve, ac2: &Geom2dAdaptorCurve) -> Result<(f64, i32), Failure> {
    let simple = |t: CurveType| t == CurveType::Circle || t == CurveType::Line;
    if !simple(ac1.get_type()?) || !simple(ac2.get_type()?) {
        return Ok((1.0e-8, 6));
    }
    Ok((1.0e-9, 7))
}

/// Whether a line bisector is far from both curves (no intersection at all to look
/// for): the ends within its range are farther than `TolInit * 10^nn`.
fn line_far_from_both(
    bis: &Geom2dAdaptorCurve,
    ac1: &Geom2dAdaptorCurve,
    ac2: &Geom2dAdaptorCurve,
    (tol_init, nn): (f64, i32),
) -> Result<bool, Failure> {
    if bis.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let mut dmax: f64 = tol_init;
    for _ in 0..nn {
        dmax *= 10.0;
    }
    dmax *= dmax;
    Ok(end_square_distance(bis, ac1)? > dmax && end_square_distance(bis, ac2)? > dmax)
}

/// The single intersection of one parallel completed by the nearer end of the other
/// within half the offset (`Params.Length() == 0 && Points2.Length() == 1` and the
/// symmetric case).
fn complete_by_end(
    bis: &Geom2dAdaptorCurve,
    ac: &Geom2dAdaptorCurve,
    t_bis: f64,
    offset: f64,
    target: &mut Vec<Point3>,
) -> Result<(), Failure> {
    let dmax: f64 = 0.25 * offset * offset;
    let p_bis: Point2 = bis.value(t_bis)?;
    let t: f64 = ac.first_parameter()?;
    let dmin: f64 = ac.value(t)?.square_distance(p_bis);
    let mut p = Point3::new(t_bis, t, 0.);
    if dmin < dmax {
        target.push(p);
    }
    let t: f64 = ac.last_parameter()?;
    let dmin1: f64 = ac.value(t)?.square_distance(p_bis);
    if dmin > dmin1 && dmin1 < dmax {
        p.set_y(t);
        match target.first_mut() {
            Some(first) => *first = p,
            None => target.push(p),
        }
    }
    Ok(())
}

/// The common intersections: pairs at the same bisector parameter (within
/// 400 * PConfusion; a gap beyond TolInit is kept, else the smaller is dropped), the
/// longer sequence cut to the shorter.
fn keep_common(params: &mut Vec<Point3>, points2: &mut Vec<Point3>, tol_init: f64) {
    let tol: f64 = 4. * 100. * precision::p_confusion();
    let mut i: usize = 0;
    while i < params.len().min(points2.len()) {
        let (p1, p2) = (params[i], points2[i]);
        let p1x_p2x: f64 = (p1.x() - p2.x()).abs();
        if p1x_p2x <= tol || p1x_p2x > tol_init {
            i += 1;
        } else if p1.x() < p2.x() {
            params.remove(i);
        } else {
            points2.remove(i);
        }
    }
    let n: usize = params.len().min(points2.len());
    params.truncate(n);
    points2.truncate(n);
}

/// The initial parameter range of the first parallel on a circle: the inverse of the
/// extension of its ends in MakeOffset of BRepFill_OffsetWire (an end of an open
/// result is not extended); RealFirst..RealLast otherwise.
fn initial_range(
    init_edge1: &Edge,
    ac1: &Geom2dAdaptorCurve,
    ends: (&Vertex, &Vertex),
    is_open_result: bool,
) -> Result<(f64, f64), Failure> {
    let (mut to_extend_first, mut to_extend_last) = (true, true);
    if is_open_result {
        let (v1, v2) = topexp::vertices(init_edge1, false);
        to_extend_first = !(v1.is_same(ends.0) || v1.is_same(ends.1));
        to_extend_last = !(v2.is_same(ends.0) || v2.is_same(ends.1));
    }
    let ic1 = BRepAdaptorCurve::with_edge(init_edge1)?;
    if ic1.get_type()? != CurveType::Circle {
        return Ok((-f64::MAX, f64::MAX));
    }
    let (ifirst, ilast) = (ic1.first_parameter()?, ic1.last_parameter()?);
    let delta: f64 = 2. * PI - ilast + ifirst;
    let mut init_fpar: f64 = -f64::MAX;
    if to_extend_first && to_extend_last {
        init_fpar = ac1.first_parameter()? + delta / 2.;
    } else if to_extend_first {
        init_fpar = ac1.first_parameter()? + delta;
    } else if to_extend_last {
        init_fpar = ac1.first_parameter()?;
    }
    Ok((init_fpar, init_fpar + ilast - ifirst))
}

/// The intersection pairs kept for the Intersection join: the one of minimal bisector
/// parameter and those inside the initial range on the first parallel.
fn keep_in_range(
    params: &mut Vec<Point3>,
    points2: &mut Vec<Point3>,
    (init_fpar, init_lpar): (f64, f64),
) {
    let mut imin: usize = 0;
    for i in 1..params.len() {
        if params[i].x() < params[imin].x() {
            imin = i;
        }
    }
    let (params_copy, points2_copy) = (std::mem::take(params), std::mem::take(points2));
    for (i, (p, q)) in params_copy.into_iter().zip(points2_copy).enumerate() {
        if imin == i || (p.y() >= init_fpar && p.y() <= init_lpar) {
            params.push(p);
            points2.push(q);
        }
    }
}

/// The parameters with the parameter on the second parallel as Z.
fn merged(params: &[Point3], points2: &[Point3]) -> Sequence<Point3> {
    let mut out: Sequence<Point3> = Sequence::default();
    for (p, q) in params.iter().zip(points2) {
        out.append(Point3::new(p.x(), p.y(), q.y()));
    }
    out
}

impl BRepFillTrimEdgeTool {
    /// `IntersectWith(Edge1, Edge2, InitShape1, InitShape2, End1, End2, theJoinType,
    /// IsOpenResult, Params)`: the intersections of the bisector with both parallels as
    /// (parameter on the bisector, on Edge1, on Edge2), the incoherent ones dropped;
    /// `t.intersect_with(&e1, &e2, &s1, &s2, &v1, &v2, JoinType::Arc, false, &mut params)?`.
    #[allow(clippy::too_many_arguments)] // The native nine-argument signature.
    pub fn intersect_with(
        &self,
        edge1: &Edge,
        edge2: &Edge,
        init_shape1: &Shape,
        init_shape2: &Shape,
        end1: &Vertex,
        end2: &Vertex,
        the_join_type: JoinType,
        is_open_result: bool,
        params: &mut Sequence<Point3>,
    ) -> Result<(), Failure> {
        params.clear(None);
        let (ac1, ..) = pcurve_adaptor(edge1)?;
        let (ac2, ..) = pcurve_adaptor(edge2)?;
        let Some((mut p1, mut p2, tol_init)) = self.coherent_intersections(&ac1, &ac2)? else {
            return Ok(());
        };
        if p1.len() == 1 && p2.len() == 1 {
            *params = merged(&p1, &p2);
            return Ok(());
        }
        keep_common(&mut p1, &mut p2, tol_init);
        if p1.len() > 1 && the_join_type == JoinType::Intersection {
            let mut range: (f64, f64) = (-f64::MAX, f64::MAX);
            if init_shape1.shape_type()? != ShapeEnum::Vertex
                && init_shape2.shape_type()? != ShapeEnum::Vertex
            {
                range = initial_range(
                    &topods::edge(init_shape1)?,
                    &ac1,
                    (end1, end2),
                    is_open_result,
                )?;
            }
            keep_in_range(&mut p1, &mut p2, range);
        }
        *params = merged(&p1, &p2);
        Ok(())
    }

    /// The intersections with both parallels, retried with tolerances growing tenfold
    /// while their counts differ (or both are empty), a single one completed by the
    /// nearer end of the other parallel; `None` when a line bisector is far from both.
    #[allow(clippy::type_complexity)] // Both sequences and the last tolerance.
    fn coherent_intersections(
        &self,
        ac1: &Geom2dAdaptorCurve,
        ac2: &Geom2dAdaptorCurve,
    ) -> Result<Option<(Vec<Point3>, Vec<Point3>, f64)>, Failure> {
        let bis = &self.my_bis;
        let mut p1: Vec<Point3> = eval_parameters(bis, ac1, precision::confusion())?;
        let mut p2: Vec<Point3> = eval_parameters(bis, ac2, precision::confusion())?;
        let (mut tol_init, nn) = retry_bounds(ac1, ac2)?;
        if p1.is_empty() && p2.is_empty() && line_far_from_both(bis, ac1, ac2, (tol_init, nn))? {
            return Ok(None);
        }
        let mut seance_de_rattrapage: i32 = 0;
        while seance_de_rattrapage < nn && (p2.len() != p1.len() || p2.is_empty()) {
            // An incoherent intersection: retried with a greater tolerance.
            tol_init *= 10.0;
            p1 = eval_parameters(bis, ac1, tol_init)?;
            p2 = eval_parameters(bis, ac2, tol_init)?;
            seance_de_rattrapage += 1;
        }
        if p1.is_empty() && p2.len() == 1 {
            complete_by_end(bis, ac1, p2[0].x(), self.my_offset, &mut p1)?;
        } else if p1.len() == 1 && p2.is_empty() {
            complete_by_end(bis, ac2, p1[0].x(), self.my_offset, &mut p2)?;
        }
        Ok(Some((p1, p2, tol_init)))
    }
}
