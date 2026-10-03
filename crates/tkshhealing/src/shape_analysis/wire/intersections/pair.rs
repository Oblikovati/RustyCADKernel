use super::*;

pub(super) struct EdgePairCurves {
    pub(super) edges: [Edge; 2],
    pub(super) pcurves: [Handle<dyn Geom2dCurve>; 2],
    pub(super) ranges: [(f64, f64); 2],
}

struct EdgePairIntersections {
    adapted: [Geom2dAdaptorCurve; 2],
    intersections: GInter,
}

struct PairEndpointBounds {
    points: [Point3; 4],
    tolerances: [f64; 4],
}

impl ShapeAnalysisWire {
    /// Check arbitrary edge indices; `analyzer.check_intersecting_edges(1, 3)?`.
    pub fn check_intersecting_edges(&self, first: i32, second: i32) -> Result<bool, Failure> {
        self.check_intersecting_edges_into(
            first,
            second,
            &mut Sequence::default(),
            &mut Sequence::default(),
            &mut Sequence::default(),
        )
    }

    /// Fill pair intersections, retaining outputs on native pre-geometry exits.
    /// Example: `analyzer.check_intersecting_edges_into(1, 3, &mut uv, &mut xyz, &mut errors)?`.
    pub fn check_intersecting_edges_into(
        &self,
        first: i32,
        second: i32,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
        errors: &mut Sequence<f64>,
    ) -> Result<bool, Failure> {
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() {
            return Ok(false);
        }
        let Some(curves) = self.pair_intersection_curves(first, second)? else {
            return Ok(false);
        };
        points_2d.clear(None);
        points_3d.clear(None);
        errors.clear(None);
        let bounds = PairEndpointBounds::new(&curves.edges)?;
        let intersections = EdgePairIntersections::new(&curves)?;
        self.collect_pair_intersections(
            &curves,
            &intersections,
            &bounds,
            points_2d,
            points_3d,
            errors,
        )
    }

    fn pair_intersection_curves(
        &self,
        first: i32,
        second: i32,
    ) -> Result<Option<EdgePairCurves>, Failure> {
        let wire = self.wire_data();
        let edges = [
            wire.edge(if first > 0 { first } else { wire.nb_edges() })?,
            wire.edge(if second > 0 { second } else { wire.nb_edges() })?,
        ];
        self.intersection_curves_for_edges(edges)
    }

    pub(super) fn intersection_curves_for_edges(
        &self,
        edges: [Edge; 2],
    ) -> Result<Option<EdgePairCurves>, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (pcurve1, a1, b1) = analysis.pcurve_on_face(&edges[0], &self.face(), false)?;
        if pcurve1.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        let (pcurve2, a2, b2) = analysis.pcurve_on_face(&edges[1], &self.face(), false)?;
        if pcurve2.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail3);
            return Ok(None);
        }
        if (a1 - b1).abs() <= precision::p_confusion()
            || (a2 - b2).abs() <= precision::p_confusion()
        {
            return Ok(None);
        }
        Ok(Some(EdgePairCurves {
            edges,
            pcurves: [pcurve1, pcurve2],
            ranges: [(a1, b1), (a2, b2)],
        }))
    }

    fn collect_pair_intersections(
        &self,
        curves: &EdgePairCurves,
        found: &EdgePairIntersections,
        bounds: &PairEndpointBounds,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
        errors: &mut Sequence<f64>,
    ) -> Result<bool, Failure> {
        let intersections = &found.intersections;
        if !intersections.is_done() {
            return Ok(false);
        }
        let count = intersections.nb_points()?;
        for index in 1..=count + intersections.nb_segments()? {
            let Some(point) = intersection_candidate(intersections, count, index)? else {
                continue;
            };
            let p1 = self.point_on_intersected_edge(
                &curves.edges[0],
                &found.adapted[0],
                point.param_on_first(),
            )?;
            let p2 = self.point_on_intersected_edge(
                &curves.edges[1],
                &found.adapted[1],
                point.param_on_second(),
            )?;
            if bounds.contains_on_both(p1, p2) {
                continue;
            }
            points_2d.append(point);
            points_3d.append(intersection_midpoint(p1, p2));
            errors.append(0.5 * p1.distance(&p2));
            self.set_last_status(ShapeExtendStatus::Done1);
        }
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }
}

impl EdgePairIntersections {
    fn new(curves: &EdgePairCurves) -> Result<Self, Failure> {
        let first =
            intersection_domain(&curves.pcurves[0], curves.ranges[0].0, curves.ranges[0].1)?;
        let second =
            intersection_domain(&curves.pcurves[1], curves.ranges[1].0, curves.ranges[1].1)?;
        let adapted = [
            Geom2dAdaptorCurve::with_curve(&curves.pcurves[0])?,
            Geom2dAdaptorCurve::with_curve(&curves.pcurves[1])?,
        ];
        let mut intersections = GInter::new();
        intersections.perform(
            &adapted[0],
            &first,
            &adapted[1],
            &second,
            INTERSECTION_TOLERANCE,
            INTERSECTION_TOLERANCE,
        )?;
        Ok(Self {
            adapted,
            intersections,
        })
    }
}

impl PairEndpointBounds {
    fn new(edges: &[Edge; 2]) -> Result<Self, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let vertices = [
            analysis.first_vertex(&edges[0]),
            analysis.last_vertex(&edges[0]),
            analysis.first_vertex(&edges[1]),
            analysis.last_vertex(&edges[1]),
        ];
        let mut bounds = Self {
            points: [Point3::default(); 4],
            tolerances: [0.; 4],
        };
        for (index, vertex) in vertices.iter().enumerate() {
            bounds.points[index] = BRepTool::pnt(vertex)?;
            bounds.tolerances[index] = BRepTool::vertex_tolerance(vertex)?;
        }
        Ok(bounds)
    }

    fn contains_on_both(&self, first: Point3, second: Point3) -> bool {
        self.contains_endpoint(first, 0) && self.contains_endpoint(second, 2)
    }

    fn contains_endpoint(&self, point: Point3, start: usize) -> bool {
        (start..start + 2).any(|index| {
            point.square_distance(&self.points[index])
                < self.tolerances[index] * self.tolerances[index]
        })
    }
}
