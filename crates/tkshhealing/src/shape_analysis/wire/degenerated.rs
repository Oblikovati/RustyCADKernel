// Rust translation of ShapeAnalysis_Wire::CheckDegenerated at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Detect a missing or incorrectly marked singular edge between consecutive wire edges.
use super::*;
use crate::shape_analysis::ShapeAnalysisEdge;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Edge;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::gp::Point2;

struct CandidateEdges<'a> {
    previous: &'a Edge,
    current: &'a Edge,
    next: &'a Edge,
    indices: (i32, i32),
}

impl ShapeAnalysisWire {
    /// `CheckDegenerated(num, p2d1, p2d2)`; reports the native status and candidate UV segment.
    /// `let (found, first, last) = analyzer.check_degenerated_at(1)?`.
    pub fn check_degenerated_at(&self, index: i32) -> Result<(bool, Point2, Point2), Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() || self.nb_edges() < 1 {
            return Ok((false, Point2::default(), Point2::default()));
        }
        self.check_degenerated_edges(index)
    }

    fn check_degenerated_edges(&self, index: i32) -> Result<(bool, Point2, Point2), Failure> {
        let count = self.nb_edges();
        let current = if index > 0 { index } else { count };
        let previous = if current > 1 { current - 1 } else { count };
        let next = if current < count { current + 1 } else { 1 };
        let data = self.wire_data();
        let (first, second, third) = (data.edge(previous)?, data.edge(current)?, data.edge(next)?);
        self.check_degenerated_pair(&first, &second, &third, previous, current)
    }

    fn check_degenerated_pair(
        &self,
        previous: &Edge,
        current: &Edge,
        next: &Edge,
        previous_index: i32,
        current_index: i32,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let edge_analysis = ShapeAnalysisEdge::new();
        if self.skip_marked_degenerated(current, previous, next, &edge_analysis)? {
            return Ok((false, Point2::default(), Point2::default()));
        }
        if self.skip_after_unrepresented_degenerated(current, previous, &edge_analysis)? {
            return Ok((false, Point2::default(), Point2::default()));
        }
        self.check_degenerated_vertices(previous, current, next, previous_index, current_index)
    }

    fn skip_after_unrepresented_degenerated(
        &self,
        current: &Edge,
        previous: &Edge,
        analysis: &ShapeAnalysisEdge,
    ) -> Result<bool, Failure> {
        if current.is_same(previous) || !BRepTool::degenerated(previous)? {
            return Ok(false);
        }
        if analysis.has_pcurve_on_face(previous, &self.face())? {
            return Ok(false);
        }
        if BRepTool::degenerated(current)? {
            self.set_last_status(ShapeExtendStatus::Fail2);
        }
        Ok(true)
    }

    fn skip_marked_degenerated(
        &self,
        current: &Edge,
        previous: &Edge,
        next: &Edge,
        analysis: &ShapeAnalysisEdge,
    ) -> Result<bool, Failure> {
        if !BRepTool::degenerated(current)?
            || !analysis.has_pcurve_on_face(current, &self.face())?
        {
            return Ok(false);
        }
        self.check_marked_pcurve(current, previous, next, analysis)?;
        Ok(true)
    }

    fn check_marked_pcurve(
        &self,
        current: &Edge,
        previous: &Edge,
        next: &Edge,
        analysis: &ShapeAnalysisEdge,
    ) -> Result<(), Failure> {
        let face = self.face();
        if !analysis.has_pcurve_on_face(previous, &face)?
            || !analysis.has_pcurve_on_face(next, &face)?
        {
            return Ok(());
        }
        if !marked_degenerated_pcurve_is_valid(current, previous, next, &face, analysis)? {
            self.set_last_status(ShapeExtendStatus::Fail2);
        }
        Ok(())
    }

    fn check_degenerated_vertices(
        &self,
        previous: &Edge,
        current: &Edge,
        next: &Edge,
        previous_index: i32,
        current_index: i32,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (previous_start, previous_end) = (
            analysis.first_vertex(previous),
            analysis.last_vertex(previous),
        );
        let (current_start, current_end) = (
            analysis.first_vertex(current),
            analysis.last_vertex(current),
        );
        if vertices_missing(&previous_start, &previous_end, &current_start, &current_end) {
            return Ok((false, Point2::default(), Point2::default()));
        }
        self.find_singular_edge(
            previous,
            current,
            next,
            (previous_index, current_index),
            (&previous_start, &previous_end, &current_start, &current_end),
        )
    }

    fn find_singular_edge(
        &self,
        previous: &Edge,
        current: &Edge,
        next: &Edge,
        indices: (i32, i32),
        vertices: (
            &rustycad_tkbrep::topods::Vertex,
            &rustycad_tkbrep::topods::Vertex,
            &rustycad_tkbrep::topods::Vertex,
            &rustycad_tkbrep::topods::Vertex,
        ),
    ) -> Result<(bool, Point2, Point2), Failure> {
        let (previous_start, previous_end, current_start, current_end) = vertices;
        let edges = CandidateEdges {
            previous,
            current,
            next,
            indices,
        };
        let tolerance = vertex_threshold(self.precision(), current_start)?;
        let point = BRepTool::pnt(current_start)?;
        let already_degenerated =
            self.detect_existing(current, current_start, current_end, point, tolerance)?;
        if already_degenerated {
            return self.finish_candidate(&edges, tolerance, true);
        }
        let previous_point = BRepTool::pnt(previous_start)?;
        if previous_index_is_distinct(indices.0, indices.1)
            && BRepTool::pnt(previous_end)?.distance(&point) <= tolerance.0
            && self.surface().is_degenerated(previous_point, tolerance.0)?
            && !BRepTool::degenerated(previous)?
        {
            return Ok((false, Point2::default(), Point2::default()));
        }
        self.find_missing_candidate(&edges, point, tolerance.1)
    }

    fn detect_existing(
        &self,
        current: &Edge,
        first: &rustycad_tkbrep::topods::Vertex,
        last: &rustycad_tkbrep::topods::Vertex,
        point: rustycad_tkmath::gp::Point3,
        tolerance: (f64, f64),
    ) -> Result<bool, Failure> {
        if BRepTool::pnt(first)?.distance(&BRepTool::pnt(last)?) > tolerance.0 {
            return Ok(false);
        }
        let surface = self.surface();
        let Some(singularity) = surface.degenerated_values(point, tolerance.1, true)? else {
            return Ok(false);
        };
        let (curve, first_parameter, last_parameter) =
            ShapeAnalysisEdge::new().curve_3d(current, true)?;
        let Some(curve) = curve.get() else {
            return Ok(true);
        };
        let midpoint = curve.value(0.5 * (first_parameter + last_parameter))?;
        Ok(midpoint.distance(&point) <= tolerance.1
            || singularity.point.distance(&point) <= tolerance.1)
    }

    fn find_missing_candidate(
        &self,
        edges: &CandidateEdges<'_>,
        point: rustycad_tkmath::gp::Point3,
        tolerance: f64,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let surface = self.surface();
        let mut closest = None;
        let mut minimum = f64::MAX;
        for rank in 1..=surface.nb_singularities(tolerance)? {
            let Some(entry) = surface.singularity(rank as i32)? else {
                continue;
            };
            let gap = point.square_distance(&entry.point);
            if gap <= tolerance * tolerance && gap < minimum {
                minimum = gap;
                closest = Some(entry);
            }
        }
        let Some(entry) = closest else {
            return self.no_candidate_status(edges.current);
        };
        self.finish_candidate_points(edges, entry.first_uv, entry.last_uv, false)
    }

    fn no_candidate_status(&self, current: &Edge) -> Result<(bool, Point2, Point2), Failure> {
        if BRepTool::degenerated(current)?
            && !ShapeAnalysisEdge::new().has_pcurve_on_face(current, &self.face())?
        {
            self.set_last_status(ShapeExtendStatus::Fail2);
        }
        Ok((false, Point2::default(), Point2::default()))
    }

    fn finish_candidate(
        &self,
        edges: &CandidateEdges<'_>,
        tolerance: (f64, f64),
        existing: bool,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let surface = self.surface();
        let Some(entry) = surface.degenerated_values(
            BRepTool::pnt(&ShapeAnalysisEdge::new().first_vertex(edges.current))?,
            tolerance.1,
            true,
        )?
        else {
            return self.no_candidate_status(edges.current);
        };
        self.finish_candidate_points(edges, entry.first_uv, entry.last_uv, existing)
    }

    fn finish_candidate_points(
        &self,
        edges: &CandidateEdges<'_>,
        mut first: Point2,
        mut last: Point2,
        existing: bool,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let surface = self.surface();
        if !existing || edges.indices.0 != edges.indices.1 {
            first = self.pcurve_endpoint(edges.previous, true, first)?;
            last = self.pcurve_endpoint(
                if existing { edges.next } else { edges.current },
                false,
                last,
            )?;
        }
        if self.missing_pcurve_status(edges.previous, edges.current, edges.next, existing)? {
            return Ok((false, first, last));
        }
        if !surface.is_degenerated_segment(first, last, self.precision(), 10.)? {
            return self.no_candidate_status(edges.current);
        }
        if first.distance(last) <= self.parameter_resolution()? {
            return Ok((false, first, last));
        }
        let status = if existing {
            ShapeExtendStatus::Done2
        } else {
            ShapeExtendStatus::Done1
        };
        self.set_last_status(status);
        Ok((true, first, last))
    }

    fn pcurve_endpoint(
        &self,
        edge: &Edge,
        at_end: bool,
        current: Point2,
    ) -> Result<Point2, Failure> {
        let (pcurve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(edge, &self.face(), true)?;
        let Some(pcurve) = pcurve.get() else {
            self.set_last_status(ShapeExtendStatus::Fail1);
            return Ok(current);
        };
        pcurve.value(if at_end { last } else { first })
    }

    fn missing_pcurve_status(
        &self,
        previous: &Edge,
        current: &Edge,
        next: &Edge,
        existing: bool,
    ) -> Result<bool, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let first_missing = !analysis.has_pcurve_on_face(previous, &self.face())?;
        let last_edge = if existing { next } else { current };
        let last_missing = !analysis.has_pcurve_on_face(last_edge, &self.face())?;
        if first_missing || last_missing {
            self.set_last_status(ShapeExtendStatus::Fail1);
        }
        Ok(first_missing || last_missing)
    }

    fn parameter_resolution(&self) -> Result<f64, Failure> {
        let adaptor = self.surface().adaptor3d();
        let surface = adaptor
            .get()
            .ok_or_else(|| Failure::new("ShapeAnalysis_Wire has no surface adaptor"))?;
        Ok(surface
            .u_resolution(self.precision())?
            .max(surface.v_resolution(self.precision())?)
            + rustycad_tkmath::gp::resolution())
    }
}

fn vertex_threshold(
    precision: f64,
    vertex: &rustycad_tkbrep::topods::Vertex,
) -> Result<(f64, f64), Failure> {
    let vertex_tolerance = BRepTool::vertex_tolerance(vertex)?;
    let lower = precision.min(vertex_tolerance);
    let upper = precision.max(vertex_tolerance);
    Ok((
        lower,
        if precision < vertex_tolerance {
            2. * upper
        } else {
            upper
        },
    ))
}

fn vertices_missing(
    first_start: &rustycad_tkbrep::topods::Vertex,
    first_end: &rustycad_tkbrep::topods::Vertex,
    second_start: &rustycad_tkbrep::topods::Vertex,
    second_end: &rustycad_tkbrep::topods::Vertex,
) -> bool {
    first_start.is_null() || first_end.is_null() || second_start.is_null() || second_end.is_null()
}

fn previous_index_is_distinct(previous: i32, current: i32) -> bool {
    previous != current
}

fn marked_degenerated_pcurve_is_valid(
    current: &Edge,
    previous: &Edge,
    next: &Edge,
    face: &rustycad_tkbrep::topods::Face,
    analysis: &ShapeAnalysisEdge,
) -> Result<bool, Failure> {
    let current_length = pcurve_length(current, face, analysis)?;
    let previous_end = pcurve_uv(previous, face, analysis, true)?;
    let next_start = pcurve_uv(next, face, analysis, false)?;
    Ok((previous_end.distance(next_start) - current_length).abs()
        <= 2. * rustycad_tkernel::precision::p_confusion())
}

fn pcurve_length(
    edge: &Edge,
    face: &rustycad_tkbrep::topods::Face,
    analysis: &ShapeAnalysisEdge,
) -> Result<f64, Failure> {
    let (curve, first, last) = analysis.pcurve_on_face(edge, face, true)?;
    let Some(curve) = curve.get() else {
        return Ok(0.);
    };
    Ok(curve.value(first)?.distance(curve.value(last)?))
}

fn pcurve_uv(
    edge: &Edge,
    face: &rustycad_tkbrep::topods::Face,
    analysis: &ShapeAnalysisEdge,
    at_end: bool,
) -> Result<Point2, Failure> {
    let (curve, first, last) = analysis.pcurve_on_face(edge, face, true)?;
    let curve = curve
        .get()
        .ok_or_else(|| Failure::new("ShapeAnalysis_Wire requires neighboring p-curves"))?;
    curve.value(if at_end { last } else { first })
}
