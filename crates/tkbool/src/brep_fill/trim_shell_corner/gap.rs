// Rust translation of the static helpers FindFreeVertices, FillGap, FindNextEdge and
// CheckAndOrientEdges (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The chains of section edges that fill the gap between two vertices of a face.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;

/// The vertex/edges ancestor map of a shape.
pub(super) fn vertex_edge_map(shape: &Shape) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut map)?;
    Ok(map)
}

/// `FindFreeVertices(theShape, theVerticesToAvoid, theListOfVertex)`: the vertices of
/// the shape bounding fewer than two edges, except those to avoid.
pub(super) fn find_free_vertices(
    the_shape: &Shape,
    the_vertices_to_avoid: &MapOfShape,
) -> Result<List<Shape>, Failure> {
    let a_map = vertex_edge_map(the_shape)?;
    let mut the_list_of_vertex: List<Shape> = List::default();
    for i in 1..=a_map.extent() {
        let a_key: &Shape = a_map.find_key(i)?;
        if the_vertices_to_avoid.contains(a_key) {
            continue;
        }
        if a_map.find_from_index(i)?.extent() < 2 {
            the_list_of_vertex.append(a_key.clone());
        }
    }
    Ok(the_list_of_vertex)
}

/// The ends of the gap to fill on a face.
pub(super) struct GapEnds<'a> {
    pub(super) first_vertex: &'a Vertex,
    pub(super) last_vertex: &'a Vertex,
    pub(super) first_point: Point2,
    pub(super) last_point: Point2,
}

/// `FillGap(theFirstVertex, theLastVertex, theFirstPoint, theLastPoint, theFace,
/// theSectionEdges, theOrderedList)`: the chain of section edges from the first to the
/// last vertex, oriented on the face; `None` when there is none.
pub(super) fn fill_gap(
    ends: &GapEnds,
    the_face: &Face,
    the_section_edges: &Compound,
) -> Result<Option<List<Shape>>, Failure> {
    let a_map = vertex_edge_map(the_section_edges)?;
    if a_map.extent() == 0
        || !a_map.contains(ends.first_vertex)
        || !a_map.contains(ends.last_vertex)
    {
        return Ok(None);
    }
    let mut a_list_of_edge: List<Shape> = List::default();
    if !find_next_edge(
        ends.first_vertex,
        ends.last_vertex,
        &a_map,
        &MapOfShape::default(),
        &mut a_list_of_edge,
    )? || a_list_of_edge.is_empty()
    {
        return Ok(None);
    }
    let mut the_ordered_list: List<Shape> = List::default();
    if check_and_orient_edges(&a_list_of_edge, ends, the_face, &mut the_ordered_list)? {
        return Ok(Some(the_ordered_list));
    }
    Ok(None)
}

/// `FindNextEdge(theFirstVertex, theLastVertex, theMapVE, theMapToAvoid,
/// theOrderedList)`: a depth-first search of a chain of edges from the first to the
/// last vertex (a failed branch drops every edge collected at its level, as natively).
pub(super) fn find_next_edge(
    the_first_vertex: &Vertex,
    the_last_vertex: &Vertex,
    the_map_ve: &IndexedDataMapOfShapeListOfShape,
    the_map_to_avoid: &MapOfShape,
    the_ordered_list: &mut List<Shape>,
) -> Result<bool, Failure> {
    let mut search = EdgeSearch {
        cur: the_first_vertex.clone(),
        avoid: the_map_to_avoid.clone(),
        edges: List::default(),
    };
    for _ in 1..=the_map_ve.extent() {
        let Some(lste) = the_map_ve.seek(&search.cur) else {
            break;
        };
        let befound = match search.step(lste, the_last_vertex, the_map_ve)? {
            Step::Done => {
                the_ordered_list.append_list(&mut search.edges);
                return Ok(true);
            }
            Step::Found(found) => found,
        };
        if search.cur.is_same(the_last_vertex) {
            break;
        }
        if !befound {
            return Ok(false);
        }
    }
    if search.cur.is_same(the_last_vertex) {
        the_ordered_list.append_list(&mut search.edges);
        return Ok(true);
    }
    Ok(false)
}

/// The state of one level of FindNextEdge.
struct EdgeSearch {
    cur: Vertex,
    avoid: MapOfShape,
    edges: List<Shape>,
}

/// The outcome of one step of FindNextEdge.
enum Step {
    /// The chain reached the last vertex through a recursive search.
    Done,
    /// Whether an edge was taken.
    Found(bool),
}

impl EdgeSearch {
    /// The edges of the current vertex tried in turn: the first one not avoided is
    /// taken, and a recursive search continues from its other vertex.
    fn step(
        &mut self,
        lste: &List<Shape>,
        the_last_vertex: &Vertex,
        the_map_ve: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<Step, Failure> {
        let mut befound: bool = false;
        for an_edge in lste.iter() {
            let a_save_cur_vertex: Vertex = self.cur.clone();
            if self.avoid.contains(an_edge) {
                continue;
            }
            self.advance(an_edge)?;
            befound = true;
            if self.cur.is_same(the_last_vertex) {
                break;
            }
            let mut a_listtmp: List<Shape> = List::default();
            if !find_next_edge(
                &self.cur,
                the_last_vertex,
                the_map_ve,
                &self.avoid,
                &mut a_listtmp,
            )? {
                self.edges.clear();
                self.cur = a_save_cur_vertex;
                continue;
            }
            self.edges.append_list(&mut a_listtmp);
            return Ok(Step::Done);
        }
        Ok(Step::Found(befound))
    }

    /// The edge taken: the current vertex moves to its other end.
    fn advance(&mut self, an_edge: &Shape) -> Result<(), Failure> {
        let (v1, v2) = topexp::vertices(&topods::edge(an_edge)?, false);
        if !self.cur.is_same(&v1) {
            self.cur = v1;
        } else if !self.cur.is_same(&v2) {
            self.cur = v2;
        }
        self.avoid.add(an_edge.clone());
        self.edges.append(an_edge.clone());
        Ok(())
    }
}

/// The parametric tolerance of a vertex on the surface (Precision::Confusion() for a
/// null vertex): the larger of the u and v resolutions.
fn uv_tolerance(a_bas: &BRepAdaptorSurface, v: &Vertex) -> Result<f64, Failure> {
    let tol: f64 = if v.is_null() {
        precision::confusion()
    } else {
        BRepTool::vertex_tolerance(v)?
    };
    let (utol, vtol) = (a_bas.u_resolution(tol)?, a_bas.v_resolution(tol)?);
    Ok(if utol > vtol { utol } else { vtol })
}

/// The p-curve of an edge on the face; Standard_NullObject when there is none (a crash
/// natively).
pub(super) fn pcurve_on(
    e: &Edge,
    face: &Face,
) -> Result<(Handle<dyn Geom2dCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_on_face(e, face)?;
    if found.pcurve.is_null() {
        return Err(Failure::with_type(
            "Standard_NullObject",
            "BRepFill_TrimShellCorner: edge without p-curve on the face; expected a p-curve",
            "BRepFill_TrimShellCorner",
        ));
    }
    Ok((found.pcurve, found.first, found.last))
}

/// The point of a p-curve at a parameter.
pub(super) fn pcurve_value(c: &Handle<dyn Geom2dCurve>, t: f64) -> Result<Point2, Failure> {
    c.get()
        .ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "BRepFill_TrimShellCorner: null p-curve; expected a p-curve",
                "BRepFill_TrimShellCorner",
            )
        })?
        .value(t)
}

/// Which ends of the gap an edge touches: (near the first point, near the last point)
/// for its start and its end.
struct EndHits {
    start_first: bool,
    start_last: bool,
    end_last: bool,
    end_first: bool,
}

/// The ends of the gap touched by the ends of an edge p-curve (the first point is
/// tested before the last one at the start, the last before the first at the end).
fn end_hits(
    e: &Edge,
    the_face: &Face,
    ends: &GapEnds,
    (tol1, tol2): (f64, f64),
    end_first_order: bool,
) -> Result<EndHits, Failure> {
    let (a_curve, f, l) = pcurve_on(e, the_face)?;
    let ap = pcurve_value(&a_curve, f)?;
    let start_first: bool = ap.distance(ends.first_point) < tol1;
    let start_last: bool = !start_first && ap.distance(ends.last_point) < tol1;
    let ap = pcurve_value(&a_curve, l)?;
    let (end_last, end_first) = if end_first_order {
        let first: bool = ap.distance(ends.first_point) < tol2;
        (!first && ap.distance(ends.last_point) < tol2, first)
    } else {
        let last: bool = ap.distance(ends.last_point) < tol2;
        (last, !last && ap.distance(ends.first_point) < tol2)
    };
    Ok(EndHits {
        start_first,
        start_last,
        end_last,
        end_first,
    })
}

/// The orientation of the first edge of the chain: the first end test that succeeds
/// decides (FORWARD from the first point or to the last one).
fn first_edge_orientation(hits: &EndHits) -> Option<Orientation> {
    if hits.start_first {
        return Some(Orientation::Forward);
    }
    if hits.start_last {
        return Some(Orientation::Reversed);
    }
    if hits.end_last {
        return Some(Orientation::Forward);
    }
    hits.end_first.then_some(Orientation::Reversed)
}

/// `CheckAndOrientEdges(theOrderedList, theFirstPoint, theLastPoint, theFace,
/// theOrientedList)`: the chain oriented on the face from its first edge (FORWARD when
/// it starts at the first point or ends at the last one); whether both ends of the gap
/// are reached.
pub(super) fn check_and_orient_edges(
    the_ordered_list: &List<Shape>,
    ends: &GapEnds,
    the_face: &Face,
    the_oriented_list: &mut List<Shape>,
) -> Result<bool, Failure> {
    let mut an_it = the_ordered_list.iter();
    let Some(first) = an_it.next() else {
        return Ok(true);
    };
    let mut a_e_prev: Edge = topods::edge(first)?;
    let a_bas = BRepAdaptorSurface::with_face(the_face, false)?;
    let (vf, vl) = topexp::vertices(&a_e_prev, false);
    let hits = end_hits(
        &a_e_prev,
        the_face,
        ends,
        (uv_tolerance(&a_bas, &vf)?, uv_tolerance(&a_bas, &vl)?),
        false,
    )?;
    if let Some(orientation) = first_edge_orientation(&hits)
        && the_oriented_list.is_empty()
    {
        the_oriented_list.append(a_e_prev.oriented(orientation));
    }
    let (mut b_first_found, mut b_last_found) = (
        hits.start_first || hits.end_first,
        hits.start_last || hits.end_last,
    );
    if let Some(last) = the_oriented_list.last() {
        a_e_prev = topods::edge(last)?;
    }
    for a_shape in an_it {
        let a_e: Edge = topods::edge(a_shape)?;
        let (a_v11, a_v12) = topexp::vertices(&a_e_prev, true);
        let (a_v21, a_v22) = topexp::vertices(&a_e, false);
        let an_ori = if a_v12.is_same(&a_v21) || a_v11.is_same(&a_v22) {
            Orientation::Forward
        } else {
            Orientation::Reversed
        };
        the_oriented_list.append(a_e.oriented(an_ori));
        a_e_prev = topods::edge(&a_e.oriented(an_ori))?;
        let hits = end_hits(
            &a_e,
            the_face,
            ends,
            (uv_tolerance(&a_bas, &a_v21)?, uv_tolerance(&a_bas, &a_v22)?),
            true,
        )?;
        b_first_found |= hits.start_first || hits.end_first;
        b_last_found |= hits.start_last || hits.end_last;
    }
    Ok(b_first_found && b_last_found)
}
