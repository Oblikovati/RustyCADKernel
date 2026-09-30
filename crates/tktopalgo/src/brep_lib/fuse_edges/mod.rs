// Rust translation of BRepLib_FuseEdges.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib_FuseEdges`: chains of connected edges on the same curve support and the same
//! faces fused into one edge each, and the shape rebuilt with them.
mod pcurve;
mod result;
mod smoothing;
mod support;

use super::errors::null_object;
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// `NCollection_DataMap<int, NCollection_List<TopoDS_Shape>>`: the chains by number.
pub type FuseEdgeLists = DataMap<i32, List<Shape>>;

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape, TopTools_ShapeMapHasher>`: the
/// modified faces.
pub type FuseFaceMap = DataMap<Shape, Shape, ShapeMapHasher>;

/// `BRepLib_FuseEdges` (docs/topalgo-design.md section 7): the chains of edges to fuse
/// (each vertex inside a chain has exactly its two edges, which share their faces and
/// lie on the same line, circle, ellipse, B-spline or Bezier curve), the fused edges and
/// the rebuilt shape, each computed once on first request, as natively.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tktopalgo::brep_lib::BRepLibFuseEdges;
/// assert!(BRepLibFuseEdges::new(&Shape::default(), false).is_err());
/// ```
pub struct BRepLibFuseEdges {
    my_shape: Shape,
    my_shape_done: bool,
    my_edges_done: bool,
    my_result_edges_done: bool,
    my_map_ver_lst_edg: IndexedDataMapOfShapeListOfShape,
    my_map_edg_lst_fac: IndexedDataMapOfShapeListOfShape,
    my_map_lst_edg: FuseEdgeLists,
    my_map_edg: DataMap<i32, Shape>,
    my_map_faces: FuseFaceMap,
    my_nb_connex_edge: i32,
    my_avoid_edg: IndexedMapOfShape,
    my_concat_bspl: bool,
}

impl BRepLibFuseEdges {
    /// `BRepLib_FuseEdges(theShape, PerformNow)`: `perform_now` is ignored, as natively;
    /// Standard_NullObject for a null shape; `BRepLibFuseEdges::new(&shape, false)?`.
    pub fn new(shape: &Shape, _perform_now: bool) -> Result<Self, Failure> {
        if shape.is_null() {
            return Err(null_object("FuseEdges"));
        }
        Ok(Self {
            my_shape: shape.clone(),
            my_shape_done: false,
            my_edges_done: false,
            my_result_edges_done: false,
            my_map_ver_lst_edg: IndexedDataMapOfShapeListOfShape::default(),
            my_map_edg_lst_fac: IndexedDataMapOfShapeListOfShape::default(),
            my_map_lst_edg: FuseEdgeLists::default(),
            my_map_edg: DataMap::default(),
            my_map_faces: FuseFaceMap::default(),
            my_nb_connex_edge: 0,
            my_avoid_edg: IndexedMapOfShape::default(),
            my_concat_bspl: false,
        })
    }

    /// `AvoidEdges(theMapEdg)`: the edges never fused; `fuser.avoid_edges(&map)`.
    pub fn avoid_edges(&mut self, edges: &IndexedMapOfShape) {
        self.my_avoid_edg = edges.clone();
    }

    /// `SetConcatBSpl(theConcatBSpl)`: B-spline edges meeting with G1 continuity are
    /// concatenated; `fuser.set_concat_bspl(true)`.
    pub fn set_concat_bspl(&mut self, concat_bspl: bool) {
        self.my_concat_bspl = concat_bspl;
    }

    /// `Edges(theMapLstEdg)`: the chains of edges to fuse, by number;
    /// `fuser.edges()?`.
    pub fn edges(&mut self) -> Result<FuseEdgeLists, Failure> {
        if !self.my_edges_done {
            self.build_list_edges()?;
        }
        Ok(self.my_map_lst_edg.clone())
    }

    /// `ResultEdges(theMapEdg)`: the fused edge of each chain whose p-curves could be
    /// rebuilt; `fuser.result_edges()?`.
    pub fn result_edges(&mut self) -> Result<DataMap<i32, Shape>, Failure> {
        if !self.my_edges_done {
            self.build_list_edges()?;
        }
        if !self.my_result_edges_done {
            self.build_list_result_edges()?;
        }
        Ok(self.my_map_edg.clone())
    }

    /// `Faces(theMapFac)`: the faces modified by the fusion and their new faces;
    /// `fuser.faces()?`.
    pub fn faces(&mut self) -> Result<FuseFaceMap, Failure> {
        self.result_edges()?;
        if !self.my_shape_done {
            self.perform()?;
        }
        Ok(self.my_map_faces.clone())
    }

    /// `NbVertices()`: the number of vertices removed by the fusion (one fewer than
    /// the edges of each chain); `fuser.nb_vertices()?`.
    pub fn nb_vertices(&mut self) -> Result<i32, Failure> {
        if !self.my_edges_done {
            self.build_list_edges()?;
        }
        Ok(self
            .my_map_lst_edg
            .values()
            .map(|list| list.extent() - 1)
            .sum())
    }

    /// `Shape()`: the shape rebuilt with the fused edges; `fuser.shape()?`.
    pub fn shape(&mut self) -> Result<&Shape, Failure> {
        self.result_edges()?;
        if !self.my_shape_done {
            self.perform()?;
        }
        Ok(&self.my_shape)
    }

    /// `Perform()`: substitutes the fused edges (the other edges of each chain removed)
    /// and records the copied faces. It does not build the chains: called first, it
    /// finds none and leaves the shape unchanged, as natively; `fuser.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        if !self.my_result_edges_done {
            self.build_list_result_edges()?;
        }
        if self.my_map_edg.extent() > 0 {
            self.substitute()?;
        }
        self.my_shape_done = true;
        Ok(())
    }

    /// The substitution of `Perform`.
    fn substitute(&mut self) -> Result<(), Failure> {
        let mut bsub = Substitution::new();
        for (i_lst, edges) in self.my_map_lst_edg.items() {
            let Ok(new_edge) = self.my_map_edg.find(i_lst) else {
                continue;
            };
            let old_edge = edges.first().expect("a chain has edges");
            bsub.substitute(old_edge, std::slice::from_ref(new_edge))?;
            // the other edges of the list will be removed
            for edge in edges.iter().filter(|e| !old_edge.is_same(e)) {
                bsub.substitute(edge, &[])?;
            }
        }
        // perform the effective substitution
        bsub.build(&self.my_shape)?;
        // before copying the resulting shape, map the modified faces into myMapFaces
        for face in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            if bsub.is_copied(&face) {
                let copy = bsub.copy(&face)?.into_iter().next().expect("a copy");
                self.my_map_faces.bind(face, copy);
            }
        }
        if bsub.is_copied(&self.my_shape) {
            self.my_shape = bsub
                .copy(&self.my_shape)?
                .into_iter()
                .next()
                .expect("a copy");
        }
        Ok(())
    }

    /// `BuildListEdges`: the vertex-to-edges and edge-to-faces maps, then the chain of
    /// each FORWARD or REVERSED edge not yet in a chain nor avoided, kept when it has
    /// more than one edge.
    fn build_list_edges(&mut self) -> Result<(), Failure> {
        self.my_map_lst_edg.clear(false);
        self.my_map_ver_lst_edg = IndexedDataMapOfShapeListOfShape::default();
        self.my_map_edg_lst_fac = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_unique_ancestors(
            &self.my_shape,
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
            &mut self.my_map_ver_lst_edg,
            false,
        )?;
        topexp::map_shapes_and_ancestors(
            &self.my_shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut self.my_map_edg_lst_fac,
        )?;
        let mut map_uniq_edg = MapOfShape::default();
        for index in 1..=self.my_map_edg_lst_fac.extent() {
            let edgecur = self.my_map_edg_lst_fac.find_key(index)?.clone();
            let oriented = matches!(
                edgecur.orientation(),
                Orientation::Forward | Orientation::Reversed
            );
            if map_uniq_edg.contains(&edgecur) || !oriented || self.my_avoid_edg.contains(&edgecur)
            {
                continue;
            }
            let chain = self.build_list_connex_edge(&edgecur, &mut map_uniq_edg)?;
            if chain.extent() > 1 {
                self.my_nb_connex_edge += 1;
                self.my_map_lst_edg.bind(self.my_nb_connex_edge, chain);
            }
        }
        self.my_edges_done = true;
        self.my_result_edges_done = false;
        Ok(())
    }

    /// `BuildListConnexEdge`: the chain through `edge`, first forward from its last
    /// vertex, then backward from its first one, stopping at an edge already met or
    /// after an INTERNAL or EXTERNAL one.
    fn build_list_connex_edge(
        &self,
        edge: &Shape,
        map_uniq: &mut MapOfShape,
    ) -> Result<List<Shape>, Failure> {
        let mut chain: List<Shape> = List::default();
        chain.append(edge.clone());
        map_uniq.add(edge.clone());
        self.extend_chain(edge, true, map_uniq, &mut chain)?;
        self.extend_chain(edge, false, map_uniq, &mut chain)?;
        Ok(chain)
    }

    /// One direction of `BuildListConnexEdge`: appended from the last vertex
    /// (`forward`), else prepended from the first one.
    fn extend_chain(
        &self,
        edge: &Shape,
        forward: bool,
        map_uniq: &mut MapOfShape,
        chain: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let end = |e: &Shape| {
            let e = Edge::unchecked(e.clone());
            if forward {
                topexp::last_vertex(&e, true)
            } else {
                topexp::first_vertex(&e, true)
            }
        };
        let mut vertex: Vertex = end(edge);
        let mut edgecur: Shape = edge.clone();
        while let Some(connex) = self.next_connex_edge(&vertex, &edgecur)? {
            if map_uniq.contains(&connex) {
                break;
            }
            if forward {
                chain.append(connex.clone());
            } else {
                chain.prepend(connex.clone());
            }
            edgecur = connex;
            // here take care about internal or external edges. It is non-sense to build
            // the connex list with such edges.
            if matches!(
                edgecur.orientation(),
                Orientation::External | Orientation::Internal
            ) {
                break;
            }
            vertex = end(&edgecur);
            map_uniq.add(edgecur.clone());
        }
        Ok(())
    }

    /// `NextConnexEdge`: the other edge at the vertex when the vertex has exactly two
    /// edges, the other one is not avoided, both have the same one or two faces, and
    /// they lie on the same support.
    fn next_connex_edge(&self, vertex: &Vertex, edge: &Shape) -> Result<Option<Shape>, Failure> {
        let edges = self.my_map_ver_lst_edg.find_from_key(vertex)?;
        if edges.extent() != 2 {
            return Ok(None);
        }
        let mut it = edges.iter();
        let mut connex = it.next().expect("two edges").clone();
        if edge.is_same(&connex) {
            connex = it.next().expect("two edges").clone();
        }
        if self.my_avoid_edg.contains(&connex) || !self.same_faces(edge, &connex)? {
            return Ok(None);
        }
        let same = support::same_support(
            &Edge::unchecked(edge.clone()),
            &Edge::unchecked(connex.clone()),
            self.my_concat_bspl,
        )?;
        Ok(same.then_some(connex))
    }

    /// The 2nd condition of `NextConnexEdge`: the same number (below 3) of faces, each
    /// face of the first edge among those of the second.
    fn same_faces(&self, edge: &Shape, connex: &Shape) -> Result<bool, Failure> {
        let faces1 = self.my_map_edg_lst_fac.find_from_key(edge)?;
        let faces2 = self.my_map_edg_lst_fac.find_from_key(connex)?;
        if faces1.extent() != faces2.extent() || faces1.extent() >= 3 {
            return Ok(false);
        }
        let mut has_connex = true;
        for face1 in faces1.iter() {
            if !has_connex {
                break;
            }
            for face2 in faces2.iter() {
                has_connex = face1.is_same(face2);
                if has_connex {
                    break;
                }
            }
        }
        Ok(has_connex)
    }
}
