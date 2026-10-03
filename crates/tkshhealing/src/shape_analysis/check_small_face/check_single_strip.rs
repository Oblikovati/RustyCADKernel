use super::spot::{ShapeAnalysisCheckSmallFace, axis_is_small, update_bounds};
use crate::shape_extend::{ShapeExtend, ShapeExtendStatus};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::{Explorer, vertices as edge_vertices};
use rustycad_tkbrep::topods::{Edge, Face, Vertex, edge, vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

type EdgeCurveRange = (
    rustycad_tkernel::standard::Handle<dyn GeomCurve>,
    (f64, f64),
);

enum SameVertexEdge {
    NotSameVertex,
    Small,
    TooLarge,
}

impl ShapeAnalysisCheckSmallFace {
    /// Validates the two-edge, two-vertex single-strip topology and edge spacing.
    /// `analyzer.check_single_strip(&face, &mut first, &mut second, -1.0)?`.
    pub fn check_single_strip(
        &mut self,
        face: &Face,
        first_edge: &mut Edge,
        second_edge: &mut Edge,
        tolerance: f64,
    ) -> Result<bool, Failure> {
        if !has_at_most_two_unique_vertices(face)? {
            return Ok(false);
        }
        let (mut first_vertex, mut second_vertex) = (Vertex::default(), Vertex::default());
        let mut effective_tolerance = tolerance;
        let Some(count) = find_great_edges(
            face,
            tolerance,
            &mut effective_tolerance,
            first_edge,
            second_edge,
            (&mut first_vertex, &mut second_vertex),
        )?
        else {
            return Ok(false);
        };
        if count < 2 || !self.check_strip_edges(first_edge, second_edge, tolerance, &mut 0.0)? {
            return Ok(false);
        }
        self.strip_status = ShapeExtend::encode_status(ShapeExtendStatus::Done3);
        Ok(true)
    }
}

fn has_at_most_two_unique_vertices(face: &Face) -> Result<bool, Failure> {
    let mut unique = Vec::with_capacity(2);
    let mut iterator = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )?;
    while iterator.more() {
        add_unique_vertex(&mut unique, vertex(iterator.current())?);
        if unique.len() > 2 {
            return Ok(false);
        }
        iterator.next()?;
    }
    Ok(true)
}

fn add_unique_vertex(unique: &mut Vec<Vertex>, candidate: Vertex) {
    if unique.iter().any(|known| known.is_same(&candidate)) {
        return;
    }
    unique.push(candidate);
}

fn find_great_edges(
    face: &Face,
    tolerance: f64,
    effective_tolerance: &mut f64,
    first_edge: &mut Edge,
    second_edge: &mut Edge,
    vertices: (&mut Vertex, &mut Vertex),
) -> Result<Option<i32>, Failure> {
    let mut count = 0;
    let mut iterator = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )?;
    while iterator.more() {
        let current = edge(iterator.current())?;
        if !is_duplicate_first(count, &current, first_edge) {
            let same_vertex = classify_same_vertex_edge(&current, tolerance, effective_tolerance)?;
            if !handle_edge_class(
                same_vertex,
                &current,
                &mut count,
                first_edge,
                second_edge,
                (&mut *vertices.0, &mut *vertices.1),
            )? {
                return Ok(None);
            }
        }
        iterator.next()?;
    }
    Ok(Some(count))
}

fn is_duplicate_first(count: i32, current: &Edge, first: &Edge) -> bool {
    count == 1 && current.is_same(first)
}

fn classify_same_vertex_edge(
    edge: &Edge,
    tolerance: f64,
    effective_tolerance: &mut f64,
) -> Result<SameVertexEdge, Failure> {
    let (first, last) = edge_vertices(edge, true);
    grow_tolerance(&first, &last, tolerance, effective_tolerance)?;
    if !first.is_same(&last) {
        return Ok(SameVertexEdge::NotSameVertex);
    }
    if BRepTool::degenerated(edge)? {
        return Ok(SameVertexEdge::Small);
    }
    let Some((curve, range)) = curve_range(edge)? else {
        return Ok(SameVertexEdge::Small);
    };
    if curve_points_are_small(&curve, range, *effective_tolerance)? {
        return Ok(SameVertexEdge::Small);
    }
    Ok(SameVertexEdge::TooLarge)
}

fn handle_edge_class(
    class: SameVertexEdge,
    current: &Edge,
    count: &mut i32,
    first_edge: &mut Edge,
    second_edge: &mut Edge,
    endpoints: (&mut Vertex, &mut Vertex),
) -> Result<bool, Failure> {
    match class {
        SameVertexEdge::Small => return Ok(true),
        SameVertexEdge::TooLarge => return Ok(false),
        SameVertexEdge::NotSameVertex => {}
    }
    let stop = record_great_edge(current, count, first_edge, second_edge, endpoints)?;
    Ok(!stop)
}

fn grow_tolerance(
    first: &Vertex,
    last: &Vertex,
    tolerance: f64,
    effective: &mut f64,
) -> Result<(), Failure> {
    if tolerance < 0.0 {
        *effective = maximum_native(*effective, BRepTool::vertex_tolerance(first)?);
        *effective = maximum_native(*effective, BRepTool::vertex_tolerance(last)?);
    }
    Ok(())
}

fn curve_range(edge: &Edge) -> Result<Option<EdgeCurveRange>, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    if curve.get().is_some() {
        return Ok(Some((curve, (first, last))));
    }
    Ok(None)
}

fn curve_points_are_small(
    curve: &rustycad_tkernel::standard::Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
) -> Result<bool, Failure> {
    let points = [
        curve.value(range.0)?,
        curve.value(range.1)?,
        curve.value((range.0 + range.1) / 2.0)?,
    ];
    let (mut minimum, mut maximum) = (Point3::default(), Point3::default());
    for (index, point) in points.into_iter().enumerate() {
        update_bounds(&mut minimum, &mut maximum, point, index == 0);
    }
    Ok(axis_is_small(maximum.x() - minimum.x(), tolerance)
        && axis_is_small(maximum.y() - minimum.y(), tolerance)
        && axis_is_small(maximum.z() - minimum.z(), tolerance))
}

fn record_great_edge(
    current: &Edge,
    count: &mut i32,
    first_edge: &mut Edge,
    second_edge: &mut Edge,
    vertices: (&mut Vertex, &mut Vertex),
) -> Result<bool, Failure> {
    let (start, end) = edge_vertices(current, true);
    if start.is_same(&end) {
        return Ok(false);
    }
    *count += 1;
    if *count > 2 {
        return Ok(true);
    }
    if !assign_edge_pair(*count, current, first_edge, second_edge, vertices) {
        return Ok(true);
    }
    Ok(false)
}

fn assign_edge_pair(
    count: i32,
    current: &Edge,
    first_edge: &mut Edge,
    second_edge: &mut Edge,
    vertices: (&mut Vertex, &mut Vertex),
) -> bool {
    let (start, end) = edge_vertices(current, true);
    if count == 1 {
        *vertices.0 = start;
        *vertices.1 = end;
        *first_edge = current.clone();
        return true;
    }
    let forward_pair = vertices.0.is_same(&start) && vertices.1.is_same(&end);
    let reverse_pair = vertices.0.is_same(&end) && vertices.1.is_same(&start);
    if !forward_pair && !reverse_pair {
        return false;
    }
    *second_edge = current.clone();
    true
}

fn maximum_native(first: f64, second: f64) -> f64 {
    if second > first { second } else { first }
}
