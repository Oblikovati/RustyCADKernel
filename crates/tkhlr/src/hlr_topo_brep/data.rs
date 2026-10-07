//! HLRTopoBRep_Data.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{FaceData, VData};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::{MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;

/// The vertices inserted on one edge, in parameter order.
type VertexList = Vec<VData>;

/// HLRTopoBRep_Data: the outlines and isolines of the faces of a shape, the edges
/// split by them, the vertices inserted on the edges, the outline and internal
/// vertices, and the map from new shapes to the original ones.
///
/// The native edge and vertex iterators point into the maps; the port keeps the
/// edge keys of `InitEdge` (the map is not modified while the edges are walked) and
/// an index into the current vertex list, which `InsertBefore` moves so that it
/// stays on the same vertex, as the native list iterator does. The edge order is
/// the bucket order of the map, which follows the TShape addresses natively as in
/// Rust (docs/topology-design.md section 7).
///
/// ```
/// use rustycad_tkhlr::hlr_topo_brep::Data;
/// let mut data = Data::new();
/// data.init_edge();
/// assert!(!data.more_edge());
/// ```
#[derive(Default)]
pub struct Data {
    old_s: DataMap<Shape, Shape, ShapeMapHasher>,
    spl_e: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    data: DataMap<Shape, FaceData, ShapeMapHasher>,
    out_v: MapOfShape,
    int_v: MapOfShape,
    edges_vertices: DataMap<Shape, VertexList, ShapeMapHasher>,
    edge_keys: Vec<Shape>,
    edge_index: usize,
    v_edge: Option<Shape>,
    v_index: usize,
}

fn no_such(member: &str, what: &str) -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        &format!("HLRTopoBRep_Data::{member}: {what} is not bound; expected a key added before"),
        "",
    )
}

impl Data {
    /// HLRTopoBRep_Data(): empty; `Data::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear(): empties every map; `data.clear()`.
    pub fn clear(&mut self) {
        self.old_s.clear(true);
        self.spl_e.clear(true);
        self.data.clear(true);
        self.out_v.clear(true);
        self.int_v.clear(true);
        self.edges_vertices.clear(true);
    }

    /// Clean(): does nothing natively; `data.clean()`.
    pub fn clean(&mut self) {}

    /// EdgeHasSplE(E): whether the edge was split; `data.edge_has_spl_e(&e)`.
    pub fn edge_has_spl_e(&self, e: &Edge) -> bool {
        self.spl_e.seek(e).is_some_and(|l| !l.is_empty())
    }

    fn face_list(&self, f: &Face, pick: fn(&FaceData) -> &List<Shape>) -> Option<&List<Shape>> {
        self.data.seek(f).map(pick)
    }

    /// FaceHasIntL(F); `data.face_has_int_l(&f)`.
    pub fn face_has_int_l(&self, f: &Face) -> bool {
        self.face_list(f, FaceData::face_int_l)
            .is_some_and(|l| !l.is_empty())
    }
    /// FaceHasOutL(F); `data.face_has_out_l(&f)`.
    pub fn face_has_out_l(&self, f: &Face) -> bool {
        self.face_list(f, FaceData::face_out_l)
            .is_some_and(|l| !l.is_empty())
    }
    /// FaceHasIsoL(F); `data.face_has_iso_l(&f)`.
    pub fn face_has_iso_l(&self, f: &Face) -> bool {
        self.face_list(f, FaceData::face_iso_l)
            .is_some_and(|l| !l.is_empty())
    }

    /// IsSplEEdgeEdge(E1, E2): whether `e2` is `e1` or one of its split parts;
    /// `data.is_spl_e_edge_edge(&e1, &e2)`.
    pub fn is_spl_e_edge_edge(&self, e1: &Edge, e2: &Edge) -> bool {
        match self.spl_e.seek(e1).filter(|l| !l.is_empty()) {
            Some(parts) => parts.iter().any(|s| s.is_same(e2)),
            None => e1.is_same(e2),
        }
    }

    // One of IsIntLFaceEdge, IsOutLFaceEdge, IsIsoLFaceEdge on a face list.
    fn is_in_face_list(&self, list: Option<&List<Shape>>, e: &Edge) -> Result<bool, Failure> {
        for s in list.into_iter().flat_map(|l| l.iter()) {
            if self.is_spl_e_edge_edge(&topods::edge(s)?, e) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// IsIntLFaceEdge(F, E); `data.is_int_l_face_edge(&f, &e)?`.
    pub fn is_int_l_face_edge(&self, f: &Face, e: &Edge) -> Result<bool, Failure> {
        self.is_in_face_list(self.face_list(f, FaceData::face_int_l), e)
    }
    /// IsOutLFaceEdge(F, E); `data.is_out_l_face_edge(&f, &e)?`.
    pub fn is_out_l_face_edge(&self, f: &Face, e: &Edge) -> Result<bool, Failure> {
        self.is_in_face_list(self.face_list(f, FaceData::face_out_l), e)
    }
    /// IsIsoLFaceEdge(F, E); `data.is_iso_l_face_edge(&f, &e)?`.
    pub fn is_iso_l_face_edge(&self, f: &Face, e: &Edge) -> Result<bool, Failure> {
        self.is_in_face_list(self.face_list(f, FaceData::face_iso_l), e)
    }

    /// NewSOldS(New): the original shape of a new one, or the shape itself;
    /// `data.new_s_old_s(&s)`.
    pub fn new_s_old_s(&self, new_s: &Shape) -> Shape {
        self.old_s
            .seek(new_s)
            .cloned()
            .unwrap_or_else(|| new_s.clone())
    }

    /// EdgeSplE(E): the split parts; Standard_NoSuchObject for an unknown edge;
    /// `data.edge_spl_e(&e)?`.
    pub fn edge_spl_e(&self, e: &Edge) -> Result<&List<Shape>, Failure> {
        self.spl_e
            .seek(e)
            .ok_or_else(|| no_such("EdgeSplE", "the edge"))
    }

    fn face_data(&self, member: &str, f: &Face) -> Result<&FaceData, Failure> {
        self.data.seek(f).ok_or_else(|| no_such(member, "the face"))
    }

    /// FaceIntL(F); Standard_NoSuchObject for an unknown face; `data.face_int_l(&f)?`.
    pub fn face_int_l(&self, f: &Face) -> Result<&List<Shape>, Failure> {
        Ok(self.face_data("FaceIntL", f)?.face_int_l())
    }
    /// FaceOutL(F); `data.face_out_l(&f)?`.
    pub fn face_out_l(&self, f: &Face) -> Result<&List<Shape>, Failure> {
        Ok(self.face_data("FaceOutL", f)?.face_out_l())
    }
    /// FaceIsoL(F); `data.face_iso_l(&f)?`.
    pub fn face_iso_l(&self, f: &Face) -> Result<&List<Shape>, Failure> {
        Ok(self.face_data("FaceIsoL", f)?.face_iso_l())
    }

    /// IsOutV(V); `data.is_out_v(&v)`.
    pub fn is_out_v(&self, v: &Vertex) -> bool {
        self.out_v.contains(v)
    }
    /// IsIntV(V); `data.is_int_v(&v)`.
    pub fn is_int_v(&self, v: &Vertex) -> bool {
        self.int_v.contains(v)
    }

    /// AddOldS(NewS, OldS): records the original of a new shape once;
    /// `data.add_old_s(&new_s, &old_s)`.
    pub fn add_old_s(&mut self, new_s: &Shape, old_s: &Shape) {
        if !self.old_s.is_bound(new_s) {
            self.old_s.bind(new_s.clone(), old_s.clone());
        }
    }

    /// AddSplE(E): the split parts, created empty; `data.add_spl_e(&e)`.
    pub fn add_spl_e(&mut self, e: &Edge) -> &mut List<Shape> {
        self.spl_e
            .try_emplaced(Shape::from(e.clone()), List::default)
    }

    fn add_face(&mut self, f: &Face) -> &mut FaceData {
        self.data
            .try_emplaced(Shape::from(f.clone()), FaceData::new)
    }

    /// AddIntL(F); `data.add_int_l(&f)`.
    pub fn add_int_l(&mut self, f: &Face) -> &mut List<Shape> {
        self.add_face(f).add_int_l()
    }
    /// AddOutL(F); `data.add_out_l(&f)`.
    pub fn add_out_l(&mut self, f: &Face) -> &mut List<Shape> {
        self.add_face(f).add_out_l()
    }
    /// AddIsoL(F); `data.add_iso_l(&f)`.
    pub fn add_iso_l(&mut self, f: &Face) -> &mut List<Shape> {
        self.add_face(f).add_iso_l()
    }

    /// AddOutV(V); `data.add_out_v(&v)`.
    pub fn add_out_v(&mut self, v: &Vertex) {
        self.out_v.add(Shape::from(v.clone()));
    }
    /// AddIntV(V); `data.add_int_v(&v)`.
    pub fn add_int_v(&mut self, v: &Vertex) {
        self.int_v.add(Shape::from(v.clone()));
    }
}

mod iteration;
