// Rust translation of the common periodic-axis path in ShapeFix_Wire::FixShifted at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Align adjacent p-curves across a closed surface period and recenter their UV bounds.
use super::{FixRoot, ShapeFixWire};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Point2, Point3, Transform2, Vector2};
#[path = "wire_shifted_meridian.rs"]
mod meridian;
#[path = "wire_shifted_periodic.rs"]
mod periodic;

struct ShiftedTraversal {
    stop_index: usize,
    degenerate_stop: bool,
    first_singularity: Option<(usize, Point3)>,
    initial_bounds: periodic::UVBounds,
}

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
        let surface_handle = surface_analysis.surface();
        let surface_data =
            periodic::shifted_surface_data(&surface_analysis, &surface_handle, self.precision())?;
        let (closed, periods, bounds) = (
            surface_data.closure,
            surface_data.periods,
            surface_data.bounds,
        );
        if !closed.0 && !closed.1 {
            return Ok(false);
        }
        let edges = periodic::shifted_edges(&analyzer)?;
        let initial_bounds = self.shift_adjacent_period_gaps(&analyzer, &edges, closed, periods)?;
        self.recenter_periodic_wire(&analyzer, &edges, initial_bounds, closed, periods, bounds)
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
        edges: &[rustycad_tkbrep::topods::Edge],
        closed: (bool, bool),
        periods: (f64, f64),
    ) -> Result<Option<(f64, f64, f64, f64)>, Failure> {
        if edges.is_empty() {
            return Ok(None);
        }
        let mut traversal = ShiftedTraversal {
            stop_index: edges.len() - 1,
            degenerate_stop: false,
            first_singularity: None,
            initial_bounds: periodic::UVBounds::default(),
        };
        self.run_shifted_traversal(analyzer, edges, closed, periods, &mut traversal)?;
        Ok(traversal.initial_bounds.finish())
    }

    fn run_shifted_traversal(
        &self,
        analyzer: &ShapeAnalysisWire,
        edges: &[Edge],
        closed: (bool, bool),
        periods: (f64, f64),
        traversal: &mut ShiftedTraversal,
    ) -> Result<(), Failure> {
        let mut index = 0;
        loop {
            let stop = index == traversal.stop_index;
            self.shift_traversal_pair(analyzer, edges, index, closed, periods, traversal)?;
            if stop {
                return Ok(());
            }
            index = (index + 1) % edges.len();
        }
    }

    fn shift_traversal_pair(
        &self,
        analyzer: &ShapeAnalysisWire,
        edges: &[Edge],
        index: usize,
        closed: (bool, bool),
        periods: (f64, f64),
        traversal: &mut ShiftedTraversal,
    ) -> Result<(), Failure> {
        let current = &edges[index];
        if self.period_pair_is_degenerate(edges, index, traversal)? {
            return Ok(());
        }
        let Some((point, axis)) = self.period_pair_singularity(current)? else {
            return Ok(());
        };
        if axis != 0
            && self.try_singular_bimeridian(edges, index, point, closed, periods, traversal)?
        {
            return Ok(());
        }
        let shift = self.shift_one_period_gap(analyzer, edges, index, closed, periods)?;
        let Some((endpoint, moderate_gap)) = shift else {
            return Ok(());
        };
        traversal.initial_bounds.add(endpoint);
        if moderate_gap && traversal.stop_index == edges.len() - 1 {
            traversal.stop_index = index;
        }
        Ok(())
    }

    fn period_pair_is_degenerate(
        &self,
        edges: &[Edge],
        index: usize,
        traversal: &mut ShiftedTraversal,
    ) -> Result<bool, Failure> {
        let previous = &edges[(index + edges.len() - 1) % edges.len()];
        let current = &edges[index];
        if !BRepTool::degenerated(previous)? && !BRepTool::degenerated(current)? {
            return Ok(false);
        }
        if !traversal.degenerate_stop {
            traversal.stop_index = index;
            traversal.degenerate_stop = true;
        }
        Ok(true)
    }

    fn period_pair_singularity(&self, edge: &Edge) -> Result<Option<(Point3, i32)>, Failure> {
        let vertex = ShapeAnalysisEdge::new().first_vertex(edge);
        if vertex.is_null() {
            return Ok(None);
        }
        let point = BRepTool::pnt(&vertex)?;
        let axis = self.singular_axis_for_edge(edge)?;
        Ok(Some((point, axis)))
    }

    fn try_singular_bimeridian(
        &self,
        edges: &[Edge],
        index: usize,
        point: Point3,
        closed: (bool, bool),
        periods: (f64, f64),
        traversal: &mut ShiftedTraversal,
    ) -> Result<bool, Failure> {
        let Some((start, previous)) = traversal.first_singularity else {
            traversal.first_singularity = Some((index, point));
            return Ok(false);
        };
        if previous.distance(&point) < self.precision() {
            traversal.first_singularity = Some((index, point));
            return Ok(false);
        }
        match self.shift_bimeridian_range(edges, start, index, closed, periods)? {
            None | Some(true) => Ok(true),
            Some(false) => Ok(false),
        }
    }

    fn shift_bimeridian_range(
        &self,
        edges: &[rustycad_tkbrep::topods::Edge],
        start: usize,
        stop: usize,
        closed: (bool, bool),
        periods: (f64, f64),
    ) -> Result<Option<bool>, Failure> {
        let face = self.analyzer_for_shifted()?.face();
        let shift = match meridian::bimeridian_shift(edges, start, stop, &face, closed, periods)? {
            meridian::BimeridianShift::MissingCurve => {
                self.or_last_fix_status(ShapeExtendStatus::Fail1);
                return Ok(None);
            }
            meridian::BimeridianShift::NoCrossing => return Ok(Some(false)),
            meridian::BimeridianShift::Translate(shift) => shift,
        };
        self.translate_edge_range(edges, start, stop, &face, shift)?;
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        Ok(Some(true))
    }

    fn translate_edge_range(
        &self,
        edges: &[rustycad_tkbrep::topods::Edge],
        start: usize,
        stop: usize,
        face: &Face,
        shift: Vector2,
    ) -> Result<(), Failure> {
        let mut index = start;
        while index != stop {
            self.translate_edge_pcurve(&edges[index], face, shift)?;
            index = (index + 1) % edges.len();
        }
        Ok(())
    }

    fn translate_edge_pcurve(
        &self,
        edge: &Edge,
        face: &Face,
        shift: Vector2,
    ) -> Result<(), Failure> {
        let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
        if curve.is_null() {
            return Ok(());
        }
        let (replacement, _, _) = ShapeBuildEdge.transform_pcurve(
            &curve,
            Transform2::translation_vector(shift),
            1.0,
            first,
            last,
        )?;
        ShapeBuildEdge.replace_pcurve(edge, &replacement, face)?;
        self.update_edge_uv_points(edge, face)
    }

    fn shift_one_period_gap(
        &self,
        analyzer: &ShapeAnalysisWire,
        edges: &[rustycad_tkbrep::topods::Edge],
        index: usize,
        closed: (bool, bool),
        periods: (f64, f64),
    ) -> Result<Option<(Point2, bool)>, Failure> {
        let count = edges.len();
        let previous = edges[(index + count - 1) % count].clone();
        let current = edges[index].clone();
        if rustycad_tkbrep::brep::BRepTool::degenerated(&previous)?
            || rustycad_tkbrep::brep::BRepTool::degenerated(&current)?
        {
            return Ok(None);
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
    ) -> Result<Option<(Point2, bool)>, Failure> {
        let (previous_curve, previous_first, previous_last) = previous;
        let (current_curve, current_first, current_last) = current;
        let (Some(previous), Some(current)) = (previous_curve.get(), current_curve.get()) else {
            self.or_last_fix_status(ShapeExtendStatus::Fail1);
            return Ok(None);
        };
        let previous_end = previous.value(previous_last)?;
        let current_start = current.value(current_first)?;
        let singular_axis = self.singular_axis_for_edge(edge)?;
        let singular_axis = if singular_axis == 0 {
            self.near_degenerate_axis(&previous_curve, previous_first, previous_last, closed)?
        } else {
            singular_axis
        };
        let shift = periodic::periodic_gap_translation(
            previous_end,
            current_start,
            closed,
            periods,
            singular_axis,
        );
        let moderate_gap =
            periodic::is_moderate_gap(previous_end, current_start, closed, periods, singular_axis);
        if shift == Vector2::default() {
            return Ok(Some((previous_end, moderate_gap)));
        }
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        let (replacement, _, _) = ShapeBuildEdge.transform_pcurve(
            &current_curve,
            Transform2::translation_vector(shift),
            1.0,
            current_first,
            current_last,
        )?;
        ShapeBuildEdge.replace_pcurve(edge, &replacement, face)?;
        self.update_edge_uv_points(edge, face)?;
        Ok(Some((previous_end, false)))
    }

    fn singular_axis_for_edge(&self, edge: &Edge) -> Result<i32, Failure> {
        let vertex = ShapeAnalysisEdge::new().first_vertex(edge);
        if vertex.is_null() {
            return Ok(0);
        }
        let point = BRepTool::pnt(&vertex)?;
        let tolerance = BRepTool::vertex_tolerance(&vertex)?.max(self.precision());
        let analysis = self.analyzer_for_shifted()?.surface();
        let Some(singularity) = analysis.degenerated_values(point, tolerance, true)? else {
            return Ok(0);
        };
        let du = (singularity.first_uv.x() - singularity.last_uv.x()).abs();
        let dv = (singularity.first_uv.y() - singularity.last_uv.y()).abs();
        Ok(if du > dv { 1 } else { 2 })
    }

    fn update_edge_uv_points(&self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, true)?;
        let curve = curve
            .get()
            .ok_or_else(|| Failure::new("ShapeFix_Wire requires the shifted p-curve"))?;
        BRepTool::set_uv_points_on_face(edge, face, curve.value(first)?, curve.value(last)?)
    }

    fn near_degenerate_axis(
        &self,
        previous: &rustycad_tkernel::standard::Handle<dyn Geom2dCurve>,
        first: f64,
        last: f64,
        closed: (bool, bool),
    ) -> Result<i32, Failure> {
        let analysis = self.analyzer_for_shifted()?.surface();
        let surface = analysis.surface();
        if !surface.is_kind_name("Geom_SurfaceOfRevolution") {
            return Ok(0);
        }
        let Some(previous) = previous.get() else {
            return Ok(0);
        };
        self.classify_revolution_boundaries(&analysis, previous, first, last, closed)
    }

    fn classify_revolution_boundaries(
        &self,
        analysis: &crate::shape_analysis::ShapeAnalysisSurface,
        previous: &dyn Geom2dCurve,
        first: f64,
        last: f64,
        closed: (bool, bool),
    ) -> Result<i32, Failure> {
        let previous_start = previous.value(first)?;
        let previous_end = previous.value(last)?;
        let bounds = analysis.surface().get().map(GeomSurface::bounds);
        let Some(bounds) = bounds else { return Ok(0) };
        let tolerance = self.max_tolerance();
        if !closed.1 {
            let (uf, ul, _, _) = bounds;
            let boundary = (
                rustycad_tkmath::gp::Point2::new(uf, previous_end.y()),
                rustycad_tkmath::gp::Point2::new(ul, previous_end.y()),
            );
            if analysis.is_degenerated_segment(boundary.0, boundary.1, tolerance, 10.)?
                && !analysis.is_degenerated_segment(previous_start, previous_end, tolerance, 10.)?
            {
                return Ok(1);
            }
        }
        if !closed.0 {
            let (_, _, vf, vl) = bounds;
            let boundary = (
                rustycad_tkmath::gp::Point2::new(previous_end.x(), vf),
                rustycad_tkmath::gp::Point2::new(previous_end.x(), vl),
            );
            if analysis.is_degenerated_segment(boundary.0, boundary.1, tolerance, 10.)?
                && !analysis.is_degenerated_segment(previous_start, previous_end, tolerance, 10.)?
            {
                return Ok(2);
            }
        }
        Ok(0)
    }

    fn recenter_periodic_wire(
        &self,
        analyzer: &ShapeAnalysisWire,
        edges: &[Edge],
        initial_bounds: Option<(f64, f64, f64, f64)>,
        closed: (bool, bool),
        periods: (f64, f64),
        bounds: (f64, f64, f64, f64),
    ) -> Result<bool, Failure> {
        let Some(initial_bounds) = initial_bounds else {
            return Ok(false);
        };
        if periodic::is_centered(initial_bounds, bounds, periods)
            && !self.last_fix_status(ShapeExtendStatus::Done)
        {
            return Ok(false);
        }
        let Some((min_u, max_u, min_v, max_v)) = periodic::wire_uv_bounds(analyzer, edges)? else {
            return Ok(false);
        };
        let shift =
            periodic::centered_translation((min_u, max_u, min_v, max_v), closed, periods, bounds);
        if shift == Vector2::default() {
            return Ok(true);
        }
        self.or_last_fix_status(ShapeExtendStatus::Done2);
        self.shift_all_pcurves(analyzer, shift)?;
        Ok(true)
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
            self.update_edge_uv_points(&edge, &face)?;
        }
        Ok(())
    }
}
