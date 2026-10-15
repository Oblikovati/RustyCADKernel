// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_FuseEdges.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_FuseEdges`: the chains of connected edges of a shape on the same
//! support fused into single edges. The class is an older copy of
//! `BRepLib_FuseEdges` (TKTopAlgo): the same text without the B-spline concatenation
//! and the closed B-spline edges, and with the edge range always moved onto the old
//! p-curves; it is the TKTopAlgo port run in that variant.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tktopalgo::brep_lib::{BRepLibFuseEdges, FuseEdgeLists, FuseFaceMap};

/// The fusion of the edges of a shape;
/// `let mut fe = TopOpeBRepToolFuseEdges::new(&shape, false)?;`.
pub struct TopOpeBRepToolFuseEdges {
    fuse: BRepLibFuseEdges,
}

impl TopOpeBRepToolFuseEdges {
    /// `TopOpeBRepTool_FuseEdges(theShape, PerformNow)`: `perform_now` is ignored, as
    /// natively; Standard_NullObject for a null shape;
    /// `TopOpeBRepToolFuseEdges::new(&shape, false)?`.
    pub fn new(the_shape: &Shape, perform_now: bool) -> Result<Self, Failure> {
        let mut fuse = BRepLibFuseEdges::new(the_shape, perform_now)?;
        fuse.set_top_ope_brep_tool_variant();
        Ok(Self { fuse })
    }

    /// `AvoidEdges(theMapEdg)`: the edges not to fuse; `fe.avoid_edges(&map)`.
    pub fn avoid_edges(&mut self, the_map_edg: &IndexedMapOfShape) {
        self.fuse.avoid_edges(the_map_edg);
    }

    /// `Edges(theMapLstEdg)`: the chains of edges to fuse; `fe.edges()?`.
    pub fn edges(&mut self) -> Result<FuseEdgeLists, Failure> {
        self.fuse.edges()
    }

    /// `ResultEdges(theMapEdg)`: the fused edge of each chain; `fe.result_edges()?`.
    pub fn result_edges(&mut self) -> Result<DataMap<i32, Shape>, Failure> {
        self.fuse.result_edges()
    }

    /// `Faces(theMapFac)`: the faces changed by the fusion and their new faces;
    /// `fe.faces()?`.
    pub fn faces(&mut self) -> Result<FuseFaceMap, Failure> {
        self.fuse.faces()
    }

    /// `Shape()`: the shape with the fused edges; `fe.shape()?`.
    pub fn shape(&mut self) -> Result<&Shape, Failure> {
        self.fuse.shape()
    }

    /// `NbVertices()`: the count of the vertices removed by the fusion;
    /// `fe.nb_vertices()?`.
    pub fn nb_vertices(&mut self) -> Result<i32, Failure> {
        self.fuse.nb_vertices()
    }

    /// `Perform()`: the shape rebuilt with the fused edges; `fe.perform()?`.
    pub fn perform(&mut self) -> Result<(), Failure> {
        self.fuse.perform()
    }
}
