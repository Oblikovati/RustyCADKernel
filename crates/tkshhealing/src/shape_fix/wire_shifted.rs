// Rust translation of the common periodic-axis path in ShapeFix_Wire::FixShifted at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Align adjacent p-curves across a closed surface period and recenter their UV bounds.
use super::ShapeFixWire;
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkmath::gp::{Transform2, Vector2};

impl ShapeFixWire {
    /// `FixShifted()`: align p-curves on a closed periodic surface before degenerated repair.
    /// `fixer.fix_shifted()?`.
    pub fn fix_shifted(&self) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let analyzer = self.analyzer_for_shifted()?;
        if !analyzer.is_ready() {
            return Ok(false);
        }
        let surface_analysis = analyzer.surface();
        let closed = (
            surface_analysis.is_u_closed(rustycad_tkernel::precision::confusion())?,
            surface_analysis.is_v_closed(rustycad_tkernel::precision::confusion())?,
        );
        if !closed.0 && !closed.1 {
            return Ok(false);
        }
        let surface = surface_analysis.surface();
        let surface = surface
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire::FixShifted requires a support surface"))?;
        let bounds = surface.bounds();
        let periods = closed_periods(closed, bounds);
        self.shift_adjacent_period_gaps(&analyzer, closed, periods)?;
        self.recenter_periodic_wire(&analyzer, closed, periods, bounds)
    }

    fn analyzer_for_shifted(
        &self,
    ) -> Result<rustycad_tkernel::standard::Handle<ShapeAnalysisWire>, Failure> {
        let analyzer = self.state.read().analyzer.clone();
        if analyzer.is_null() {
            return Err(Failure::new(
                "ShapeFix_Wire requires Init(analyzer) or Load(wire)",
            ));
        }
        Ok(analyzer)
    }

    fn shift_adjacent_period_gaps(
        &self,
        analyzer: &ShapeAnalysisWire,
        closed: (bool, bool),
        periods: (f64, f64),
    ) -> Result<(), Failure> {
        let count = analyzer.nb_edges();
        for index in 2..=count {
            self.shift_one_period_gap(analyzer, index, closed, periods)?;
        }
        Ok(())
    }

    fn shift_one_period_gap(
        &self,
        analyzer: &ShapeAnalysisWire,
        index: i32,
        closed: (bool, bool),
        periods: (f64, f64),
    ) -> Result<(), Failure> {
        let data = analyzer.wire_data();
        let previous = data.edge(index - 1)?;
        let current = data.edge(index)?;
        if rustycad_tkbrep::brep::BRepTool::degenerated(&previous)?
            || rustycad_tkbrep::brep::BRepTool::degenerated(&current)?
        {
            return Ok(());
        }
        let face = analyzer.face();
        let edge_analysis = ShapeAnalysisEdge::new();
        let prev_curve = edge_analysis.pcurve_on_face(&previous, &face, true)?;
        let curr_curve = edge_analysis.pcurve_on_face(&current, &face, true)?;
        self.shift_if_period_gap(&current, &face, prev_curve, curr_curve, closed, periods)
    }

    fn shift_if_period_gap(
        &self,
        edge: &rustycad_tkbrep::topods::Edge,
        face: &rustycad_tkbrep::topods::Face,
        previous: (
            rustycad_tkernel::standard::Handle<dyn Geom2dCurve>,
            f64,
            f64,
        ),
        current: (
            rustycad_tkernel::standard::Handle<dyn Geom2dCurve>,
            f64,
            f64,
        ),
        closed: (bool, bool),
        periods: (f64, f64),
    ) -> Result<(), Failure> {
        let (previous_curve, _, previous_last) = previous;
        let (current_curve, current_first, current_last) = current;
        let (Some(previous), Some(current)) = (previous_curve.get(), current_curve.get()) else {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
            return Ok(());
        };
        let previous_end = previous.value(previous_last)?;
        let current_start = current.value(current_first)?;
        let shift = periodic_gap_translation(previous_end, current_start, closed, periods);
        if shift == Vector2::default() {
            return Ok(());
        }
        let (replacement, _, _) = ShapeBuildEdge.transform_pcurve(
            &current_curve,
            Transform2::translation_vector(shift),
            1.0,
            current_first,
            current_last,
        )?;
        ShapeBuildEdge.replace_pcurve(edge, &replacement, face)?;
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        Ok(())
    }

    fn recenter_periodic_wire(
        &self,
        analyzer: &ShapeAnalysisWire,
        closed: (bool, bool),
        periods: (f64, f64),
        bounds: (f64, f64, f64, f64),
    ) -> Result<bool, Failure> {
        let Some((min_u, max_u, min_v, max_v)) = self.wire_uv_bounds(analyzer)? else {
            return Ok(false);
        };
        if is_centered((min_u, max_u, min_v, max_v), bounds)
            && !self.last_fix_status(ShapeExtendStatus::Done)
        {
            return Ok(false);
        }
        let shift = centered_translation((min_u, max_u, min_v, max_v), closed, periods, bounds);
        if shift == Vector2::default() {
            return Ok(true);
        }
        self.shift_all_pcurves(analyzer, shift)?;
        self.or_last_fix_status(ShapeExtendStatus::Done2);
        Ok(true)
    }

    fn wire_uv_bounds(
        &self,
        analyzer: &ShapeAnalysisWire,
    ) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
        let mut bounds = UVBounds::default();
        for index in 1..=analyzer.nb_edges() {
            let edge = analyzer.wire_data().edge(index)?;
            let (curve, first, last) =
                ShapeAnalysisEdge::new().pcurve_on_face(&edge, &analyzer.face(), true)?;
            let Some(curve) = curve.get() else { continue };
            bounds.add(curve.value(first)?);
            bounds.add(curve.value(0.5 * (first + last))?);
        }
        Ok(bounds.finish())
    }

    fn shift_all_pcurves(
        &self,
        analyzer: &ShapeAnalysisWire,
        shift: Vector2,
    ) -> Result<(), Failure> {
        let face = analyzer.face();
        for index in 1..=analyzer.nb_edges() {
            let edge = analyzer.wire_data().edge(index)?;
            let (curve, first, last) =
                ShapeAnalysisEdge::new().pcurve_on_face(&edge, &face, true)?;
            if curve.is_null() {
                continue;
            }
            let (replacement, _, _) = ShapeBuildEdge.transform_pcurve(
                &curve,
                Transform2::translation_vector(shift),
                1.0,
                first,
                last,
            )?;
            ShapeBuildEdge.replace_pcurve(&edge, &replacement, &face)?;
        }
        Ok(())
    }
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
    fn add(&mut self, point: rustycad_tkmath::gp::Point2) {
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

fn periodic_gap_translation(
    previous: rustycad_tkmath::gp::Point2,
    current: rustycad_tkmath::gp::Point2,
    closed: (bool, bool),
    periods: (f64, f64),
) -> Vector2 {
    let du = if closed.0 && (previous.x() - current.x()).abs() > 0.8 * periods.0 {
        adjust_by_period(current.x(), previous.x(), periods.0)
    } else {
        0.0
    };
    let dv = if closed.1 && (previous.y() - current.y()).abs() > 0.8 * periods.1 {
        adjust_by_period(current.y(), previous.y(), periods.1)
    } else {
        0.0
    };
    Vector2::new(du, dv)
}

fn centered_translation(
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
        if closed.0 {
            adjust_by_period(u_middle, surface_u_middle, periods.0)
        } else {
            0.0
        },
        if closed.1 {
            adjust_by_period(v_middle, surface_v_middle, periods.1)
        } else {
            0.0
        },
    )
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

fn is_centered(range: (f64, f64, f64, f64), surface: (f64, f64, f64, f64)) -> bool {
    let u_width = (surface.1 - surface.0).abs();
    let v_width = (surface.3 - surface.2).abs();
    (range.0 + range.1 - surface.0 - surface.1).abs() < u_width
        && (range.2 + range.3 - surface.2 - surface.3).abs() < v_width
}
