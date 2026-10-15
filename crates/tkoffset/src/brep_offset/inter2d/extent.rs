// Rust translation of BRepOffset_Inter2d::ExtentEdge with the static evaluateMaxSegment
// and ExtendPCurve of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Inter2d.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extension of an offset edge: its p-curves extended (or their range widened) up
//! to twice the offset beyond its ends within the bounds of their surfaces, and its 3d
//! curve rebuilt or ranged accordingly.
use super::BRepOffsetInter2d;
use crate::brep_offset::tool::live;
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepCurveRepresentation, BRepGCurve, BRepTool};
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dLine,
    Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomBoundedCurve, GeomCurve, GeomLine, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkgeombase::{geom_convert, geom2d_convert};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Vector2, Vector3};
use rustycad_tktopalgo::brep_lib;

/// `evaluateMaxSegment(aCurveOnSurface)`: 30 plus the greater knot count of the B-spline
/// surface and the B-spline p-curve.
fn evaluate_max_segment(a_curve_on_surface: &Adaptor3dCurveOnSurface) -> Result<i32, Failure> {
    let a_surf = a_curve_on_surface.get_surface();
    let a_curv2d = a_curve_on_surface.get_curve();
    let (mut a_nb_s_knots, mut a_nb_c2d_knots) = (0., 0.);
    let surf = live(&a_surf, "surface")?;
    if surf.get_type()? == SurfaceType::BSplineSurface {
        let a_bspline = surf.bspline()?;
        let b = live(&a_bspline, "B-spline surface")?;
        a_nb_s_knots = f64::from(b.nb_u_knots().max(b.nb_v_knots()));
    }
    let curv = live(&a_curv2d, "pcurve")?;
    if curv.get_type()? == CurveType::BSplineCurve {
        a_nb_c2d_knots = f64::from(curv.nb_knots()?);
    }
    Ok((30. + a_nb_s_knots.max(a_nb_c2d_knots)) as i32)
}

/// A 2d line from the first pole of a two-pole curve to the second.
fn pole_line(p1: Point2, p2: Point2) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let d = Direction2::from_vector(Vector2::between(p1, p2))?;
    Ok(Handle::new(Geom2dLine::from_point(p1, d)).cast())
}

/// `ExtendPCurve(aPCurve, anEf, anEl, a2Offset, NewPCurve)`: a straight two-pole
/// Bezier or B-spline p-curve made a line, any other one prolonged by tangent segments
/// of max(2 offset, 1) at each end not reaching twice the offset beyond the edge;
/// `None` when a segment does not join.
fn extend_pcurve(
    a_pcurve: &Handle<dyn Geom2dCurve>,
    an_ef: f64,
    an_el: f64,
    a2_offset: f64,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let mut new_pcurve = a_pcurve.clone();
    if new_pcurve.is_instance_name("Geom2d_TrimmedCurve") {
        new_pcurve =
            live(&new_pcurve.clone().cast::<Geom2dTrimmedCurve>(), "pcurve")?.basis_curve();
    }
    let npc = live(&new_pcurve, "pcurve")?;
    let (first_par, last_par) = (npc.first_parameter(), npc.last_parameter());
    if new_pcurve.is_kind_name("Geom2d_BoundedCurve")
        && (first_par > an_ef - a2_offset || last_par < an_el + a2_offset)
    {
        if new_pcurve.is_instance_name("Geom2d_BezierCurve") {
            let a_bezier = new_pcurve.clone().cast::<Geom2dBezierCurve>();
            let b = live(&a_bezier, "Bezier")?;
            if b.nb_poles() == 2 {
                let poles = b.poles();
                return Ok(Some(pole_line(poles[1], poles[2])?));
            }
        } else if new_pcurve.is_instance_name("Geom2d_BSplineCurve") {
            let a_bspline = new_pcurve.clone().cast::<Geom2dBSplineCurve>();
            let b = live(&a_bspline, "B-spline")?;
            if b.nb_knots() == 2 && b.nb_poles() == 2 {
                let poles = b.poles();
                return Ok(Some(pole_line(poles[1], poles[2])?));
            }
        }
    }
    let apc = live(a_pcurve, "pcurve")?;
    let (first_par, last_par) = (apc.first_parameter(), apc.last_parameter());
    let a_tr_curve: Handle<dyn Geom2dBoundedCurve> = Handle::new(Geom2dTrimmedCurve::new(
        a_pcurve, first_par, last_par, true, true,
    )?)
    .cast();
    // The curve is not prolonged at its beginning or end: a segment is added there.
    let mut a_comp_curve = geom2d_convert::CompCurveToBSplineCurve::with_curve(
        &a_tr_curve,
        ParameterisationType::RationalC1,
    )?;
    let a_tol = precision::confusion();
    let a_delta = a2_offset.max(1.);
    let segment = |p: Point2, d: Direction2| -> Result<Handle<dyn Geom2dBoundedCurve>, Failure> {
        let a_lin: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p, d)).cast();
        Ok(Handle::new(Geom2dTrimmedCurve::new(&a_lin, 0., a_delta, true, true)?).cast())
    };
    if first_par > an_ef - a2_offset {
        let (a_p_bnd, a_v_bnd) = apc.d1(first_par)?;
        let a_d_bnd = Direction2::from_xy(a_v_bnd.xy())?;
        let a_p_beg = a_p_bnd.translated(Vector2::from(a_d_bnd.xy() * -a_delta));
        if !a_comp_curve.add(&segment(a_p_beg, a_d_bnd)?, a_tol, false)? {
            return Ok(None);
        }
    }
    if last_par < an_el + a2_offset {
        let (a_p_beg, a_v_bnd) = apc.d1(last_par)?;
        let a_d_bnd = Direction2::from_xy(a_v_bnd.xy())?;
        if !a_comp_curve.add(&segment(a_p_beg, a_d_bnd)?, a_tol, false)? {
            return Ok(None);
        }
    }
    Ok(Some(a_comp_curve.bspline_curve().cast()))
}

/// The extended range of a p-curve representation (its curve extended when bounded
/// and short, its range widened when periodic, shortened when closed), cut at the
/// bounds of its surface.
fn extended_pcurve_range(
    curve_rep: &dyn BRepCurveRepresentation,
    an_ef: f64,
    an_el: f64,
    a2_offset: f64,
) -> Result<(f64, f64), Failure> {
    let the_curve = curve_rep.pcurve()?;
    let tc = live(&the_curve, "pcurve")?;
    let (mut first_par, mut last_par) = (tc.first_parameter(), tc.last_parameter());
    if the_curve.is_kind_name("Geom2d_BoundedCurve")
        && (first_par > an_ef - a2_offset || last_par < an_el + a2_offset)
    {
        if let Some(new_pcurve) = extend_pcurve(&the_curve, an_ef, an_el, a2_offset)? {
            curve_rep.set_pcurve(&new_pcurve)?;
            let n = live(&new_pcurve, "pcurve")?;
            (first_par, last_par) = (n.first_parameter(), n.last_parameter());
            if curve_rep.is_curve_on_closed_surface() {
                let pcurve2 = curve_rep.pcurve2()?;
                if let Some(np2) = extend_pcurve(&pcurve2, an_ef, an_el, a2_offset)? {
                    curve_rep.set_pcurve2(&np2)?;
                }
            }
        }
    } else if tc.is_periodic() {
        let delta = (tc.period()? - (an_el - an_ef)) * 0.5 * 0.95;
        first_par = an_ef - delta;
        last_par = an_el + delta;
    } else if tc.is_closed()? {
        last_par -= 0.05 * (last_par - first_par);
    }
    // FirstPar and LastPar: the p-curve should stay in its surface.
    let the_curve = curve_rep.pcurve()?;
    let the_surf = curve_rep.surface()?;
    let (umin, umax, vmin, vmax) = live(&the_surf, "surface")?.bounds();
    let mut bound_lines: Vec<Handle<dyn Geom2dCurve>> = Vec::new();
    let line = |x: f64, y: f64, d: Direction2| -> Handle<dyn Geom2dCurve> {
        Handle::new(Geom2dLine::from_point(Point2::new(x, y), d)).cast()
    };
    if !precision::is_infinite(vmin) {
        bound_lines.push(line(0., vmin, Direction2::X));
    }
    if !precision::is_infinite(umin) {
        bound_lines.push(line(umin, 0., Direction2::Y));
    }
    if !precision::is_infinite(vmax) {
        bound_lines.push(line(0., vmax, Direction2::X));
    }
    if !precision::is_infinite(umax) {
        bound_lines.push(line(umax, 0., Direction2::Y));
    }
    let inside = |p: Point2| p.x() >= umin && p.x() <= umax && p.y() >= vmin && p.y() <= vmax;
    let mut params = Vec::new();
    let ga_curve = Geom2dAdaptorCurve::with_curve(&the_curve)?;
    for bl in &bound_lines {
        let ga_line = Geom2dAdaptorCurve::with_curve(bl)?;
        let int_cc = GInter::between(
            &ga_curve,
            &ga_line,
            precision::p_confusion(),
            precision::p_confusion(),
        )?;
        if !int_cc.is_done() {
            continue;
        }
        for j in 1..=int_cc.nb_points()? {
            let ip = int_cc.point(j)?;
            if inside(ip.value()) {
                params.push(ip.param_on_first());
            }
        }
        for j in 1..=int_cc.nb_segments()? {
            let is = int_cc.segment(j)?;
            if is.has_first_point() {
                let ip = is.first_point()?;
                if inside(ip.value()) {
                    params.push(ip.param_on_first());
                }
            }
            if is.has_last_point() {
                let ip = is.last_point()?;
                if inside(ip.value()) {
                    params.push(ip.param_on_first());
                }
            }
        }
    }
    if params.len() == 1 {
        let pnt_first = live(&the_curve, "pcurve")?.value(first_par)?;
        if inside(pnt_first) {
            if last_par > params[0] {
                last_par = params[0];
            }
        } else if first_par < params[0] {
            first_par = params[0];
        }
    } else if !params.is_empty() {
        let (mut fpar, mut lpar) = (f64::MAX, f64::MIN);
        for &p in &params {
            if p < fpar {
                fpar = p;
            }
            if p > lpar {
                lpar = p;
            }
        }
        if first_par < fpar {
            first_par = fpar;
        }
        if last_par > lpar {
            last_par = lpar;
        }
    }
    Ok((first_par, last_par))
}

/// The p-curve with the narrowest range on the edge (`MinPC`, `MinSurf`, `MinLoc`).
struct MinPCurve {
    pc: Handle<dyn Geom2dCurve>,
    surf: Handle<dyn GeomSurface>,
    loc: rustycad_tkmath::toploc::Location,
}

impl BRepOffsetInter2d {
    /// `ExtentEdge(E, NE, theOffset)`: the empty copy of the edge with its p-curves
    /// extended twice the offset beyond its ends (cut at the bounds of their surfaces)
    /// and its 3d curve ranged on them (built from the narrowest p-curve when missing),
    /// or, without p-curve, its 3d curve prolonged; false when the p-curves cannot be
    /// projected again on a closed range; `BRepOffsetInter2d::extent_edge(&e, &mut ne, 1.0)?`.
    pub fn extent_edge(e: &Edge, ne: &mut Edge, the_offset: f64) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        let a2_offset = 2. * the_offset.abs();
        let (an_ef, an_el) = BRepTool::range(e)?;
        *ne = topods::edge(&e.empty_copied()?)?;
        // Enough for analytic edges; in general the geometry of the edge would be
        // rebuilt from the intersection of the surfaces.
        let mut nb_pcurves = 0;
        let (mut first_par_on_pc, mut last_par_on_pc) = (f64::MIN, f64::MAX);
        let mut min: Option<MinPCurve> = None;
        let tedge = edge_tshape(ne.as_ref(), "BRepOffset_Inter2d::ExtentEdge")?;
        for curve_rep in tedge.curves() {
            let rep = live(&curve_rep, "curve representation")?;
            if !rep.is_curve_on_surface() {
                continue;
            }
            nb_pcurves += 1;
            let (first_par, last_par) = extended_pcurve_range(rep, an_ef, an_el, a2_offset)?;
            let gc = curve_rep.clone().cast::<dyn BRepGCurve>();
            live(&gc, "GCurve")?.set_range(first_par, last_par)?;
            // FirstParOnPC and LastParOnPC.
            let current = || -> Result<MinPCurve, Failure> {
                Ok(MinPCurve {
                    pc: rep.pcurve()?,
                    surf: rep.surface()?,
                    loc: rep.location(),
                })
            };
            if first_par > first_par_on_pc {
                first_par_on_pc = first_par;
                min = Some(current()?);
            }
            if last_par < last_par_on_pc {
                last_par_on_pc = last_par;
                min = Some(current()?);
            }
        }
        let (mut c3d, f, l) = BRepTool::curve(ne)?;
        if nb_pcurves == 0 {
            extend_curve3d(ne, &c3d, (an_ef, an_el), a2_offset)?;
            return Ok(true);
        }
        let min = min.ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "BRepOffset_Inter2d::ExtentEdge: no p-curve",
            )
        })?;
        let min_loc = (e.location() * &min.loc)?;
        if !c3d.is_null() {
            let (f, l) = c3d_range(
                &c3d,
                &min,
                &min_loc,
                (f, l),
                (first_par_on_pc, last_par_on_pc),
            )?;
            builder.range(ne, f, l, false)?;
            if !precision::is_infinite(f) && !precision::is_infinite(l) {
                brep_lib::same_parameter_shape(ne.as_ref(), precision::confusion(), true)?;
            }
            return Ok(true);
        }
        if BRepTool::degenerated(e)? {
            return Ok(true);
        }
        // No 3d curve.
        let min_surf: Handle<dyn GeomSurface> = live(&min.surf, "surface")?
            .transformed(min_loc.transformation())?
            .cast();
        let mut max_deviation = 0.;
        if precision::is_infinite(first_par_on_pc) || precision::is_infinite(last_par_on_pc) {
            if min.pc.is_instance_name("Geom2d_Line") {
                let mut is_line = false;
                if min_surf.is_instance_name("Geom_Plane") {
                    is_line = true;
                } else if min_surf.is_instance_name("Geom_CylindricalSurface")
                    || min_surf.is_instance_name("Geom_ConicalSurface")
                {
                    let the_line = min.pc.clone().cast::<Geom2dLine>();
                    if live(&the_line, "line")?
                        .direction()
                        .is_parallel(Direction2::Y, precision::angular())
                    {
                        is_line = true;
                    }
                }
                if is_line {
                    let pc = live(&min.pc, "pcurve")?;
                    let (p2d1, p2d2) = (pc.value(0.)?, pc.value(1.)?);
                    let s = live(&min_surf, "surface")?;
                    let p1 = s.value(p2d1.x(), p2d1.y())?;
                    let p2 = s.value(p2d2.x(), p2d2.y())?;
                    let d = Direction3::from_vector(Vector3::between(p1, p2))?;
                    c3d = Handle::new(GeomLine::from_point(p1, d)).cast();
                }
            }
        } else {
            let hc2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_range(
                &min.pc,
                first_par_on_pc,
                last_par_on_pc,
            )?)
            .cast();
            let hsurf: Handle<dyn Adaptor3dSurface> =
                Handle::new(GeomAdaptorSurface::with_surface(&min_surf)?).cast();
            let con_s = Adaptor3dCurveOnSurface::with_curve(&hc2d, &hsurf)?;
            let max_segment = evaluate_max_segment(&con_s)?;
            (max_deviation, _) = geom_lib::build_curve3d(
                precision::confusion(),
                &con_s,
                (first_par_on_pc, last_par_on_pc),
                &mut c3d,
                Continuity::C1,
                14,
                max_segment,
            )?;
        }
        builder.update_edge_curve(ne, &c3d, max_deviation)?;
        let mut projection_success = true;
        if nb_pcurves > 1 {
            for curve_rep in tedge.curves() {
                let rep = live(&curve_rep, "curve representation")?;
                if !rep.is_curve_on_surface() {
                    continue;
                }
                let the_curve = rep.pcurve()?;
                let mut the_surf = rep.surface()?;
                let mut the_loc = rep.location();
                if the_curve == min.pc && the_surf == min_surf && the_loc == min_loc {
                    continue;
                }
                let gc = curve_rep.clone().cast::<dyn BRepGCurve>();
                let g = live(&gc, "GCurve")?;
                let (first_par, last_par) = (g.first(), g.last());
                if (first_par - first_par_on_pc).abs() <= precision::p_confusion()
                    && (last_par - last_par_on_pc).abs() <= precision::p_confusion()
                {
                    continue;
                }
                the_loc = (e.location() * &the_loc)?;
                the_surf = live(&the_surf, "surface")?
                    .transformed(the_loc.transformation())?
                    .cast();
                if the_curve.is_instance_name("Geom2d_Line")
                    && the_surf.is_kind_name("Geom_BoundedSurface")
                {
                    let the_line = the_curve.clone().cast::<Geom2dLine>();
                    let tl = live(&the_line, "line")?;
                    let the_dir = tl.direction();
                    if the_dir.is_parallel(Direction2::X, precision::angular())
                        || the_dir.is_parallel(Direction2::Y, precision::angular())
                    {
                        let (u1, u2, v1, v2) = live(&the_surf, "surface")?.bounds();
                        let origin = tl.location();
                        if (origin.x() - u1).abs() <= precision::confusion()
                            || (origin.x() - u2).abs() <= precision::confusion()
                            || (origin.y() - v1).abs() <= precision::confusion()
                            || (origin.y() - v2).abs() <= precision::confusion()
                        {
                            brep_lib::same_parameter_shape(
                                ne.as_ref(),
                                precision::confusion(),
                                true,
                            )?;
                            break;
                        }
                    }
                }
                if !c3d.is_null() && first_par_on_pc < last_par_on_pc {
                    let proj_pcurve = geom_proj_lib::curve2d_range(
                        &c3d,
                        (first_par_on_pc, last_par_on_pc),
                        &the_surf,
                    )?;
                    if proj_pcurve.is_null() {
                        projection_success = false;
                    } else {
                        rep.set_pcurve(&proj_pcurve)?;
                    }
                } else {
                    return Ok(false);
                }
            }
        }
        if projection_success {
            builder.range(ne, first_par_on_pc, last_par_on_pc, false)?;
        } else {
            builder.range(ne, first_par_on_pc, last_par_on_pc, true)?;
            brep_lib::same_parameter_shape(ne.as_ref(), precision::confusion(), true)?;
        }
        Ok(true)
    }
}

/// The range of the 3d curve of the extended edge: the p-curve range for a closed
/// narrowest p-curve, widened when periodic, shortened when closed, else the
/// projections of the ends of the narrowest p-curve.
fn c3d_range(
    c3d: &Handle<dyn GeomCurve>,
    min: &MinPCurve,
    min_loc: &rustycad_tkmath::toploc::Location,
    (mut f, mut l): (f64, f64),
    (first_par_on_pc, last_par_on_pc): (f64, f64),
) -> Result<(f64, f64), Failure> {
    let c = live(c3d, "curve")?;
    if live(&min.pc, "pcurve")?.is_closed()? {
        return Ok((first_par_on_pc, last_par_on_pc));
    }
    if c.is_periodic() {
        let delta = (c.period()? - (l - f)) * 0.5 * 0.95;
        return Ok((f - delta, l + delta));
    }
    if c.is_closed()? {
        l -= 0.05 * (l - f);
        return Ok((f, l));
    }
    f = first_par_on_pc;
    l = last_par_on_pc;
    let a_min_loc_trsf = min_loc.transformation();
    let mut projector = ProjectPointOnCurve::new();
    let pc = live(&min.pc, "pcurve")?;
    let s = live(&min.surf, "surface")?;
    if !precision::is_infinite(first_par_on_pc) {
        let p2d1 = pc.value(first_par_on_pc)?;
        let p1 = s.value(p2d1.x(), p2d1.y())?.transformed(a_min_loc_trsf);
        projector.init(p1, c3d)?;
        if projector.nb_points() > 0 {
            f = projector.lower_distance_parameter()?;
        }
    }
    if !precision::is_infinite(last_par_on_pc) {
        let p2d2 = pc.value(last_par_on_pc)?;
        let p2 = s.value(p2d2.x(), p2d2.y())?.transformed(a_min_loc_trsf);
        projector.init(p2, c3d)?;
        if projector.nb_points() > 0 {
            l = projector.lower_distance_parameter()?;
        }
    }
    Ok((f, l))
}

/// The 3d curve of an edge without p-curve prolonged (bounded and short: tangent
/// segments; periodic: widened range; closed: shortened range).
fn extend_curve3d(
    ne: &Edge,
    c3d: &Handle<dyn GeomCurve>,
    (an_ef, an_el): (f64, f64),
    a2_offset: f64,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let c = live(c3d, "curve")?;
    let (mut first_par, mut last_par) = (c.first_parameter(), c.last_parameter());
    if c3d.is_kind_name("Geom_BoundedCurve")
        && (first_par > an_ef - a2_offset || last_par < an_el + a2_offset)
    {
        let a_tr_curve: Handle<dyn GeomBoundedCurve> =
            Handle::new(GeomTrimmedCurve::new(c3d, first_par, last_par, true, true)?).cast();
        // The curve is not prolonged at its beginning or end: a segment is added there.
        let mut a_comp_curve = geom_convert::CompCurveToBSplineCurve::with_curve(
            &a_tr_curve,
            ParameterisationType::RationalC1,
        )?;
        let a_tol = precision::confusion();
        let a_delta = a2_offset.max(1.);
        let segment = |p, d| -> Result<Handle<dyn GeomBoundedCurve>, Failure> {
            let a_lin: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(p, d)).cast();
            Ok(Handle::new(GeomTrimmedCurve::new(&a_lin, 0., a_delta, true, true)?).cast())
        };
        if first_par > an_ef - a2_offset {
            let (a_p_bnd, a_v_bnd) = c.d1(first_par)?;
            let a_d_bnd = Direction3::from_xyz(*a_v_bnd.xyz())?;
            let a_p_beg = a_p_bnd.translated(Vector3::from(a_d_bnd.xyz() * -a_delta));
            if !a_comp_curve.add(&segment(a_p_beg, a_d_bnd)?, a_tol, false, true, 0)? {
                return Ok(());
            }
        }
        if last_par < an_el + a2_offset {
            let (a_p_beg, a_v_bnd) = c.d1(last_par)?;
            let a_d_bnd = Direction3::from_xyz(*a_v_bnd.xyz())?;
            if !a_comp_curve.add(&segment(a_p_beg, a_d_bnd)?, a_tol, false, true, 0)? {
                return Ok(());
            }
        }
        let new_c3d: Handle<dyn GeomCurve> = a_comp_curve.bspline_curve().cast();
        let n = live(&new_c3d, "curve")?;
        (first_par, last_par) = (n.first_parameter(), n.last_parameter());
        builder.update_edge_curve(ne, &new_c3d, precision::confusion())?;
    } else if c.is_periodic() {
        let delta = (c.period()? - (an_el - an_ef)) * 0.5 * 0.95;
        first_par = an_ef - delta;
        last_par = an_el + delta;
    } else if c.is_closed()? {
        last_par -= 0.05 * (last_par - first_par);
    }
    builder.range(ne, first_par, last_par, false)
}
