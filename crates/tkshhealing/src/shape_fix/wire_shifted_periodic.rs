use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomOffsetCurve, GeomSurface, GeomSurfaceOfRevolution, GeomSweptSurface,
    GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Point2, Vector2};

pub(super) struct PeriodicSurfaceData {
    pub(super) closure: (bool, bool),
    pub(super) periods: (f64, f64),
    pub(super) bounds: (f64, f64, f64, f64),
}

/// UV bounds and periods shared by `FixShifted`'s adjacency and recentering passes.
pub(super) fn shifted_surface_data(
    analysis: &crate::shape_analysis::ShapeAnalysisSurface,
    surface: &Handle<dyn GeomSurface>,
) -> Result<PeriodicSurfaceData, Failure> {
    let closed = shifted_closure(analysis, surface)?;
    let bounds = surface
        .get()
        .ok_or_else(|| Failure::new("ShapeFix_Wire::FixShifted requires a support surface"))?
        .bounds();
    let periods = shifted_periods(closed, bounds, surface);
    Ok(PeriodicSurfaceData {
        closure: closed,
        periods,
        bounds,
    })
}

/// Filters native-degenerate edges that have no p-curve on the active face.
pub(super) fn shifted_edges(analyzer: &ShapeAnalysisWire) -> Result<Vec<Edge>, Failure> {
    let data = analyzer.wire_data();
    let edge_analysis = ShapeAnalysisEdge::new();
    let mut edges = Vec::new();
    for index in 1..=data.nb_edges() {
        let edge = data.edge(index)?;
        if BRepTool::degenerated(&edge)?
            && !edge_analysis.has_pcurve_on_face(&edge, &analyzer.face())?
        {
            continue;
        }
        edges.push(edge);
    }
    Ok(edges)
}

/// Returns the two parameters used by native global UV recentering.
pub(super) fn wire_uv_bounds(
    analyzer: &ShapeAnalysisWire,
    edges: &[Edge],
) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
    let mut bounds = UVBounds::default();
    for edge in edges {
        add_edge_uv_bounds(analyzer, edge, &mut bounds)?;
    }
    Ok(bounds.finish())
}

/// Collects the prior-edge endpoints used by FixShifted's early recenter test.
pub(super) fn initial_wire_uv_bounds(
    analyzer: &ShapeAnalysisWire,
    edges: &[Edge],
) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
    let mut bounds = UVBounds::default();
    for index in 0..edges.len() {
        add_pair_endpoint(analyzer, edges, index, &mut bounds)?;
    }
    Ok(bounds.finish())
}

fn add_pair_endpoint(
    analyzer: &ShapeAnalysisWire,
    edges: &[Edge],
    index: usize,
    bounds: &mut UVBounds,
) -> Result<(), Failure> {
    let previous = &edges[(index + edges.len() - 1) % edges.len()];
    let current = &edges[index];
    if BRepTool::degenerated(previous)? || BRepTool::degenerated(current)? {
        return Ok(());
    }
    let (curve, _, last) =
        ShapeAnalysisEdge::new().pcurve_on_face(previous, &analyzer.face(), true)?;
    let Some(curve) = curve.get() else {
        return Ok(());
    };
    bounds.add(curve.value(last)?);
    Ok(())
}

fn add_edge_uv_bounds(
    analyzer: &ShapeAnalysisWire,
    edge: &Edge,
    bounds: &mut UVBounds,
) -> Result<(), Failure> {
    let (curve, first, last) =
        ShapeAnalysisEdge::new().pcurve_on_face(edge, &analyzer.face(), true)?;
    let Some(curve) = curve.get() else {
        return Ok(());
    };
    bounds.add(curve.value(first)?);
    bounds.add(curve.value(0.5 * (first + last))?);
    Ok(())
}

/// Moves one p-curve endpoint across a period unless the endpoint is singular on that axis.
pub(super) fn periodic_gap_translation(
    previous: Point2,
    current: Point2,
    closed: (bool, bool),
    periods: (f64, f64),
    singular_axis: i32,
) -> Vector2 {
    let du = period_gap(
        previous.x(),
        current.x(),
        closed.0,
        periods.0,
        singular_axis != 1,
    );
    let dv = period_gap(
        previous.y(),
        current.y(),
        closed.1,
        periods.1,
        singular_axis != 2,
    );
    Vector2::new(du, dv)
}

fn period_gap(previous: f64, current: f64, closed: bool, period: f64, can_shift: bool) -> f64 {
    if !closed || !can_shift || (previous - current).abs() <= 0.8 * period {
        return 0.0;
    }
    adjust_by_period(current, previous, period)
}

/// Recenters the current wire bounds inside each closed parameter period.
pub(super) fn centered_translation(
    range: (f64, f64, f64, f64),
    closed: (bool, bool),
    periods: (f64, f64),
    bounds: (f64, f64, f64, f64),
) -> Vector2 {
    let u_middle = 0.5 * (range.0 + range.1);
    let v_middle = 0.5 * (range.2 + range.3);
    let surface_u_middle = 0.5 * (bounds.0 + bounds.1);
    let surface_v_middle = 0.5 * (bounds.2 + bounds.3);
    Vector2::new(
        periodic_middle_shift(u_middle, surface_u_middle, closed.0, periods.0),
        periodic_middle_shift(v_middle, surface_v_middle, closed.1, periods.1),
    )
}

fn periodic_middle_shift(value: f64, target: f64, closed: bool, period: f64) -> f64 {
    if closed {
        adjust_by_period(value, target, period)
    } else {
        0.0
    }
}

/// Tests the native first endpoint plus midpoint condition before a recenter pass.
pub(super) fn is_centered(range: (f64, f64, f64, f64), surface: (f64, f64, f64, f64)) -> bool {
    let u_width = (surface.1 - surface.0).abs();
    let v_width = (surface.3 - surface.2).abs();
    (range.0 + range.1 - surface.0 - surface.1).abs() < u_width
        && (range.2 + range.3 - surface.2 - surface.3).abs() < v_width
}

fn shifted_closure(
    analysis: &crate::shape_analysis::ShapeAnalysisSurface,
    surface: &Handle<dyn GeomSurface>,
) -> Result<(bool, bool), Failure> {
    let u_closed = analysis.is_u_closed(rustycad_tkernel::precision::confusion())?;
    let v_closed = analysis.is_v_closed(rustycad_tkernel::precision::confusion())?
        || surface.is_kind_name("Geom_SphericalSurface")
        || revolution_basis_period(surface).is_some();
    Ok((u_closed, v_closed))
}

fn shifted_periods(
    closed: (bool, bool),
    bounds: (f64, f64, f64, f64),
    surface: &Handle<dyn GeomSurface>,
) -> (f64, f64) {
    let mut periods = closed_periods(closed, bounds);
    if let Some(period) = revolution_basis_period(surface) {
        periods.1 = period;
    }
    periods
}

fn closed_periods(closed: (bool, bool), bounds: (f64, f64, f64, f64)) -> (f64, f64) {
    (
        if closed.0 {
            (bounds.1 - bounds.0).abs()
        } else {
            rustycad_tkernel::precision::infinite()
        },
        if closed.1 {
            (bounds.3 - bounds.2).abs()
        } else {
            rustycad_tkernel::precision::infinite()
        },
    )
}

fn revolution_basis_period(surface: &Handle<dyn GeomSurface>) -> Option<f64> {
    let revolution = surface.cast::<GeomSurfaceOfRevolution>();
    let curve = revolution.get()?.basis_curve();
    let curve = unwrap_periodic_curve(curve);
    let curve = curve.get()?;
    curve.is_periodic().then(|| curve.period().ok()).flatten()
}

fn unwrap_periodic_curve(mut curve: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    loop {
        if let Some(offset) = curve.cast::<GeomOffsetCurve>().get() {
            curve = offset.basis_curve();
        } else if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
            curve = trimmed.basis_curve();
        } else {
            return curve;
        }
    }
}

fn adjust_by_period(value: f64, target: f64, period: f64) -> f64 {
    let difference = value - target;
    let distance = difference.abs();
    if distance <= 0.5 * period {
        return 0.0;
    }
    let multiple = (distance / period + 0.5).floor();
    difference.signum().mul_add(-period * multiple, 0.0)
}

#[derive(Default)]
struct UVBounds {
    min_u: f64,
    max_u: f64,
    min_v: f64,
    max_v: f64,
    seen: bool,
}

impl UVBounds {
    fn add(&mut self, point: Point2) {
        if !self.seen {
            (self.min_u, self.max_u, self.min_v, self.max_v) =
                (point.x(), point.x(), point.y(), point.y());
            self.seen = true;
            return;
        }
        self.min_u = self.min_u.min(point.x());
        self.max_u = self.max_u.max(point.x());
        self.min_v = self.min_v.min(point.y());
        self.max_v = self.max_v.max(point.y());
    }

    fn finish(self) -> Option<(f64, f64, f64, f64)> {
        self.seen
            .then_some((self.min_u, self.max_u, self.min_v, self.max_v))
    }
}
