// Rust translation of DBRep_IsoBuilder::FillGaps of
// Draw/TKDraw/DBRep/DBRep_IsoBuilder.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DBRep_IsoBuilder::FillGaps`: closes the 2D gaps between connected p-curves that the
//! vertex tolerances close in 3D, or trims p-curves that cross near the vertex.
use super::iso_builder::{DBRepIsoBuilder, EdgePCurves};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::{first_vertex, last_vertex};
use rustycad_tkbrep::topods::{Edge, Face, ShapeIterator, wire as to_wire};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::{Direction2, Point2, Point3, Vector2};

/// The end of a p-curve at a common vertex: the curve, its range and the vertex
/// parameter clamped to it.
#[derive(Clone)]
struct CurveEnd {
    curve: Handle<dyn Geom2dCurve>,
    range: (f64, f64),
    at: f64,
}

/// The vertex shared by two consecutive edges.
struct SharedVertex {
    point: Point3,
    /// `aTolV2`: the squared tolerance, or the tolerance itself when it is infinite.
    tolerance: f64,
    /// Whether the tolerance (before squaring) is below `Precision::Infinite()`.
    finite: bool,
}

impl DBRepIsoBuilder {
    /// `FillGaps`: for each wire of `face`, each pair of consecutive edges (the last
    /// with the first) sharing a vertex whose tolerance covers both p-curve ends gets
    /// a 2D segment between the ends, unless the p-curves intersect near the vertex,
    /// in which case they are trimmed at the intersection instead (called by
    /// [`DBRepIsoBuilder::new`]; protected in C++).
    pub fn fill_gaps(&mut self, face: &Face, pcurves: &mut EdgePCurves) -> Result<(), Failure> {
        let surface = BRepAdaptorSurface::with_face(face, false)?;
        let mut wires = ShapeIterator::new(face, true, true);
        while wires.more() {
            let shape = wires.value()?.clone();
            wires.next();
            if shape.shape_type()? != ShapeEnum::Wire {
                continue;
            }
            let single_edge = shape.nb_children() == 1;
            let mut explorer = WireExplorer::new();
            explorer.init_with_bounds(&to_wire(&shape)?, face, self.bounds)?;
            self.fill_wire_gaps(&mut explorer, (&surface, single_edge), pcurves)?;
        }
        Ok(())
    }

    /// The edge pairs of one wire, in wire order, closing with (last, first).
    fn fill_wire_gaps(
        &mut self,
        explorer: &mut WireExplorer,
        (surface, single_edge): (&BRepAdaptorSurface, bool),
        pcurves: &mut EdgePCurves,
    ) -> Result<(), Failure> {
        if !explorer.more() {
            return Ok(());
        }
        let first_edge = explorer.current().clone();
        let mut current = first_edge.clone();
        let mut stop = false;
        while !stop {
            let previous = current.clone();
            explorer.next()?;
            if explorer.more() {
                current = explorer.current().clone();
            } else {
                current = first_edge.clone();
                stop = true;
            }
            if previous.is_equal(&current) && !single_edge {
                continue;
            }
            self.fill_pair_gap(surface, (&previous, &current), pcurves)?;
        }
        Ok(())
    }

    /// One pair of consecutive edges.
    fn fill_pair_gap(
        &mut self,
        surface: &BRepAdaptorSurface,
        (previous, current): (&Edge, &Edge),
        pcurves: &mut EdgePCurves,
    ) -> Result<(), Failure> {
        let (Some(prev_slot), Some(curr_slot)) = (slot(pcurves, previous), slot(pcurves, current))
        else {
            return Ok(());
        };
        let Some((shared, mut prev_end, mut curr_end)) =
            shared_ends(pcurves, (prev_slot, curr_slot), (previous, current))?
        else {
            return Ok(());
        };
        let prev_point = curve_value(&prev_end.curve, prev_end.at)?;
        let curr_point = curve_value(&curr_end.curve, curr_end.at)?;
        if !covers_gap(surface, &shared, (prev_point, curr_point))? {
            return Ok(());
        }
        let segment = Vector2::between(prev_point, curr_point);
        let length = segment.magnitude();
        let mut add_segment = length > precision::p_confusion() && fits_periods(surface, length)?;
        if add_segment && !previous.is_equal(current) {
            let trimmed = trim_at_crossing(
                surface,
                &shared,
                (previous, current),
                (&mut prev_end, &mut curr_end),
            )?;
            if trimmed {
                pcurves[prev_slot].1 = prev_end.curve.clone();
                pcurves[curr_slot].1 = curr_end.curve.clone();
                add_segment = false;
            }
        }
        if add_segment {
            self.add_gap_segment(prev_point, segment, length)?;
        }
        Ok(())
    }

    /// The segment `Geom2d_TrimmedCurve(Geom2d_Line(P, V), 0, |V|)` as a FORWARD element.
    fn add_gap_segment(
        &mut self,
        start: Point2,
        segment: Vector2,
        length: f64,
    ) -> Result<(), Failure> {
        let line = Geom2dLine::from_point(start, Direction2::from_vector(segment)?);
        let line = Handle::new(line).cast::<dyn Geom2dCurve>();
        let trimmed = Geom2dTrimmedCurve::new(&line, 0.0, length, true, true)?;
        let curve =
            Geom2dAdaptorCurve::with_curve(&Handle::new(trimmed).cast::<dyn Geom2dCurve>())?;
        self.hatcher.add_element(curve, Orientation::Forward);
        Ok(())
    }
}

/// The index of `edge` in the map (`ChangeSeek`, keyed with `IsEqual`).
fn slot(pcurves: &EdgePCurves, edge: &Edge) -> Option<usize> {
    pcurves.iter().position(|(key, _)| key.is_equal(edge))
}

/// `C->Value(t)`.
fn curve_value(curve: &Handle<dyn Geom2dCurve>, t: f64) -> Result<Point2, Failure> {
    curve
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "p-curve"))?
        .value(t)
}

/// The shared vertex (last of `previous`, first of `current`, both with the edge
/// orientation) and the clamped p-curve ends there; `None` when the vertices are
/// missing or differ.
fn shared_ends(
    pcurves: &EdgePCurves,
    (prev_slot, curr_slot): (usize, usize),
    (previous, current): (&Edge, &Edge),
) -> Result<Option<(SharedVertex, CurveEnd, CurveEnd)>, Failure> {
    let on_previous = last_vertex(previous, true);
    if on_previous.is_null() {
        return Ok(None);
    }
    let prev_end = curve_end(
        &pcurves[prev_slot].1,
        BRepTool::parameter(&on_previous, previous)?,
    )?;
    let on_current = first_vertex(current, true);
    if on_current.is_null() || !on_previous.is_same(&on_current) {
        return Ok(None);
    }
    let curr_end = curve_end(
        &pcurves[curr_slot].1,
        BRepTool::parameter(&on_current, current)?,
    )?;
    let tolerance = BRepTool::vertex_tolerance(&on_previous)?;
    let point = BRepTool::pnt(&on_previous)?;
    let finite = tolerance < precision::infinite();
    let tolerance = if finite {
        tolerance * tolerance
    } else {
        tolerance
    };
    let shared = SharedVertex {
        point,
        tolerance,
        finite,
    };
    Ok(Some((shared, prev_end, curr_end)))
}

/// The end of `curve` at parameter `t` clamped to its range.
fn curve_end(curve: &Handle<dyn Geom2dCurve>, t: f64) -> Result<CurveEnd, Failure> {
    let geometry = curve
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "p-curve"))?;
    let range = (geometry.first_parameter(), geometry.last_parameter());
    let at = if t < range.0 {
        range.0
    } else if t > range.1 {
        range.1
    } else {
        t
    };
    Ok(CurveEnd {
        curve: curve.clone(),
        range,
        at,
    })
}

/// Whether the vertex tolerance covers both ends in 3D (always for an infinite
/// tolerance).
fn covers_gap(
    surface: &BRepAdaptorSurface,
    shared: &SharedVertex,
    (prev_point, curr_point): (Point2, Point2),
) -> Result<bool, Failure> {
    if !shared.finite {
        return Ok(true);
    }
    for point in [prev_point, curr_point] {
        let on_surface = surface.value(point.x(), point.y())?;
        if shared.point.square_distance(&on_surface) > shared.tolerance {
            return Ok(false);
        }
    }
    Ok(true)
}

/// A segment on a periodic surface must be shorter than a quarter period.
fn fits_periods(surface: &BRepAdaptorSurface, length: f64) -> Result<bool, Failure> {
    // `bAddSegment = aSegmLen < Period / 4.`: false for NaN, as in C++.
    let shorter = |period: f64| length < period / 4.;
    if surface.is_u_periodic()? && !shorter(surface.u_period()?) {
        return Ok(false);
    }
    if surface.is_v_periodic()? && !shorter(surface.v_period()?) {
        return Ok(false);
    }
    Ok(true)
}

/// Intersects the two p-curves on their ranges; when they cross near the shared
/// vertex (and the crossing is within its tolerance), trims both there and returns
/// true.
fn trim_at_crossing(
    surface: &BRepAdaptorSurface,
    shared: &SharedVertex,
    (previous, current): (&Edge, &Edge),
    (prev_end, curr_end): (&mut CurveEnd, &mut CurveEnd),
) -> Result<bool, Failure> {
    let Some((t_prev, t_curr)) = closest_crossing(prev_end, curr_end)? else {
        return Ok(false);
    };
    let common = common_vertex_count(previous, current)?;
    let near_prev = (prev_end.at - t_prev).abs() < (prev_end.range.1 - prev_end.range.0) / 2.;
    let near_curr = (curr_end.at - t_curr).abs() < (curr_end.range.1 - curr_end.range.0) / 2.;
    if !(common == 1 || near_prev || near_curr) {
        return Ok(false);
    }
    let crossing = curve_value(&prev_end.curve, t_prev)?;
    let on_surface = surface.value(crossing.x(), crossing.y())?;
    // Native quirk kept: the tolerance was squared only when finite, and an infinite
    // one is compared with `>` (never true for exactly Precision::Infinite()).
    if !(shared.tolerance > precision::infinite()
        || on_surface.square_distance(&shared.point) < shared.tolerance)
    {
        return Ok(false);
    }
    trim_end(prev_end, t_prev)?;
    trim_end(curr_end, t_curr)?;
    Ok(true)
}

/// Trims `end` from the crossing `t` to the end away from the vertex, unless the
/// remaining range is shorter than the parametric confusion.
fn trim_end(end: &mut CurveEnd, t: f64) -> Result<(), Failure> {
    let (first, last) = end.range;
    let (f, l) = if (first - end.at).abs() < (last - end.at).abs() {
        (t, last)
    } else {
        (first, t)
    };
    if l - f > precision::p_confusion() {
        let trimmed = Geom2dTrimmedCurve::new(&end.curve, f, l, true, true)?;
        end.curve = Handle::new(trimmed).cast::<dyn Geom2dCurve>();
    }
    Ok(())
}

/// The intersection point of the two p-curves (segment ends and points, inside both
/// ranges) closest to the vertex parameters: `(t on previous, t on current)`.
fn closest_crossing(
    prev_end: &CurveEnd,
    curr_end: &CurveEnd,
) -> Result<Option<(f64, f64)>, Failure> {
    let tolerance = precision::p_confusion();
    let first =
        Geom2dAdaptorCurve::with_range(&prev_end.curve, prev_end.range.0, prev_end.range.1)?;
    let second =
        Geom2dAdaptorCurve::with_range(&curr_end.curve, curr_end.range.0, curr_end.range.1)?;
    let intersection = GInter::between(&first, &second, tolerance, tolerance)?;
    if !intersection.is_done() || intersection.is_empty()? {
        return Ok(None);
    }
    let mut points: Vec<IntersectionPoint> = Vec::new();
    for i in 1..=intersection.nb_segments()? {
        let segment = intersection.segment(i)?;
        points.push(segment.first_point()?);
        points.push(segment.last_point()?);
    }
    for i in 1..=intersection.nb_points()? {
        points.push(intersection.point(i)?);
    }
    Ok(closest_in_range(&points, prev_end, curr_end))
}

/// The scan of the collected points: a point replaces the best one when it is closer
/// on either curve (as upstream, not on both).
fn closest_in_range(
    points: &[IntersectionPoint],
    prev_end: &CurveEnd,
    curr_end: &CurveEnd,
) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64)> = None;
    let (mut delta_prev, mut delta_curr) = (f64::MAX, f64::MAX);
    for point in points {
        let (t_prev, t_curr) = (point.param_on_first(), point.param_on_second());
        let (fp, lp) = prev_end.range;
        let (fc, lc) = curr_end.range;
        if t_prev < fp || t_prev > lp || t_curr < fc || t_curr > lc {
            continue;
        }
        let d1 = (t_prev - prev_end.at).abs();
        let d2 = (t_curr - curr_end.at).abs();
        if d1 < delta_prev || d2 < delta_curr {
            best = Some((t_prev, t_curr));
            (delta_prev, delta_curr) = (d1, d2);
        }
    }
    best
}

/// The number of (sub-shape) pairs of the two edges that are the same vertex.
fn common_vertex_count(previous: &Edge, current: &Edge) -> Result<i32, Failure> {
    let mut count = 0;
    let mut first = ShapeIterator::new(previous, true, true);
    while first.more() {
        let mut second = ShapeIterator::new(current, true, true);
        while second.more() {
            if first.value()?.is_same(second.value()?) {
                count += 1;
            }
            second.next();
        }
        first.next();
    }
    Ok(count)
}
