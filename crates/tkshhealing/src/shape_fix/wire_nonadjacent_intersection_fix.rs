//! Tolerance repair for an explicitly selected pair of wire edges.
use super::{FixRoot, ShapeFixWire};
use crate::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::int_res2d::IntersectionPoint;
use rustycad_tkmath::gp::{Direction3, Line3, Point3};

struct NonadjacentEdges {
    edges: [Edge; 2],
    vertices: [Vertex; 4],
    points: [Point3; 4],
    tolerances: [f64; 4],
    edge_tolerances: [f64; 2],
}

struct VertexPair {
    first_point: Point3,
    first_parameter: f64,
    second_parameter: f64,
    nearest_distance: f64,
    required_vertex_tolerance: f64,
}

#[derive(Default)]
struct PairToleranceUpdates {
    vertices: [f64; 4],
    edges: [f64; 2],
}

impl ShapeFixWire {
    /// Run native `FixIntersectingEdges(num1, num2)` for a selected nonadjacent pair.
    /// Example: `let fixed = fixer.fix_nonadjacent_intersecting_edges(1, 3)?;`.
    // OCCT declares this overload private and has no production call sites.
    #[allow(dead_code)]
    pub(super) fn fix_nonadjacent_intersecting_edges(
        &self,
        first_index: i32,
        second_index: i32,
    ) -> Result<bool, Failure> {
        self.set_last_fix_status(ShapeExtendStatus::Ok);
        let Some((analyzer, intersections)) =
            self.analyze_nonadjacent_pair(first_index, second_index)?
        else {
            return Ok(false);
        };
        let selected = selected_pair_indices(analyzer.nb_edges(), first_index, second_index);
        if selected.0 == selected.1 {
            return Ok(false);
        }
        let edges = NonadjacentEdges::load(&analyzer, first_index, second_index)?;
        self.apply_pair_intersections(&edges, intersections)?;
        Ok(true)
    }

    fn analyze_nonadjacent_pair(
        &self,
        first: i32,
        second: i32,
    ) -> Result<Option<(Handle<ShapeAnalysisWire>, PairIntersections)>, Failure> {
        let analyzer = self.analyzer()?;
        if !analyzer.is_ready() {
            return Ok(None);
        }
        let intersections = analyze_pair(&analyzer, first, second)?;
        self.transfer_intersection_failure(&analyzer);
        if !analyzer.last_check_status(ShapeExtendStatus::Done) {
            return Ok(None);
        }
        Ok(Some((analyzer, intersections)))
    }

    fn apply_pair_intersections(
        &self,
        edges: &NonadjacentEdges,
        intersections: PairIntersections,
    ) -> Result<(), Failure> {
        let mut updates = PairToleranceUpdates::default();
        for index in 1..=intersections.points.len() {
            self.apply_pair_intersection(edges, &intersections, index, &mut updates)?;
        }
        self.apply_pair_edge_tolerances(edges, &mut updates)?;
        apply_vertex_tolerances(edges, updates.vertices)
    }

    fn apply_pair_intersection(
        &self,
        edges: &NonadjacentEdges,
        intersections: &PairIntersections,
        index: usize,
        updates: &mut PairToleranceUpdates,
    ) -> Result<(), Failure> {
        let point = *intersections.points.value(index)?;
        let radius = *intersections.errors.value(index)?;
        let nearest = edges.nearest_vertex_pair(&point)?;
        if nearest.nearest_distance < rustycad_tkernel::precision::confusion() {
            return Ok(());
        }
        let local = local_edge_tolerances(edges, intersections, index, &nearest)?;
        self.record_endpoint_tolerance(edges, point, radius, local, updates);
        Ok(())
    }

    fn record_endpoint_tolerance(
        &self,
        edges: &NonadjacentEdges,
        point: Point3,
        radius: f64,
        local: [f64; 2],
        updates: &mut PairToleranceUpdates,
    ) {
        let rank = nearest_endpoint(edges, point, radius);
        let required = 1.0001 * (point.distance(&edges.points[rank]) + radius);
        if required > self.max_tolerance() {
            self.or_last_fix_status(ShapeExtendStatus::Fail2);
            return;
        }
        self.or_last_fix_status(ShapeExtendStatus::Done1);
        update_tolerance_choice(updates, rank, required, local);
    }

    fn apply_pair_edge_tolerances(
        &self,
        edges: &NonadjacentEdges,
        updates: &mut PairToleranceUpdates,
    ) -> Result<(), Failure> {
        for (edge_index, tolerance) in updates.edges.iter().copied().enumerate() {
            if tolerance <= 0.0 {
                continue;
            }
            update_edge_endpoint_tolerances(edges, edge_index, tolerance, &mut updates.vertices);
            BRepBuilder::default().update_edge_tolerance(&edges.edges[edge_index], tolerance)?;
        }
        Ok(())
    }
}

fn update_tolerance_choice(
    updates: &mut PairToleranceUpdates,
    rank: usize,
    required: f64,
    local: [f64; 2],
) {
    if updates.vertices[rank] >= required {
        return;
    }
    if local[0].max(local[1]) < required && local.iter().any(|value| *value > 0.0) {
        updates.edges[0] = updates.edges[0].max(local[0]);
        updates.edges[1] = updates.edges[1].max(local[1]);
        return;
    }
    updates.vertices[rank] = required;
}

fn selected_pair_indices(count: i32, first: i32, second: i32) -> (i32, i32) {
    (
        if second > 1 { second } else { count },
        if first > 0 { first } else { count },
    )
}

struct PairIntersections {
    parameters: Sequence<IntersectionPoint>,
    points: Sequence<Point3>,
    errors: Sequence<f64>,
}

fn analyze_pair(
    analyzer: &ShapeAnalysisWire,
    first: i32,
    second: i32,
) -> Result<PairIntersections, Failure> {
    let mut result = PairIntersections {
        parameters: Sequence::default(),
        points: Sequence::default(),
        errors: Sequence::default(),
    };
    analyzer.check_intersecting_edges_into(
        first,
        second,
        &mut result.parameters,
        &mut result.points,
        &mut result.errors,
    )?;
    Ok(result)
}

impl NonadjacentEdges {
    fn load(analyzer: &ShapeAnalysisWire, first: i32, second: i32) -> Result<Self, Failure> {
        let wire = analyzer.wire_data();
        let selected = selected_pair_indices(wire.nb_edges(), first, second);
        let edges = [wire.edge(selected.0)?, wire.edge(selected.1)?];
        let vertices = endpoint_vertices(&edges);
        let (points, tolerances) = endpoint_state(&vertices)?;
        let edge_tolerances = [
            BRepTool::edge_tolerance(&edges[0])?,
            BRepTool::edge_tolerance(&edges[1])?,
        ];
        Ok(Self {
            edges,
            vertices,
            points,
            tolerances,
            edge_tolerances,
        })
    }

    fn nearest_vertex_pair(&self, point: &Point3) -> Result<VertexPair, Failure> {
        let mut nearest = VertexPair::default();
        for first in 0..2 {
            for second in 2..4 {
                self.consider_vertex_pair(point, first, second, &mut nearest)?;
            }
        }
        Ok(nearest)
    }

    fn consider_vertex_pair(
        &self,
        point: &Point3,
        first: usize,
        second: usize,
        nearest: &mut VertexPair,
    ) -> Result<(), Failure> {
        let point_distance = point.distance(&self.points[first]);
        let pair_distance = self.points[first].distance(&self.points[second]);
        if nearest.nearest_distance <= point_distance || point_distance <= pair_distance {
            return Ok(());
        }
        *nearest = self.vertex_pair(first, second, point_distance, pair_distance)?;
        Ok(())
    }

    fn vertex_pair(
        &self,
        first: usize,
        second: usize,
        point_distance: f64,
        pair_distance: f64,
    ) -> Result<VertexPair, Failure> {
        Ok(VertexPair {
            first_point: self.points[first],
            first_parameter: BRepTool::parameter(&self.vertices[first], &self.edges[0])?,
            second_parameter: BRepTool::parameter(&self.vertices[second], &self.edges[1])?,
            nearest_distance: point_distance,
            required_vertex_tolerance: pair_distance,
        })
    }
}

impl Default for VertexPair {
    fn default() -> Self {
        Self {
            first_point: Point3::default(),
            first_parameter: 0.0,
            second_parameter: 0.0,
            nearest_distance: f64::MAX,
            required_vertex_tolerance: 0.0,
        }
    }
}

fn endpoint_vertices(edges: &[Edge; 2]) -> [Vertex; 4] {
    let analysis = ShapeAnalysisEdge::new();
    [
        analysis.first_vertex(&edges[0]),
        analysis.last_vertex(&edges[0]),
        analysis.first_vertex(&edges[1]),
        analysis.last_vertex(&edges[1]),
    ]
}

fn endpoint_state(vertices: &[Vertex; 4]) -> Result<([Point3; 4], [f64; 4]), Failure> {
    let mut points = [Point3::default(); 4];
    let mut tolerances = [0.0; 4];
    for index in 0..4 {
        points[index] = BRepTool::pnt(&vertices[index])?;
        tolerances[index] = BRepTool::vertex_tolerance(&vertices[index])?;
    }
    Ok((points, tolerances))
}

fn local_edge_tolerances(
    edges: &NonadjacentEdges,
    intersections: &PairIntersections,
    index: usize,
    nearest: &VertexPair,
) -> Result<[f64; 2], Failure> {
    if should_skip_curve_sampling(nearest) {
        return Ok([0.0; 2]);
    }
    let point = *intersections.points.value(index)?;
    let line = crossing_line(nearest, point)?;
    let Some(curves) = curve_pair(edges)? else {
        return Ok([0.0; 2]);
    };
    let local = sample_curve_pair_tolerances(edges, intersections, index, nearest, line, &curves)?;
    if vertex_tolerance_exceeds_edge_support(edges, nearest, local) {
        return Ok([0.0; 2]);
    }
    Ok(local)
}

fn should_skip_curve_sampling(nearest: &VertexPair) -> bool {
    nearest.nearest_distance < rustycad_tkernel::precision::confusion()
        || nearest.nearest_distance == f64::MAX
}

fn sample_curve_pair_tolerances(
    edges: &NonadjacentEdges,
    intersections: &PairIntersections,
    index: usize,
    nearest: &VertexPair,
    line: Line3,
    curves: &[Handle<dyn GeomCurve>; 2],
) -> Result<[f64; 2], Failure> {
    let first = sample_pair_edge(edges, intersections, index, nearest, line, curves, 0)?;
    let second = sample_pair_edge(edges, intersections, index, nearest, line, curves, 1)?;
    Ok([first, second])
}

fn sample_pair_edge(
    edges: &NonadjacentEdges,
    intersections: &PairIntersections,
    index: usize,
    nearest: &VertexPair,
    line: Line3,
    curves: &[Handle<dyn GeomCurve>; 2],
    edge_index: usize,
) -> Result<f64, Failure> {
    let parameter = pair_parameter(intersections, index, edge_index)?;
    let endpoint = nearest.endpoint_parameter(edge_index);
    sampled_curve_tolerance(
        &curves[edge_index],
        &edges.edges[edge_index],
        line,
        parameter,
        endpoint,
    )
}

fn pair_parameter(
    intersections: &PairIntersections,
    index: usize,
    edge_index: usize,
) -> Result<f64, Failure> {
    let parameters = intersections.parameters.value(index)?;
    Ok(if edge_index == 0 {
        parameters.param_on_first()
    } else {
        parameters.param_on_second()
    })
}

impl VertexPair {
    fn endpoint_parameter(&self, edge_index: usize) -> f64 {
        if edge_index == 0 {
            self.first_parameter
        } else {
            self.second_parameter
        }
    }
}

fn vertex_tolerance_exceeds_edge_support(
    edges: &NonadjacentEdges,
    nearest: &VertexPair,
    local: [f64; 2],
) -> bool {
    nearest.required_vertex_tolerance > local[0].max(edges.edge_tolerances[0])
        || nearest.required_vertex_tolerance > local[1].max(edges.edge_tolerances[1])
}

fn crossing_line(nearest: &VertexPair, point: Point3) -> Result<Line3, Failure> {
    let direction = Direction3::new(
        point.x() - nearest.first_point.x(),
        point.y() - nearest.first_point.y(),
        point.z() - nearest.first_point.z(),
    )?;
    Ok(Line3::new(nearest.first_point, direction))
}

fn curve_pair(edges: &NonadjacentEdges) -> Result<Option<[Handle<dyn GeomCurve>; 2]>, Failure> {
    let curves = [
        BRepTool::curve(&edges.edges[0])?.0,
        BRepTool::curve(&edges.edges[1])?.0,
    ];
    if curves.iter().any(Handle::is_null) {
        return Ok(None);
    }
    Ok(Some(curves))
}

fn sampled_curve_tolerance(
    curve: &Handle<dyn GeomCurve>,
    edge: &Edge,
    line: Line3,
    crossing_parameter: f64,
    endpoint_parameter: f64,
) -> Result<f64, Failure> {
    let Some(curve) = curve.get() else {
        return Ok(0.0);
    };
    let mut maximum = 0.0_f64;
    for sample in 2..19 {
        let parameter = endpoint_parameter
            + f64::from(sample) * 0.05 * (crossing_parameter - endpoint_parameter);
        maximum = sampled_curve_maximum(curve, edge, line, parameter, maximum)?;
    }
    Ok(maximum)
}

fn sampled_curve_maximum(
    curve: &dyn GeomCurve,
    edge: &Edge,
    line: Line3,
    parameter: f64,
    maximum: f64,
) -> Result<f64, Failure> {
    let distance = line.distance(curve.value(parameter)?) * 2.0000001;
    if distance > BRepTool::edge_tolerance(edge)? && distance > maximum {
        return Ok(distance);
    }
    Ok(maximum)
}

fn nearest_endpoint(edges: &NonadjacentEdges, point: Point3, radius: f64) -> usize {
    let mut selected = 0;
    let mut minimum = f64::MAX;
    for index in 0..4 {
        let required = 1.0001 * (point.distance(&edges.points[index]) + radius);
        if required < minimum {
            minimum = required;
            selected = index;
        }
    }
    selected
}

fn update_edge_endpoint_tolerances(
    edges: &NonadjacentEdges,
    edge_index: usize,
    tolerance: f64,
    new_tolerances: &mut [f64; 4],
) {
    let start = edge_index * 2;
    for (index, new_tolerance) in new_tolerances.iter_mut().enumerate().skip(start).take(2) {
        if tolerance > edges.tolerances[index].max(*new_tolerance) {
            *new_tolerance = tolerance;
        }
    }
}

fn apply_vertex_tolerances(
    edges: &NonadjacentEdges,
    new_tolerances: [f64; 4],
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for (index, tolerance) in new_tolerances.iter().enumerate() {
        if *tolerance > 0.0 {
            builder.update_vertex_tolerance(&edges.vertices[index], *tolerance)?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "wire_nonadjacent_intersection_fix_tests.rs"]
mod tests;
