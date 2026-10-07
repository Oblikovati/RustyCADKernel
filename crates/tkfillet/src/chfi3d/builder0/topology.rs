// Rust translation of ChFi3d_cherche_face1, ChFi3d_cherche_element, ChFi3d_cherche_edge,
// ChFi3d_nbface, ChFi3d_edge_common_faces, ChFi3d_AngleEdge, ChFi3d_ChercheBordsLibres,
// ChFi3d_NbNotDegeneratedEdges, ChFi3d_NbSharpEdges, ChFi3d_NumberOfEdges,
// ChFi3d_NumberOfSharpEdges, ChFi3d_cherche_vertex, ChFi3d_Couture,
// ChFi3d_CoutureOnVertex and ChFi3d_IsPseudoSeam (ChFi3d_Builder_0.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The searches in the topology around the vertices the corner builders fill.
use super::faces::conexfaces;
use crate::chfi_ds::Map;
use crate::chfi3d::is_tangent_faces;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;

fn construction(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

/// The sub-shapes of a kind in map order.
fn shapes_of(shape: &Shape, kind: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map)?;
    Ok(map)
}

/// `ChFi3d_cherche_face1(map, F1, F)`: `F` set to the first face of the list other than
/// `F1`; Standard_ConstructionError when `F` is still null (a face given by the caller
/// stays when none is found, as natively); `cherche_face1(&faces, &f1, &mut f)?`.
pub fn cherche_face1(map: &List<Shape>, f1: &Face, f: &mut Face) -> Result<(), Failure> {
    for shape in map.iter() {
        let fcur = topods::face(shape)?;
        if !fcur.is_same(f1) {
            *f = fcur;
            break;
        }
    }
    if f.is_null() {
        return Err(construction("Failed to find face"));
    }
    Ok(())
}

/// The other vertex of an edge of two vertices (map order) when one is `v`.
fn other_vertex(ecur: &Edge, v: &Vertex) -> Result<Option<Vertex>, Failure> {
    let map_v = shapes_of(ecur, ShapeEnum::Vertex)?;
    if map_v.extent() != 2 {
        return Ok(None);
    }
    let (v1, v2) = (
        topods::vertex(map_v.find_key(1)?)?,
        topods::vertex(map_v.find_key(2)?)?,
    );
    if v1.is_same(v) {
        return Ok(Some(v2));
    }
    Ok(v2.is_same(v).then_some(v1))
}

/// The first edge of the face (map order) not excluded that has `v`, written with its
/// other vertex; Standard_ConstructionError with the message when `E` is still null
/// (the caller's edge and vertex stay when none is found, as natively).
fn edge_through(
    (v, f1): (&Vertex, &Face),
    excluded: impl Fn(&Edge) -> bool,
    (e, vtx): (&mut Edge, &mut Vertex),
    message: &str,
) -> Result<(), Failure> {
    let map_e = shapes_of(f1, ShapeEnum::Edge)?;
    for ie in 1..=map_e.extent() {
        let ecur = topods::edge(map_e.find_key(ie)?)?;
        if excluded(&ecur) {
            continue;
        }
        if let Some(other) = other_vertex(&ecur, v)? {
            (*e, *vtx) = (ecur, other);
            break;
        }
    }
    if e.is_null() {
        return Err(construction(message));
    }
    Ok(())
}

/// `ChFi3d_cherche_element(V, E1, F1, E, Vtx)`: `E` and `Vtx` set to the edge of the face
/// other than `E1` through `V` and its other vertex;
/// `cherche_element(&v, &e1, &f1, &mut e, &mut vtx)?`.
pub fn cherche_element(
    v: &Vertex,
    e1: &Edge,
    f1: &Face,
    e: &mut Edge,
    vtx: &mut Vertex,
) -> Result<(), Failure> {
    edge_through(
        (v, f1),
        |x| x.is_same(e1),
        (e, vtx),
        "Failed to find element",
    )
}

/// `ChFi3d_cherche_edge(V, E1, F1, E, Vtx)`: `E` and `Vtx` set to the edge of the face not
/// in the list `E1` through `V` and its other vertex;
/// `cherche_edge(&v, &edges, &f1, &mut e, &mut vtx)?`.
pub fn cherche_edge(
    v: &Vertex,
    e1: &[Shape],
    f1: &Face,
    e: &mut Edge,
    vtx: &mut Vertex,
) -> Result<(), Failure> {
    edge_through(
        (v, f1),
        |x| e1.iter().any(|y| x.is_same(y)),
        (e, vtx),
        "Failed to find edge",
    )
}

/// `ChFi3d_nbface(mapVF)`: the number of distinct faces in the list.
///
/// ```
/// use rustycad_tkernel::ncollection::List;
/// assert_eq!(rustycad_tkfillet::chfi3d::nbface(&List::default()), 0);
/// ```
pub fn nbface(map_vf: &List<rustycad_tkbrep::topods::Shape>) -> i32 {
    let shapes: Vec<&Shape> = map_vf.iter().collect();
    let firsts = shapes
        .iter()
        .enumerate()
        .filter(|(fj, cur)| !shapes[..*fj].iter().any(|x| cur.is_same(x)));
    firsts.count() as i32
}

/// `ChFi3d_edge_common_faces(mapEF, F1, F2)`: the first face of the list and the first
/// other one (`F1` again on a seam); `edge_common_faces(&faces)?`.
pub fn edge_common_faces(map_ef: &List<Shape>) -> Result<(Face, Face), Failure> {
    let first = map_ef.iter().next().ok_or_else(|| {
        Failure::with_message(
            "Standard_NoSuchObject",
            "ChFi3d_edge_common_faces: empty list",
        )
    })?;
    let f1 = topods::face(first)?;
    for shape in map_ef.iter() {
        let f = topods::face(shape)?;
        if !f.is_same(&f1) {
            return Ok((f1, f));
        }
    }
    Ok((f1.clone(), f1))
}

/// `ChFi3d_AngleEdge(Vtx, E1, E2)`: the angle between the edges leaving their common
/// vertex; `angle_edge(&v, &e1, &e2)?`.
pub fn angle_edge(vtx: &Vertex, e1: &Edge, e2: &Edge) -> Result<f64, Failure> {
    let (bcurv1, bcurv2) = (
        BRepAdaptorCurve::with_edge(e1)?,
        BRepAdaptorCurve::with_edge(e2)?,
    );
    let (par_e1, par_e2) = (BRepTool::parameter(vtx, e1)?, BRepTool::parameter(vtx, e2)?);
    let (_, mut dir1) = bcurv1.d1(par_e1)?;
    let (_, mut dir2) = bcurv2.d1(par_e2)?;
    if !vtx.is_same(&topexp::first_vertex(e1, false)) {
        dir1.reverse();
    }
    if !vtx.is_same(&topexp::first_vertex(e2, false)) {
        dir2.reverse();
    }
    Ok(dir1.angle(dir2)?.abs())
}

/// The first edge of the list at the vertex that is not degenerated, not `skip`, and
/// occurs once (a free border).
fn free_border(edges: &List<Shape>, skip: Option<&Edge>) -> Result<Option<Edge>, Failure> {
    for shape in edges.iter() {
        let cur = topods::edge(shape)?;
        if BRepTool::degenerated(&cur)? || skip.is_some_and(|s| cur.is_same(s)) {
            continue;
        }
        if edges.iter().filter(|x| x.is_same(&cur)).count() == 1 {
            return Ok(Some(cur));
        }
    }
    Ok(None)
}

/// `ChFi3d_ChercheBordsLibres(myVEMap, V1, bordlibre, edgelibre1, edgelibre2)`:
/// `(bordlibre, edgelibre1, edgelibre2)`, whether the vertex has two free-border edges;
/// the first one found is returned even without a second (null edges when not found);
/// `cherche_bords_libres(&ve_map, &v)?`.
pub fn cherche_bords_libres(ve_map: &Map, v1: &Vertex) -> Result<(bool, Edge, Edge), Failure> {
    let edges = ve_map.find_from_key(v1)?;
    let Some(edgelibre1) = free_border(edges, None)? else {
        return Ok((false, Edge::default(), Edge::default()));
    };
    let edgelibre2 = free_border(edges, Some(&edgelibre1))?;
    Ok((
        edgelibre2.is_some(),
        edgelibre1,
        edgelibre2.unwrap_or_default(),
    ))
}

/// `ChFi3d_NbNotDegeneratedEdges(Vtx, VEMap)`: the edges at the vertex that are not
/// degenerated (an edge twice on a seam); `nb_not_degenerated_edges(&v, &ve_map)?`.
pub fn nb_not_degenerated_edges(vtx: &Vertex, ve_map: &Map) -> Result<i32, Failure> {
    let mut nba = 0;
    for shape in ve_map.find_from_key(vtx)?.iter() {
        if !BRepTool::degenerated(&topods::edge(shape)?)? {
            nba += 1;
        }
    }
    Ok(nba)
}

/// `ChFi3d_NbSharpEdges(Vtx, VEMap, EFMap)`: the edges at the vertex that are neither
/// degenerated nor between G2-tangent faces.
fn nb_sharp_edges(vtx: &Vertex, ve_map: &Map, ef_map: &Map) -> Result<i32, Failure> {
    let mut nba = 0;
    for shape in ve_map.find_from_key(vtx)?.iter() {
        let cur = topods::edge(shape)?;
        if BRepTool::degenerated(&cur)? {
            continue;
        }
        let (f1, f2) = conexfaces(&cur, ef_map)?;
        if f2.is_null() || !is_tangent_faces(&cur, &f1, &f2, Continuity::G2)? {
            nba += 1;
        }
    }
    Ok(nba)
}

/// The count of edges of a vertex from its count of edge occurrences: each inner edge
/// occurs twice, the two free borders once.
fn edges_of_occurrences(nba: i32, bordlibre: bool) -> i32 {
    if bordlibre {
        (nba - 2) / 2 + 2
    } else {
        nba / 2
    }
}

/// `ChFi3d_NumberOfEdges(Vtx, VEMap)`: the edges arriving at the vertex, degenerated
/// ones aside; `number_of_edges(&v, &ve_map)?`.
pub fn number_of_edges(vtx: &Vertex, ve_map: &Map) -> Result<i32, Failure> {
    let nba = nb_not_degenerated_edges(vtx, ve_map)?;
    Ok(edges_of_occurrences(
        nba,
        cherche_bords_libres(ve_map, vtx)?.0,
    ))
}

/// `ChFi3d_NumberOfSharpEdges(Vtx, VEMap, EFmap)`: the sharp edges arriving at the
/// vertex; `number_of_sharp_edges(&v, &ve_map, &ef_map)?`.
pub fn number_of_sharp_edges(vtx: &Vertex, ve_map: &Map, ef_map: &Map) -> Result<i32, Failure> {
    let nba = nb_sharp_edges(vtx, ve_map, ef_map)?;
    Ok(edges_of_occurrences(
        nba,
        cherche_bords_libres(ve_map, vtx)?.0,
    ))
}

/// `ChFi3d_cherche_vertex(E1, E2, vertex, trouve)`: the first vertex of `E1` (map order)
/// that `E2` has; `cherche_vertex(&e1, &e2)?`.
pub fn cherche_vertex(e1: &Edge, e2: &Edge) -> Result<Option<Vertex>, Failure> {
    let (map_v1, map_v2) = (
        shapes_of(e1, ShapeEnum::Vertex)?,
        shapes_of(e2, ShapeEnum::Vertex)?,
    );
    for i in 1..=map_v1.extent() {
        let vcur1 = topods::vertex(map_v1.find_key(i)?)?;
        for j in 1..=map_v2.extent() {
            if topods::vertex(map_v2.find_key(j)?)?.is_same(&vcur1) {
                return Ok(Some(vcur1));
            }
        }
    }
    Ok(None)
}

/// The first seam edge of the face (map order) accepted by `keep`.
fn seam_of(f: &Face, keep: impl Fn(&Edge) -> bool) -> Result<Option<Edge>, Failure> {
    let map_e1 = shapes_of(f, ShapeEnum::Edge)?;
    let (surf, loc) = BRepTool::surface_with_location(f)?;
    for i in 1..=map_e1.extent() {
        let ecur = topods::edge(map_e1.find_key(i)?)?;
        if BRepTool::is_closed_on_surface(&ecur, &surf, &loc)? && keep(&ecur) {
            return Ok(Some(ecur));
        }
    }
    Ok(None)
}

/// `ChFi3d_Couture(F, couture, edgecouture)`: the seam edge of the face, if any;
/// `couture(&f)?`.
pub fn couture(f: &Face) -> Result<Option<Edge>, Failure> {
    seam_of(f, |_| true)
}

/// `ChFi3d_CoutureOnVertex(F, V, couture, edgecouture)`: the seam edge of the face at
/// the vertex, if any; `couture_on_vertex(&f, &v)?`.
pub fn couture_on_vertex(f: &Face, v: &Vertex) -> Result<Option<Edge>, Failure> {
    seam_of(f, |e| {
        let (vf, vl) = topexp::vertices(e, false);
        vf.is_same(v) || vl.is_same(v)
    })
}

/// `ChFi3d_IsPseudoSeam(E, F)`: whether the closed edge has a neighbour that is really
/// closed on the face; `is_pseudo_seam(&e, &f)?`.
pub fn is_pseudo_seam(e: &Edge, f: &Face) -> Result<bool, Failure> {
    if !BRepTool::is_closed_on_face(e, f)? {
        return Ok(false);
    }
    let (vf, vl) = topexp::vertices(e, false);
    let mut explo = Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explo.more() {
        let ecur = topods::edge(explo.current())?;
        if !ecur.is_same(e) {
            let (v1, v2) = topexp::vertices(&ecur, false);
            let touches = v1.is_same(&vf) || v1.is_same(&vl) || v2.is_same(&vf) || v2.is_same(&vl);
            if touches && brep_tools::is_really_closed(&ecur, f)? {
                return Ok(true);
            }
        }
        explo.next()?;
    }
    Ok(false)
}
