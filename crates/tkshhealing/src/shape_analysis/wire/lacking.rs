//! Missing p-curve connection checks from pinned OCCT ShapeAnalysis_Wire.cxx.
use super::*;
use rustycad_tkbrep::{
    brep_tools::compare_vertices,
    topods::{Edge, Vertex},
};
use rustycad_tkg2d::{adaptor2d::Adaptor2dCurve2d, geom2d_adaptor::Geom2dAdaptorCurve};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::Orientation};
use rustycad_tkmath::gp::{self, Point2, Vector2};

impl ShapeAnalysisWire {
    /// Accumulate missing-connection findings over a wire with at least two edges.
    /// Example: `analyzer.check_lacking()?`.
    pub fn check_lacking(&self) -> Result<bool, Failure> {
        if !self.is_ready() || self.nb_edges() < 2 {
            return Ok(false);
        }
        let mut index = 1;
        while index <= self.wire_data().nb_edges() {
            self.check_lacking_edge(index, 0.0)?;
            let mut fields = self.fields.write();
            fields.status_lacking |= fields.status;
            index += 1;
        }
        Ok(self.status_lacking(ShapeExtendStatus::Done))
    }

    /// Discard UV endpoints from one missing-connection check.
    /// Example: `analyzer.check_lacking_edge(1, 0.0)?`.
    pub fn check_lacking_edge(&self, index: i32, tolerance: f64) -> Result<bool, Failure> {
        Ok(self.check_lacking_at(index, tolerance)?.0)
    }

    /// Check adjacent p-curve endpoints using the smaller eligible caller tolerance.
    /// Example: `let (found, previous, current) = analyzer.check_lacking_at(1, 0.0)?;`.
    pub fn check_lacking_at(
        &self,
        index: i32,
        tolerance: f64,
    ) -> Result<(bool, Point2, Point2), Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok((false, Point2::default(), Point2::default()));
        }
        let current = if index > 0 { index } else { self.nb_edges() };
        let previous = if current > 1 {
            current - 1
        } else {
            self.nb_edges()
        };
        let wire = self.wire_data();
        self.check_lacking_pair(&wire.edge(previous)?, &wire.edge(current)?, tolerance)
    }

    /// Query retained aggregate findings; `analyzer.status_lacking(ShapeExtendStatus::Fail3)`.
    pub fn status_lacking(&self, status: ShapeExtendStatus) -> bool {
        ShapeExtend::decode_status(self.fields.read().status_lacking, status)
    }

    fn check_lacking_pair(
        &self,
        previous: &Edge,
        current: &Edge,
        tolerance: f64,
    ) -> Result<(bool, Point2, Point2), Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let first_vertex = analysis.last_vertex(previous);
        let last_vertex = analysis.first_vertex(current);
        if !self.lacking_vertices_match(&first_vertex, &last_vertex)? {
            return Ok((false, Point2::default(), Point2::default()));
        }
        let Some((first, first_tangent)) = self.lacking_endpoint(previous, true)? else {
            return Ok((false, Point2::default(), Point2::default()));
        };
        let Some((last, last_tangent)) = self.lacking_endpoint(current, false)? else {
            return Ok((false, first, Point2::default()));
        };
        let tolerance = lacking_vertex_tolerance(&first_vertex, &last_vertex, tolerance)?;
        let found =
            self.classify_lacking_gap(first, last, first_tangent, last_tangent, tolerance)?;
        Ok((found, first, last))
    }

    fn lacking_vertices_match(&self, first: &Vertex, last: &Vertex) -> Result<bool, Failure> {
        if first.is_null() || last.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        if !compare_vertices(first, last)? {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(false);
        }
        Ok(true)
    }

    fn lacking_endpoint(
        &self,
        edge: &Edge,
        at_end: bool,
    ) -> Result<Option<(Point2, Vector2)>, Failure> {
        let (curve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(edge, &self.face(), true)?;
        if curve.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        let adaptor = Geom2dAdaptorCurve::with_curve(&curve)?;
        let (point, mut tangent) = adaptor.d1(if at_end { last } else { first })?;
        if edge.orientation() == Orientation::Reversed {
            tangent.reverse();
        }
        Ok(Some((point, tangent)))
    }

    fn classify_lacking_gap(
        &self,
        first: Point2,
        last: Point2,
        first_tangent: Vector2,
        last_tangent: Vector2,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        let gap = Vector2::new(last.x() - first.x(), last.y() - first.y());
        let squared = gap.square_magnitude();
        self.fields.write().max_2d = squared;
        let adaptor = self.surface().adaptor3d();
        let tolerance_2d = 2.
            * adaptor
                .u_resolution(tolerance)?
                .max(adaptor.v_resolution(tolerance)?);
        if squared < tolerance_2d * tolerance_2d {
            return Ok(false);
        }
        let distance = squared.sqrt();
        self.store_lacking_distances(distance, tolerance, tolerance_2d);
        self.record_lacking_status(distance, gap, first_tangent, last_tangent)?;
        Ok(true)
    }

    fn record_lacking_status(
        &self,
        distance: f64,
        gap: Vector2,
        first: Vector2,
        last: Vector2,
    ) -> Result<(), Failure> {
        self.set_last_status(ShapeExtendStatus::Done1);
        if distance < precision::p_confusion()
            || tangent_opposes_gap(gap, first)?
            || tangent_opposes_gap(gap, last)?
        {
            self.fields.write().status |= ShapeExtend::encode_status(ShapeExtendStatus::Done2);
        }
        Ok(())
    }

    fn store_lacking_distances(&self, distance: f64, tolerance: f64, tolerance_2d: f64) {
        let mut fields = self.fields.write();
        fields.max_2d = distance;
        fields.max_3d = tolerance * distance / tolerance_2d.max(gp::resolution());
    }
}

fn tangent_opposes_gap(gap: Vector2, tangent: Vector2) -> Result<bool, Failure> {
    Ok(tangent.square_magnitude() > gp::resolution()
        && gap.angle(tangent)?.abs() > 0.9 * std::f64::consts::PI)
}

fn lacking_vertex_tolerance(first: &Vertex, last: &Vertex, requested: f64) -> Result<f64, Failure> {
    let vertex_tolerance =
        BRepTool::vertex_tolerance(first)?.max(BRepTool::vertex_tolerance(last)?);
    Ok(
        if requested > gp::resolution() && requested < vertex_tolerance {
            requested
        } else {
            vertex_tolerance
        },
    )
}
