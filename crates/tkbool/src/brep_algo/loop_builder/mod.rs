// Rust translation of ModelingAlgorithms/TKBool/BRepAlgo/BRepAlgo_Loop.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871 (OCCT_DEBUG_ALGO undefined).

//! `BRepAlgo_Loop`: the wires (and faces) built on a face from edges cut at given
//! vertices and constant edges, walking from edge to edge at their shared vertices.
mod cut;
mod statics;
mod ve_map;

pub use statics::VertexSubstitutes;

use super::face_restrictor::BRepAlgoFaceRestrictor;
use super::image::BRepAlgoImage;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use statics::{purge_new_edges, remove_pending_edges, same_pnt2d, select_edge, store_in_mve};

/// The builder of loops on a face (`BRepAlgo_Loop`); the default comparison tolerance
/// is 0.001; `let mut l = BRepAlgoLoop::new(); l.init(&face); l.add_const_edge(&e); l.perform()?;`.
#[derive(Clone)]
pub struct BRepAlgoLoop {
    my_face: Face,
    my_const_edges: List<Shape>,
    my_edges: List<Shape>,
    my_ver_on_edges: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    my_new_wires: List<Shape>,
    my_new_faces: List<Shape>,
    my_cut_edges: DataMap<Shape, List<Shape>, ShapeMapHasher>,
    my_vertices_for_substitute: VertexSubstitutes,
    my_image_vv: BRepAlgoImage,
    my_tol_conf: f64,
}

impl Default for BRepAlgoLoop {
    /// `BRepAlgo_Loop()`: the tolerance 0.001.
    fn default() -> Self {
        Self {
            my_face: Face::default(),
            my_const_edges: List::default(),
            my_edges: List::default(),
            my_ver_on_edges: DataMap::default(),
            my_new_wires: List::default(),
            my_new_faces: List::default(),
            my_cut_edges: DataMap::default(),
            my_vertices_for_substitute: VertexSubstitutes::default(),
            my_image_vv: BRepAlgoImage::default(),
            my_tol_conf: 0.001,
        }
    }
}

impl BRepAlgoLoop {
    /// `BRepAlgo_Loop()`; `BRepAlgoLoop::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init(F)`: no edge, wire, face or cut edge (the substitutes and the vertex
    /// images are kept); `l.init(&face)`.
    pub fn init(&mut self, f: &Face) {
        self.my_const_edges.clear();
        self.my_edges.clear();
        self.my_ver_on_edges.clear(false);
        self.my_new_wires.clear();
        self.my_new_faces.clear();
        self.my_cut_edges.clear(false);
        self.my_face = f.clone();
    }

    /// `AddEdge(E, LV)`: an edge to cut at the vertices `lv`; `l.add_edge(&e, &lv)`.
    pub fn add_edge(&mut self, e: &Edge, lv: &List<Shape>) {
        self.my_edges.append(e.as_ref().clone());
        self.my_ver_on_edges.bind(e.as_ref().clone(), lv.clone());
    }

    /// `AddConstEdge(E)`: an edge kept as it is; `l.add_const_edge(&e)`.
    pub fn add_const_edge(&mut self, e: &Edge) {
        self.my_const_edges.append(e.as_ref().clone());
    }

    /// `AddConstEdges(LE)`; `l.add_const_edges(&le)`.
    pub fn add_const_edges(&mut self, le: &List<Shape>) {
        for e in le.iter() {
            self.my_const_edges.append(e.clone());
        }
    }

    /// `SetImageVV(theImageVV)`: the vertex to vertex image; `l.set_image_vv(&image)`.
    pub fn set_image_vv(&mut self, the_image_vv: &BRepAlgoImage) {
        self.my_image_vv = the_image_vv.clone();
    }

    /// `SetTolConf(theTolConf)`; `l.set_tol_conf(1e-4)`.
    pub fn set_tol_conf(&mut self, the_tol_conf: f64) {
        self.my_tol_conf = the_tol_conf;
    }

    /// `GetTolConf()`; `l.get_tol_conf()`.
    pub fn get_tol_conf(&self) -> f64 {
        self.my_tol_conf
    }

    /// The cut edges of the edges with vertices.
    fn cut_edges(&mut self) -> Result<(), Failure> {
        let edges: Vec<Shape> = self.my_edges.iter().cloned().collect();
        for an_edge in &edges {
            let Some(vertices) = self.my_ver_on_edges.seek(an_edge).cloned() else {
                continue;
            };
            let mut lce = List::default();
            self.cut_edge(&topods::edge(an_edge)?, &vertices, &mut lce)?;
            self.my_cut_edges.bind(an_edge.clone(), lce);
        }
        Ok(())
    }

    /// The map vertex => edges of the cut edges (once each) and the constant edges
    /// (once each, seams doubled by StoreInMVE).
    fn vertex_edges_map(&mut self) -> Result<IndexedDataMapOfShapeListOfShape, Failure> {
        let mut mve = IndexedDataMapOfShapeListOfShape::default();
        let mut ya_couture = false;
        let mut emap = MapOfShape::default();
        let edges: Vec<Shape> = self.my_edges.iter().cloned().collect();
        for e in &edges {
            let Some(lce) = self.my_cut_edges.seek(e).cloned() else {
                continue;
            };
            for mut ce in lce.iter().cloned() {
                if emap.add(ce.clone()) {
                    store_in_mve(
                        &self.my_face,
                        &mut ce,
                        &mut mve,
                        &mut ya_couture,
                        &mut self.my_vertices_for_substitute,
                        self.my_tol_conf,
                    )?;
                }
            }
        }
        let mut deja_vu = MapOfShape::default();
        let consts: Vec<Shape> = self.my_const_edges.iter().cloned().collect();
        for mut e in consts {
            if deja_vu.add(e.clone()) {
                store_in_mve(
                    &self.my_face,
                    &mut e,
                    &mut mve,
                    &mut ya_couture,
                    &mut self.my_vertices_for_substitute,
                    self.my_tol_conf,
                )?;
            }
        }
        Ok(mve)
    }

    /// The wire walked from the first edge of the map: its start vertex, first and last
    /// edges, and end vertex; `None` when the start vertex is not in the map.
    fn walk_wire(
        &self,
        mve: &mut IndexedDataMapOfShapeListOfShape,
        nw: &mut Shape,
        used_edges: &mut MapOfShape,
    ) -> Result<Option<(Vertex, Edge, Edge, Vertex)>, Failure> {
        let first = mve.find_from_index(1)?.first().cloned().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepAlgo_Loop::Perform: a vertex without edge; expected one",
                "BRepAlgo_Loop",
            )
        })?;
        let mut ce = topods::edge(&first)?;
        let ef = ce.clone();
        let (v1, v2) = topexp::vertices(&ce, false);
        let vf = if ce.as_ref().orientation() == Orientation::Forward {
            v1
        } else {
            v2
        };
        let mut cv = vf.clone();
        if !mve.contains(cv.as_ref()) {
            return Ok(None);
        }
        remove_first_equal(mve.change_from_key(cv.as_ref())?, ce.as_ref());
        loop {
            let (v1, v2) = topexp::vertices(&ce, false);
            cv = if !cv.as_ref().is_same(v1.as_ref()) {
                v1
            } else {
                v2
            };
            BRepBuilder::default().add(nw, ce.as_ref())?;
            used_edges.add(ce.as_ref().clone());
            if !mve.contains(cv.as_ref()) || mve.find_from_key(cv.as_ref())?.is_empty() {
                break;
            }
            let Some(ne) = select_edge(&self.my_face, &ce, &cv, mve.change_from_key(cv.as_ref())?)?
            else {
                break;
            };
            ce = ne;
            if mve.find_from_key(cv.as_ref())?.is_empty() {
                mve.remove_key(cv.as_ref());
            }
        }
        Ok(Some((vf, ef, ce, cv)))
    }

    /// The closed wire kept when its ends meet in UV, retried with the vertex tolerance
    /// raised to the comparison tolerance.
    fn keep_closed_wire(
        &mut self,
        nw: Shape,
        vf: &Vertex,
        ef: &Edge,
        ce: &Edge,
        cv: &Vertex,
    ) -> Result<(), Failure> {
        if !vf.as_ref().is_same(cv.as_ref()) {
            return Ok(());
        }
        let mut closes = same_pnt2d(vf, ef, ce, &self.my_face)?;
        if !closes && BRepTool::vertex_tolerance(vf)? < self.my_tol_conf {
            BRepBuilder::default().update_vertex_tolerance(vf, self.my_tol_conf)?;
            closes = same_pnt2d(vf, ef, ce, &self.my_face)?;
        }
        if closes {
            nw.set_closed(true)?;
            self.my_new_wires.append(nw);
        }
        Ok(())
    }

    /// `Perform()`: the edges cut, the vertex => edges map (vertices merged, images
    /// applied), then the wires walked until the map is empty (hanging edges removed
    /// first); the unused cut edges are purged; `l.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.cut_edges()?;
        let mut mve = self.vertex_edges_map()?;
        self.update_ve_map(&mut mve)?;
        let mut used_edges = MapOfShape::default();
        while !mve.is_empty() {
            let mut wire = Wire::default();
            BRepBuilder::default().make_wire(&mut wire);
            let mut nw = wire.into_shape();
            remove_pending_edges(&mut mve)?;
            if mve.is_empty() {
                break;
            }
            // Natively a start vertex missing from the map loops again on the same map.
            let Some((vf, ef, ce, cv)) = self.walk_wire(&mut mve, &mut nw, &mut used_edges)? else {
                continue;
            };
            self.keep_closed_wire(nw, &vf, &ef, &ce, &cv)?;
        }
        purge_new_edges(&mut self.my_cut_edges, &used_edges);
        Ok(())
    }

    /// `NewWires()`: the closed wires built; `l.new_wires()`.
    pub fn new_wires(&self) -> &List<Shape> {
        &self.my_new_wires
    }

    /// `WiresToFaces()`: the faces of the new wires by a face restrictor on the FORWARD
    /// face, with the orientation of the face; `l.wires_to_faces()?`.
    pub fn wires_to_faces(&mut self) -> Result<(), Failure> {
        if self.my_new_wires.is_empty() {
            return Ok(());
        }
        let mut fr = BRepAlgoFaceRestrictor::new();
        fr.init(
            &topods::face(&self.my_face.as_ref().oriented(Orientation::Forward))?,
            false,
            false,
        );
        for w in self.my_new_wires.iter() {
            fr.add(&topods::wire(w)?);
        }
        fr.perform()?;
        if !fr.is_done() {
            return Ok(());
        }
        let ori_f = self.my_face.as_ref().orientation();
        while fr.more() {
            self.my_new_faces
                .append(fr.current()?.as_ref().oriented(ori_f));
            fr.next()?;
        }
        Ok(())
    }

    /// `NewFaces()`: the faces of WiresToFaces; `l.new_faces()`.
    pub fn new_faces(&self) -> &List<Shape> {
        &self.my_new_faces
    }

    /// `NewEdges(E)`: the cut edges of `e`; Standard_NoSuchObject for an edge not cut;
    /// `l.new_edges(&e)?`.
    pub fn new_edges(&self, e: &Edge) -> Result<&List<Shape>, Failure> {
        self.my_cut_edges.seek(e.as_ref()).ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "BRepAlgo_Loop::NewEdges: the edge was not cut; expected an edge added by AddEdge",
                "BRepAlgo_Loop",
            )
        })
    }

    /// `GetVerticesForSubstitute(VerVerMap)`: a copy of the substitutes;
    /// `l.get_vertices_for_substitute(&mut map)`.
    pub fn get_vertices_for_substitute(&self, ver_ver_map: &mut VertexSubstitutes) {
        *ver_ver_map = self.my_vertices_for_substitute.clone();
    }

    /// `VerticesForSubstitute(VerVerMap)`: the substitutes set; `l.vertices_for_substitute(&map)`.
    pub fn vertices_for_substitute(&mut self, ver_ver_map: &VertexSubstitutes) {
        self.my_vertices_for_substitute = ver_ver_map.clone();
    }
}

/// The first shape equal (IsEqual) to `ce` removed from `l`.
fn remove_first_equal(l: &mut List<Shape>, ce: &Shape) {
    let mut removed = false;
    l.retain(|s| {
        let drop = !removed && s.is_equal(ce);
        removed |= drop;
        !drop
    });
}
