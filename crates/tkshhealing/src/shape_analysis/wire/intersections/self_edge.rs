use super::*;

struct SelfIntersectionInput {
    edge: Edge,
    adapted: Geom2dAdaptorCurve,
    intersections: GInter,
}

struct EndpointBounds {
    points: (Point3, Point3),
    tolerances: (f64, f64),
}

impl ShapeAnalysisWire {
    /// Detect an edge's interior self-intersections; `analyzer.check_self_intersecting_edge(1)?`.
    pub fn check_self_intersecting_edge(&self, index: i32) -> Result<bool, Failure> {
        self.check_self_intersecting_edge_into(
            index,
            &mut Sequence::default(),
            &mut Sequence::default(),
        )
    }

    /// Clear and fill native intersection records and spatial points.
    /// Example: `analyzer.check_self_intersecting_edge_into(1, &mut uv, &mut xyz)?`.
    pub fn check_self_intersecting_edge_into(
        &self,
        index: i32,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
    ) -> Result<bool, Failure> {
        points_2d.clear(None);
        points_3d.clear(None);
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let Some(input) = self.prepare_self_intersection(index)? else {
            return Ok(false);
        };
        self.collect_self_intersections(&input, points_2d, points_3d)
    }

    fn prepare_self_intersection(
        &self,
        index: i32,
    ) -> Result<Option<SelfIntersectionInput>, Failure> {
        let edge = self
            .wire_data()
            .edge(if index > 0 { index } else { self.nb_edges() })?;
        let (curve, first, last) =
            ShapeAnalysisEdge::new().pcurve_on_face(&edge, &self.face(), false)?;
        if curve.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail1);
            return Ok(None);
        }
        if (first - last).abs() <= precision::p_confusion() {
            return Ok(None);
        }
        let domain = intersection_domain(&curve, first, last)?;
        let adapted = Geom2dAdaptorCurve::with_curve(&curve)?;
        let mut intersections = GInter::new();
        intersections.perform_self_on(
            &adapted,
            &domain,
            INTERSECTION_TOLERANCE,
            INTERSECTION_TOLERANCE,
        )?;
        Ok(Some(SelfIntersectionInput {
            edge,
            adapted,
            intersections,
        }))
    }

    fn collect_self_intersections(
        &self,
        input: &SelfIntersectionInput,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
    ) -> Result<bool, Failure> {
        if !input.intersections.is_done() {
            return Ok(false);
        }
        let Some(bounds) = self.self_intersection_endpoints(&input.edge)? else {
            return Ok(false);
        };
        for index in 1..=input.intersections.nb_points()? {
            let point = input.intersections.point(index)?;
            let Some(spatial) = self.self_intersection_point(input, &point, &bounds)? else {
                continue;
            };
            points_2d.append(point);
            points_3d.append(spatial);
            self.set_last_status(ShapeExtendStatus::Done1);
        }
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }

    fn self_intersection_endpoints(&self, edge: &Edge) -> Result<Option<EndpointBounds>, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (first, last) = (analysis.first_vertex(edge), analysis.last_vertex(edge));
        if first.is_null() || last.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(None);
        }
        let tolerances = (
            BRepTool::vertex_tolerance(&first)?,
            BRepTool::vertex_tolerance(&last)?,
        );
        let points = (BRepTool::pnt(&first)?, BRepTool::pnt(&last)?);
        Ok(Some(EndpointBounds { points, tolerances }))
    }

    fn self_intersection_point(
        &self,
        input: &SelfIntersectionInput,
        point: &IntersectionPoint,
        bounds: &EndpointBounds,
    ) -> Result<Option<Point3>, Failure> {
        if !has_middle_transition(point) {
            return Ok(None);
        }
        let spatial =
            self.point_on_intersected_edge(&input.edge, &input.adapted, point.param_on_first())?;
        if !bounds.outside_tolerances(spatial) {
            return Ok(None);
        }
        Ok(Some(spatial))
    }
}

impl EndpointBounds {
    fn outside_tolerances(&self, point: Point3) -> bool {
        self.points.0.square_distance(&point) > self.tolerances.0 * self.tolerances.0
            && self.points.1.square_distance(&point) > self.tolerances.1 * self.tolerances.1
    }
}
