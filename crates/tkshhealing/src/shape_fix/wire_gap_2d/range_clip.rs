//! The surface-bound clipping of the FixGap2d range branch: a new junction parameter
//! outside the old range whose point leaves the surface bounds moves to the crossing of
//! its pcurve with the bounding line, then never inside the old range.
use super::range_search::{candidates, intersection_point, nearer};
use super::ranges::{RangeGap, adjust_on_periodic};
use super::{Face, Failure, ShapeFixWire};
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::Domain;
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::{Direction2, Point2};

/// `(umin, umax, vmin, vmax)`.
type UvBounds = (f64, f64, f64, f64);

/// Clips the new parameters when one lies outside its old range;
/// `range_clip::clip(&mut state, tool, &face)?`.
pub(super) fn clip(state: &mut RangeGap, tool: &ShapeFixWire, face: &Face) -> Result<(), Failure> {
    if (0..2).all(|i| inside_old_range(state, i)) {
        return Ok(());
    }
    let bounds = surface_bounds(tool, face)?;
    for i in 0..2 {
        if inside_old_range(state, i) {
            continue;
        }
        let mut ipnt = state.curves[i].value(state.ipar[i])?;
        // k = 1 crosses a U bound, k = 2 a V bound.
        for k in 0..2 {
            if let Some(line) = bounding_line(ipnt, bounds, k == 0) {
                cross_bound(state, i, line, &mut ipnt)?;
            }
        }
        keep_outside_old_range(state, i);
    }
    Ok(())
}

// `ipar >= first && ipar <= last` in edge orientation.
fn inside_old_range(state: &RangeGap, i: usize) -> bool {
    let (first, last) = state.bounds[i];
    state.ipar[i] >= first && state.ipar[i] <= last
}

// The native analyzer surface bounds; infinite ones come from the face UV bounds.
fn surface_bounds(tool: &ShapeFixWire, face: &Face) -> Result<UvBounds, Failure> {
    let surface = tool.working_analyzer().surface().surface();
    let surface = surface.get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "FixGap2d: null analyzer surface; expected the face surface",
        )
    })?;
    let (mut umin, mut umax, mut vmin, mut vmax) = surface.bounds();
    let infinite = precision::is_infinite;
    if infinite(umin) || infinite(umax) || infinite(vmin) || infinite(vmax) {
        let (fumin, fumax, fvmin, fvmax) = uv_bounds(face)?;
        let preci = precision::p_confusion();
        let replace = |bound: &mut f64, value: f64| {
            if infinite(*bound) {
                *bound = value;
            }
        };
        replace(&mut umin, fumin - preci);
        replace(&mut umax, fumax + preci);
        replace(&mut vmin, fvmin - preci);
        replace(&mut vmax, fvmax + preci);
    }
    Ok((umin, umax, vmin, vmax))
}

// The bounding line `(P1, P2)` that `point` lies beyond, if any.
fn bounding_line(
    point: Point2,
    (umin, umax, vmin, vmax): UvBounds,
    u_bound: bool,
) -> Option<(Point2, Point2)> {
    let (u, v) = (point.x(), point.y());
    if u_bound {
        if u < umin {
            return Some((Point2::new(umin, vmin), Point2::new(umin, vmax)));
        }
        if u > umax {
            return Some((Point2::new(umax, vmin), Point2::new(umax, vmax)));
        }
        return None;
    }
    if v < vmin {
        return Some((Point2::new(umin, vmin), Point2::new(umax, vmin)));
    }
    if v > vmax {
        return Some((Point2::new(umin, vmax), Point2::new(umax, vmax)));
    }
    None
}

// Intersects the bounding line with the pcurve between its far end and the new
// parameter and takes the crossing nearest to the far end.
fn cross_bound(
    state: &mut RangeGap,
    i: usize,
    (p1, p2): (Point2, Point2),
    ipnt: &mut Point2,
) -> Result<(), Failure> {
    let far = state.far[i];
    let (fpar, lpar) = if far < state.ipar[i] {
        (far, state.ipar[i])
    } else {
        (state.ipar[i], far)
    };
    let curve = state.curves[i].clone();
    let inter = line_crossings((p1, p2), &curve, (fpar, lpar))?;
    if !inter.is_done() || (inter.nb_points()? == 0 && inter.nb_segments()? == 0) {
        return Ok(());
    }
    let tolerance = precision::p_confusion();
    let pick = nearest_crossing(state, i, &candidates(&inter)?, (fpar, lpar))?;
    let point = intersection_point(&inter, pick)?;
    let uu = adjust_on_periodic(
        &curve,
        state.take_first(i),
        (fpar, lpar),
        point.param_on_second(),
    )?;
    if (far - uu).abs() > tolerance {
        state.ipar[i] = uu;
        *ipnt = point.value();
    }
    Ok(())
}

// The bounding line from P1 to P2, on [0, |P1P2|], against the pcurve on [fpar, lpar].
fn line_crossings(
    (p1, p2): (Point2, Point2),
    curve: &Handle<dyn Geom2dCurve>,
    (fpar, lpar): (f64, f64),
) -> Result<GInter, Failure> {
    let tolerance = precision::p_confusion();
    let direction = Direction2::new(p2.x() - p1.x(), p2.y() - p1.y())?;
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p1, direction)).cast();
    let line_domain = Domain::bounded(p1, 0., tolerance, p2, p1.distance(p2), tolerance);
    let (first, last) = (curve.value(fpar)?, curve.value(lpar)?);
    let curve_domain = Domain::bounded(first, fpar, tolerance, last, lpar, tolerance);
    let line_adaptor = Geom2dAdaptorCurve::with_curve(&line)?;
    let curve_adaptor = Geom2dAdaptorCurve::with_curve(curve)?;
    GInter::with_domains(
        &line_adaptor,
        &line_domain,
        &curve_adaptor,
        &curve_domain,
        tolerance,
        tolerance,
    )
}

// The native choice, which for segment end points reads the flags and far ends by the
// end index (`jj == 1 ? reversed1 : !reversed2`, `jj == 1 ? cfirst1 : clast2`) instead
// of by the curve; the quirk is kept.
fn nearest_crossing(
    state: &RangeGap,
    i: usize,
    candidates: &[((i32, i32), IntersectionPoint)],
    range: (f64, f64),
) -> Result<(i32, i32), Failure> {
    // Natively the pick starts at point 1, kept even when no candidate is compared.
    let mut best = ((1, 0), -1.);
    for &(pick, ip) in candidates {
        // A point reads its own curve; segment end 1 reads curve 1, end 2 curve 2.
        let side = if pick.1 == 0 {
            i
        } else {
            (pick.1 - 1) as usize
        };
        let uu = adjust_on_periodic(
            &state.curves[i],
            state.take_first(side),
            range,
            ip.param_on_second(),
        )?;
        nearer(&mut best, pick, (state.far[side] - uu).abs());
    }
    Ok(best.0)
}

// "Adjust if intersection lies inside old bounds".
fn keep_outside_old_range(state: &mut RangeGap, i: usize) {
    let (first, last) = state.bounds[i];
    let ipar = &mut state.ipar[i];
    match (i, state.reversed[i]) {
        (0, true) if *ipar > first => *ipar = first,
        (0, false) if *ipar < last => *ipar = last,
        (1, true) if *ipar < last => *ipar = last,
        (1, false) if *ipar > first => *ipar = first,
        _ => {}
    }
}
