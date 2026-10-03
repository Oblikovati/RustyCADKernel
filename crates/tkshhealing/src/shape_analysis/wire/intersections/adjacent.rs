use super::pair::EdgePairCurves;
use super::*;
use rustycad_tkbrep::{brep_tools, topods::Vertex};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::Orientation};

struct AdjacentVertices {
    shared: [Vertex; 2],
    outer: [Vertex; 2],
}

struct AdjacentEndpointState {
    shared_point: Point3,
    tolerance: f64,
    outer: [Vertex; 2],
}

struct AdjacentIntersections {
    adapted: [Geom2dAdaptorCurve; 2],
    intersections: GInter,
    reversed: bool,
}

struct AdjacentSelection {
    curves: EdgePairCurves,
    intersection: AdjacentIntersections,
    endpoints: AdjacentEndpointState,
    tolerance: f64,
    lacking: Option<bool>,
}

impl ShapeAnalysisWire {
    /// Check the preceding edge against `index`; `analyzer.check_adjacent_intersections(1)?`.
    pub fn check_adjacent_intersections(&self, index: i32) -> Result<bool, Failure> {
        self.check_adjacent_intersections_into(
            index,
            &mut Sequence::default(),
            &mut Sequence::default(),
            &mut Sequence::default(),
        )
    }

    /// Clear outputs before checking adjacent-edge connections and intersections.
    /// Example: `analyzer.check_adjacent_intersections_into(1, &mut uv, &mut xyz, &mut errors)?`.
    pub fn check_adjacent_intersections_into(
        &self,
        index: i32,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
        errors: &mut Sequence<f64>,
    ) -> Result<bool, Failure> {
        points_2d.clear(None);
        points_3d.clear(None);
        errors.clear(None);
        self.set_last_status(ShapeExtendStatus::Ok);
        if !self.is_ready() || self.nb_edges() < 2 {
            return Ok(false);
        }
        let Some(mut selection) = self.prepare_adjacent_selection(index)? else {
            return Ok(false);
        };
        self.collect_adjacent_intersections(&mut selection, points_2d, points_3d, errors)
    }

    fn prepare_adjacent_selection(&self, index: i32) -> Result<Option<AdjacentSelection>, Failure> {
        let (previous, current) = self.gap_pair(index)?;
        let edges = [previous, current];
        let Some(vertices) = self.adjacent_endpoint_vertices(&edges)? else {
            return Ok(None);
        };
        let Some(curves) = self.intersection_curves_for_edges(edges)? else {
            return Ok(None);
        };
        self.finish_adjacent_selection(index, curves, vertices)
    }

    fn finish_adjacent_selection(
        &self,
        index: i32,
        curves: EdgePairCurves,
        vertices: AdjacentVertices,
    ) -> Result<Option<AdjacentSelection>, Failure> {
        let endpoints = vertices.measure()?;
        let intersection = AdjacentIntersections::new(&curves, index == 1)?;
        if !intersection.intersections.is_done() {
            return Ok(None);
        }
        let tolerance =
            adjacent_filter_tolerance(&curves.edges, endpoints.tolerance, self.precision())?;
        Ok(Some(AdjacentSelection {
            curves,
            intersection,
            endpoints,
            tolerance,
            lacking: None,
        }))
    }

    fn adjacent_endpoint_vertices(
        &self,
        edges: &[Edge; 2],
    ) -> Result<Option<AdjacentVertices>, Failure> {
        let analysis = ShapeAnalysisEdge::new();
        let (first, second) = (
            analysis.last_vertex(&edges[0]),
            analysis.first_vertex(&edges[1]),
        );
        if first.is_null() || second.is_null() {
            self.set_last_status(ShapeExtendStatus::Fail1);
            return Ok(None);
        }
        if !brep_tools::compare_vertices(&first, &second)? {
            self.set_last_status(ShapeExtendStatus::Fail2);
            return Ok(None);
        }
        let outer = [
            analysis.first_vertex(&edges[0]),
            analysis.last_vertex(&edges[1]),
        ];
        Ok(Some(AdjacentVertices {
            shared: [first, second],
            outer,
        }))
    }

    fn collect_adjacent_intersections(
        &self,
        selection: &mut AdjacentSelection,
        points_2d: &mut Sequence<IntersectionPoint>,
        points_3d: &mut Sequence<Point3>,
        errors: &mut Sequence<f64>,
    ) -> Result<bool, Failure> {
        let count = selection.intersection.intersections.nb_points()?;
        let total = count + selection.intersection.intersections.nb_segments()?;
        for index in 1..=total {
            let Some(point) =
                intersection_candidate(&selection.intersection.intersections, count, index)?
            else {
                continue;
            };
            let Some((first, second)) = self.selected_adjacent_point(selection, &point)? else {
                continue;
            };
            points_2d.append(point);
            points_3d.append(intersection_midpoint(first, second));
            errors.append(0.5 * first.distance(&second));
            self.set_last_status(ShapeExtendStatus::Done1);
        }
        Ok(self.last_check_status(ShapeExtendStatus::Done))
    }

    fn selected_adjacent_point(
        &self,
        selection: &mut AdjacentSelection,
        point: &IntersectionPoint,
    ) -> Result<Option<(Point3, Point3)>, Failure> {
        let parameters = adjacent_parameters(point, selection.intersection.reversed);
        if !parameters_within_ranges(parameters, selection.curves.ranges) {
            return Ok(None);
        }
        let first = self.point_on_intersected_edge(
            &selection.curves.edges[0],
            &selection.intersection.adapted[0],
            parameters.0,
        )?;
        let second = self.point_on_intersected_edge(
            &selection.curves.edges[1],
            &selection.intersection.adapted[1],
            parameters.1,
        )?;
        let distance_squared = first
            .square_distance(&selection.endpoints.shared_point)
            .max(second.square_distance(&selection.endpoints.shared_point));
        if selection.lacking.is_none() {
            selection.lacking = Some(self.adjacent_connection_lacking(selection)?);
        }
        if !(distance_squared > selection.tolerance * selection.tolerance
            || selection.lacking == Some(true))
        {
            return Ok(None);
        }
        if !outer_endpoints_allow(&selection.endpoints, distance_squared, first, second)? {
            return Ok(None);
        }
        Ok(Some((first, second)))
    }

    fn adjacent_connection_lacking(&self, selection: &AdjacentSelection) -> Result<bool, Failure> {
        let curves = &selection.curves;
        let end1 =
            curves.pcurves[0].value(if curves.edges[0].orientation() == Orientation::Forward {
                curves.ranges[0].1
            } else {
                curves.ranges[0].0
            })?;
        let end2 =
            curves.pcurves[1].value(if curves.edges[1].orientation() == Orientation::Forward {
                curves.ranges[1].0
            } else {
                curves.ranges[1].1
            })?;
        let surface = self.surface().adaptor3d();
        let tolerance = 2.
            * surface
                .u_resolution(selection.endpoints.tolerance)?
                .max(surface.v_resolution(selection.endpoints.tolerance)?);
        Ok(end1.square_distance(end2) >= tolerance * tolerance)
    }
}

impl AdjacentIntersections {
    fn new(curves: &EdgePairCurves, reversed: bool) -> Result<Self, Failure> {
        let adapted = [
            Geom2dAdaptorCurve::with_curve(&curves.pcurves[0])?,
            Geom2dAdaptorCurve::with_curve(&curves.pcurves[1])?,
        ];
        let first = adapted_domain(&adapted[0], curves.ranges[0])?;
        let second = adapted_domain(&adapted[1], curves.ranges[1])?;
        let mut intersections = GInter::new();
        // CSR CTS17806: OCCT intersects edges in ascending wire order at closure.
        if reversed {
            intersections.perform(
                &adapted[1],
                &second,
                &adapted[0],
                &first,
                INTERSECTION_TOLERANCE,
                INTERSECTION_TOLERANCE,
            )?;
        } else {
            intersections.perform(
                &adapted[0],
                &first,
                &adapted[1],
                &second,
                INTERSECTION_TOLERANCE,
                INTERSECTION_TOLERANCE,
            )?;
        }
        Ok(Self {
            adapted,
            intersections,
            reversed,
        })
    }
}

fn adapted_domain(curve: &Geom2dAdaptorCurve, range: (f64, f64)) -> Result<Domain, Failure> {
    Ok(Domain::bounded(
        curve.value(range.0)?,
        range.0,
        INTERSECTION_TOLERANCE,
        curve.value(range.1)?,
        range.1,
        INTERSECTION_TOLERANCE,
    ))
}

fn adjacent_parameters(point: &IntersectionPoint, reversed: bool) -> (f64, f64) {
    if reversed {
        return (point.param_on_second(), point.param_on_first());
    }
    (point.param_on_first(), point.param_on_second())
}

fn parameters_within_ranges(parameters: (f64, f64), ranges: [(f64, f64); 2]) -> bool {
    let tolerance = precision::p_confusion();
    !(ranges[0].0 - parameters.0 > tolerance
        || parameters.0 - ranges[0].1 > tolerance
        || ranges[1].0 - parameters.1 > tolerance
        || parameters.1 - ranges[1].1 > tolerance)
}

fn adjacent_filter_tolerance(
    edges: &[Edge; 2],
    tolerance: f64,
    precision: f64,
) -> Result<f64, Failure> {
    let first = if BRepTool::same_parameter(&edges[0])? {
        BRepTool::edge_tolerance(&edges[0])?
    } else {
        tolerance
    };
    let second = if BRepTool::same_parameter(&edges[1])? {
        BRepTool::edge_tolerance(&edges[1])?
    } else {
        tolerance
    };
    Ok(tolerance.min(first.max(second).max(precision)))
}

fn outer_endpoints_allow(
    endpoints: &AdjacentEndpointState,
    distance_squared: f64,
    first: Point3,
    second: Point3,
) -> Result<bool, Failure> {
    if !brep_tools::compare_vertices(&endpoints.outer[0], &endpoints.outer[1])? {
        return Ok(true);
    }
    Ok(distance_squared
        < intersection_midpoint(first, second)
            .square_distance(&BRepTool::pnt(&endpoints.outer[0])?))
}

impl AdjacentVertices {
    fn measure(self) -> Result<AdjacentEndpointState, Failure> {
        let tolerance = BRepTool::vertex_tolerance(&self.shared[0])?
            .max(BRepTool::vertex_tolerance(&self.shared[1])?);
        let shared_point = BRepTool::pnt(&self.shared[0])?;
        Ok(AdjacentEndpointState {
            shared_point,
            tolerance,
            outer: self.outer,
        })
    }
}
