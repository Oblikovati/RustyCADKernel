//! `MergeEdges`, `MergeSeq`, `generateSubSeq`, `IsMergingPossible` and
//! `GetLineEdgePoints` of ShapeUpgrade_UnifySameDomain.cxx: the edges chained through
//! their vertices, the chains cut where merging is impossible, and each sub-chain
//! replaced by its merged edge in the context.
use super::geometry::is_linear;
use super::seams::find_closest_points;
use super::{ShapeUpgradeSubSequenceOfEdges, ShapeUpgradeUnifySameDomain, UnifyState};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomLine, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The direction of the first straight edge of a sub-chain (`GetLineEdgePoints`).
#[derive(Clone, Copy, Default)]
struct LineDirection {
    ok: bool,
    first_point: Point3,
    direction: Vector3,
}

/// `GetLineEdgePoints(theInpEdge, theFirstPoint, theDirectionVec)`: the start point and
/// unit direction of an edge on a Geom_Line (possibly trimmed), in the edge direction.
fn get_line_edge_points(edge: &Edge, previous: LineDirection) -> Result<LineDirection, Failure> {
    let (curve, mut first, mut last) = BRepTool::curve(edge)?;
    if curve.is_null() {
        return Ok(LineDirection {
            ok: false,
            ..previous
        });
    }
    let mut curve = curve;
    if let Some(trimmed) = curve.cast::<GeomTrimmedCurve>().get() {
        curve = trimmed.basis_curve();
    }
    if curve.cast::<GeomLine>().is_null() {
        return Ok(LineDirection {
            ok: false,
            ..previous
        });
    }
    if edge.orientation() == Orientation::Reversed {
        std::mem::swap(&mut first, &mut last);
    }
    let line = curve.get().ok_or_else(super::geometry::null_surface)?;
    let first_point = line.value(first)?;
    let mut direction = Vector3::between(first_point, line.value(last)?);
    direction.normalize()?;
    Ok(LineDirection {
        ok: true,
        first_point,
        direction,
    })
}

/// The tangent at the junction: at the end of the first edge, at the start of the second
/// one, in the edge directions.
fn junction_tangent(
    curve: &BRepAdaptorCurve,
    edge: &Edge,
    at_end: bool,
) -> Result<(Point3, Vector3), Failure> {
    let forward = edge.orientation() == Orientation::Forward;
    let parameter = if forward == at_end {
        curve.last_parameter()?
    } else {
        curve.first_parameter()?
    };
    let (point, mut tangent) = curve.d1(parameter)?;
    if !forward {
        tangent.reverse();
    }
    Ok((point, tangent))
}

/// The tolerances and the vertices merging must keep.
#[derive(Clone, Copy)]
struct MergeLimits<'a> {
    ang_tol: f64,
    lin_tol: f64,
    avoid: &'a MapOfShape,
    vf_map: &'a IndexedDataMapOfShapeListOfShape,
}

/// `IsMergingPossible(edge1, edge2, ...)`: two degenerated edges touching in 2D, or two
/// edges of compatible kinds meeting tangentially at a vertex not to avoid (with the
/// accumulated deviation of a straight chain within the tolerances).
fn is_merging_possible(
    edge1: &Edge,
    edge2: &Edge,
    l: MergeLimits<'_>,
    line: LineDirection,
) -> Result<bool, Failure> {
    let (deg1, deg2) = (BRepTool::degenerated(edge1)?, BRepTool::degenerated(edge2)?);
    if deg1 && deg2 {
        let Some(closest) = find_closest_points(edge1, edge2, l.vf_map)? else {
            return Ok(false);
        };
        return Ok(closest.min_sq_dist <= precision::square_confusion());
    }
    if deg1 || deg2 {
        return Ok(false);
    }
    let common = topexp::last_vertex(edge1, true);
    if common.is_null() || l.avoid.contains(common.as_ref()) {
        return Ok(false);
    }
    let (ade1, ade2) = (
        BRepAdaptorCurve::with_edge(edge1)?,
        BRepAdaptorCurve::with_edge(edge2)?,
    );
    if !compatible_kinds(&ade1, &ade2)? {
        return Ok(false);
    }
    let (_, diff1) = junction_tangent(&ade1, edge1, true)?;
    let (_, diff2) = junction_tangent(&ade2, edge2, false)?;
    if diff1.angle(diff2)? > l.ang_tol {
        return Ok(false);
    }
    if !(line.ok && ade2.get_type()? == CurveType::Line) {
        return Ok(true);
    }
    within_line_deviation(&ade2, edge2, diff2, l, line)
}

/// Concentric circles, two straight edges, two splines, or two curves of one kind.
fn compatible_kinds(ade1: &BRepAdaptorCurve, ade2: &BRepAdaptorCurve) -> Result<bool, Failure> {
    let (t1, t2) = (ade1.get_type()?, ade2.get_type()?);
    if t1 == CurveType::Circle && t2 == CurveType::Circle {
        let gap = ade1
            .circle()?
            .location()
            .distance(&ade2.circle()?.location());
        if gap > precision::confusion() {
            return Ok(false);
        }
    }
    let spline = |t: CurveType| matches!(t, CurveType::BezierCurve | CurveType::BSplineCurve);
    let linear = is_linear(ade1)?.is_some() && is_linear(ade2)?.is_some();
    Ok(linear || (spline(t1) && spline(t2)) || t1 == t2)
}

/// "Check that the accumulated deflection does not exceed the linear tolerance" and the
/// angles of the first and last edges with the resulting line.
fn within_line_deviation(
    ade2: &BRepAdaptorCurve,
    edge2: &Edge,
    diff2: Vector3,
    l: MergeLimits<'_>,
    line: LineDirection,
) -> Result<bool, Failure> {
    let last = if edge2.orientation() == Orientation::Forward {
        ade2.last_parameter()?
    } else {
        ade2.first_parameter()?
    };
    let current = Vector3::between(line.first_point, ade2.value(last)?);
    if line.direction.cross_square_magnitude(current) > l.lin_tol * l.lin_tol {
        return Ok(false);
    }
    Ok(!(line.direction.angle(current)? > l.ang_tol || diff2.angle(current)? > l.ang_tol))
}

/// `generateSubSeq(...)`: the chain cut between the edges that cannot be merged; for a
/// closed chain the last sub-chain joined to the first one when they can be merged.
fn generate_sub_seq(
    chain: &[Shape],
    closed: bool,
    l: MergeLimits<'_>,
) -> Result<Vec<ShapeUpgradeSubSequenceOfEdges>, Failure> {
    let first = topods::edge(&chain[0])?;
    let mut result = vec![ShapeUpgradeSubSequenceOfEdges {
        seqs_edges: vec![chain[0].clone()],
        ..Default::default()
    }];
    let mut line = get_line_edge_points(&first, LineDirection::default())?;
    for pair in chain.windows(2) {
        let (edge1, edge2) = (topods::edge(&pair[0])?, topods::edge(&pair[1])?);
        if is_merging_possible(&edge1, &edge2, l, line)? {
            if let Some(last) = result.last_mut() {
                last.seqs_edges.push(pair[1].clone());
            }
            continue;
        }
        result.push(ShapeUpgradeSubSequenceOfEdges {
            seqs_edges: vec![pair[1].clone()],
            ..Default::default()
        });
        line = get_line_edge_points(&edge2, line)?;
    }
    // "check first and last chain segments".
    if closed && result.len() > 1 {
        let (last_edge, first_edge) = (topods::edge(chain.last().unwrap_or(&chain[0]))?, first);
        if is_merging_possible(
            &last_edge,
            &first_edge,
            l,
            LineDirection { ok: false, ..line },
        )? {
            let head = result.remove(0);
            if let Some(last) = result.last_mut() {
                last.seqs_edges.extend(head.seqs_edges);
            }
        }
    }
    Ok(result)
}

/// The vertices of the forward and reversed edges with their edges.
fn vertex_edge_map(edges: &[Shape]) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    for edge in edges {
        for vertex in ShapeIterator::new(&edge.oriented(Orientation::Forward), true, true) {
            if !matches!(
                vertex.orientation(),
                Orientation::Forward | Orientation::Reversed
            ) {
                continue;
            }
            if !map.contains(&vertex) {
                map.add(vertex.clone(), List::default());
            }
            map.change_from_key(&vertex)?.append(edge.clone());
        }
    }
    Ok(map)
}

/// The chain of unused edges through `edge`, grown at both ends.
fn grow_chain(
    edge: &Shape,
    map: &IndexedDataMapOfShapeListOfShape,
    used: &mut MapOfShape,
) -> Result<(Vec<Shape>, [Vertex; 2]), Failure> {
    let mut chain = std::collections::VecDeque::from([edge.clone()]);
    let (v0, v1) = topexp::vertices(&topods::edge(edge)?, true);
    let mut ends = [v0, v1];
    for (j, end) in ends.iter_mut().enumerate() {
        while !end.is_null() {
            let Some((next, vertex)) = next_in_chain(end, j, map, used)? else {
                break;
            };
            if j == 0 {
                chain.push_front(next.clone());
            } else {
                chain.push_back(next.clone());
            }
            used.add(next);
            *end = vertex;
        }
    }
    Ok((chain.into(), ends))
}

/// The unused edge with `end` reversed at its opposite end, and its vertex at end `j`.
fn next_in_chain(
    end: &Vertex,
    j: usize,
    map: &IndexedDataMapOfShapeListOfShape,
    used: &MapOfShape,
) -> Result<Option<(Shape, Vertex)>, Failure> {
    for shape in map.find_from_key(end.as_ref())?.iter() {
        if used.contains(shape) {
            continue;
        }
        let (a, b) = topexp::vertices(&topods::edge(shape)?, true);
        let ends = [a, b];
        // "the neighboring edge must have V[j] reversed and located on the opposite end".
        if ends[1 - j].as_ref() == &end.as_ref().reversed() {
            return Ok(Some((shape.clone(), ends[j].clone())));
        }
    }
    Ok(None)
}

impl UnifyState {
    /// `MergeEdges(SeqEdges, theVFmap, SeqOfSubSeqOfEdges, NonMergVrt)`: the chains of
    /// the edges cut into mergeable sub-chains, each with its merged edge when
    /// `MergeSubSeq` succeeds; always true.
    pub(super) fn merge_edges(
        &self,
        edges: &[Shape],
        vf_map: &IndexedDataMapOfShapeListOfShape,
        sub_seqs: &mut Vec<ShapeUpgradeSubSequenceOfEdges>,
        non_merg_vert: &MapOfShape,
    ) -> Result<bool, Failure> {
        let map = vertex_edge_map(edges)?;
        let mut avoid = MapOfShape::default();
        avoid.unite(non_merg_vert);
        let mut used = MapOfShape::default();
        let limits = MergeLimits {
            ang_tol: self.ang_tol,
            lin_tol: self.lin_tol,
            avoid: &avoid,
            vf_map,
        };
        for edge in edges {
            if !used.add(edge.clone()) {
                continue;
            }
            let (chain, ends) = grow_chain(edge, &map, &mut used)?;
            if chain.len() < 2 {
                continue;
            }
            sub_seqs.extend(generate_sub_seq(&chain, ends[0].is_same(&ends[1]), limits)?);
        }
        for sub_seq in sub_seqs.iter_mut() {
            if sub_seq.seqs_edges.len() < 2 {
                continue;
            }
            if let Some(merged) = self.merge_sub_seq(&sub_seq.seqs_edges, vf_map)? {
                sub_seq.union_edges = merged;
            }
        }
        Ok(true)
    }

    /// `MergeSeq(SeqEdges, theVFmap, nonMergVert)`: the merged sub-chains replaced in the
    /// context by their merged edges.
    pub(super) fn merge_seq(
        &self,
        edges: &[Shape],
        vf_map: &IndexedDataMapOfShapeListOfShape,
        non_merg_vert: &MapOfShape,
    ) -> Result<bool, Failure> {
        let mut sub_seqs = Vec::new();
        if !self.merge_edges(edges, vf_map, &mut sub_seqs, non_merg_vert)? {
            return Ok(false);
        }
        for sub_seq in &sub_seqs {
            if sub_seq.union_edges.is_null() {
                continue;
            }
            self.context()?
                .merge(&sub_seq.seqs_edges, sub_seq.union_edges.as_ref())?;
        }
        Ok(true)
    }
}

impl ShapeUpgradeUnifySameDomain {
    /// `MergeEdges(SeqEdges, theVFmap, SeqOfSubSeqOfEdges, NonMergVrt)` (protected);
    /// `unifier.merge_edges(&edges, &vf_map, &mut sub_seqs, &vertices)?`.
    pub fn merge_edges(
        &self,
        edges: &[Shape],
        vf_map: &IndexedDataMapOfShapeListOfShape,
        sub_seqs: &mut Vec<ShapeUpgradeSubSequenceOfEdges>,
        non_merg_vert: &MapOfShape,
    ) -> Result<bool, Failure> {
        self.state
            .read()
            .merge_edges(edges, vf_map, sub_seqs, non_merg_vert)
    }

    /// `MergeSeq(SeqEdges, theVFmap, nonMergVert)` (protected);
    /// `unifier.merge_seq(&edges, &vf_map, &vertices)?`.
    pub fn merge_seq(
        &self,
        edges: &[Shape],
        vf_map: &IndexedDataMapOfShapeListOfShape,
        non_merg_vert: &MapOfShape,
    ) -> Result<bool, Failure> {
        self.state.read().merge_seq(edges, vf_map, non_merg_vert)
    }
}
