// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_FaceBuilder.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_FaceBuilder`: the faces built on the wires and the blocks of edges
//! of a wire edge set, iterated as faces of wires of edges.
mod unclosed;

pub use unclosed::{
    CLOSEDW, GCLOSEDW, ISUNKNOWN, ISVERTEX, ShapeToShapeMap, UNCLOSEDW, fun_analyzemap_von1e,
    fun_detect_vertices_on1_edge,
};

use super::area_builder::TopOpeBRepBuildFaceAreaBuilder;
use super::block_builder::TopOpeBRepBuildBlockBuilder;
use super::loops::{
    TopOpeBRepBuildBlockIterator, TopOpeBRepBuildLoop, TopOpeBRepBuildLoopBase,
    TopOpeBRepBuildLoopSet, TopOpeBRepBuildPlainLoopSet,
};
use super::shape_set::TopOpeBRepBuildShapeSet;
use super::wire_edge_classifier::TopOpeBRepBuildWireEdgeClassifier;
use super::wire_edge_set::TopOpeBRepBuildWireEdgeSet;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};

/// The builder of faces (`TopOpeBRepBuild_FaceBuilder`);
/// `let mut fb = TopOpeBRepBuildFaceBuilder::with_wire_edge_set(&mut wes, &face, false)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildFaceBuilder {
    my_face: Face,
    my_loop_set: TopOpeBRepBuildPlainLoopSet,
    my_block_iterator: TopOpeBRepBuildBlockIterator,
    my_block_builder: TopOpeBRepBuildBlockBuilder,
    my_face_area_builder: TopOpeBRepBuildFaceAreaBuilder,
    /// `myMOSI`: the edge connexities, unused (EdgeConnexity is deactivated).
    #[allow(dead_code)] // native member kept for the deactivated EdgeConnexity
    my_mosi: DataMap<Shape, i32, ShapeMapHasher>,
}

/// Standard_DomainError of InitEdge on a shape loop.
fn domain_error(message: &str) -> Failure {
    Failure::with_type(
        "Standard_DomainError",
        message,
        "TopOpeBRepBuild_FaceBuilder",
    )
}

impl TopOpeBRepBuildFaceBuilder {
    /// `TopOpeBRepBuild_FaceBuilder()`; `TopOpeBRepBuildFaceBuilder::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepBuild_FaceBuilder(ES, F, ForceClass)` (ForceClass natively defaulted
    /// to false); `TopOpeBRepBuildFaceBuilder::with_wire_edge_set(&mut wes, &face, false)?`.
    pub fn with_wire_edge_set(
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        f: &Shape,
        force_class: bool,
    ) -> Result<Self, Failure> {
        let mut fb = Self::new();
        fb.init_face_builder(wes, f, force_class)?;
        Ok(fb)
    }

    /// `InitFaceBuilder(ES, F, ForceClass)`: the loops of the set classified into face
    /// areas by a wire edge classifier on `f`; `fb.init_face_builder(&mut wes, &face, false)?`.
    pub fn init_face_builder(
        &mut self,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        f: &Shape,
        force_class: bool,
    ) -> Result<(), Failure> {
        self.my_face = topods::face(f)?;
        self.make_loops(wes)?;
        let mut wec = TopOpeBRepBuildWireEdgeClassifier::new(f, &self.my_block_builder)?;
        self.my_face_area_builder.init_face_area_builder(
            &mut self.my_loop_set,
            &mut wec,
            force_class,
        )
    }

    /// `MakeLoops(SS)` (private): the blocks of the set, then a shape loop per shape
    /// and a block loop per block.
    fn make_loops(&mut self, ss: &mut dyn TopOpeBRepBuildShapeSet) -> Result<(), Failure> {
        self.my_block_builder.make_block(ss)?;
        let ll = self.my_loop_set.change_list_of_loop();
        ll.clear();
        ss.init_shapes();
        while ss.more_shapes() {
            let shape_loop = Handle::new(TopOpeBRepBuildLoopBase::from_shape(ss.shape()?));
            ll.append(shape_loop.cast::<dyn TopOpeBRepBuildLoop>());
            ss.next_shape();
        }
        self.my_block_builder.init_block();
        while self.my_block_builder.more_block() {
            let bi = self.my_block_builder.block_iterator()?;
            let block_loop = Handle::new(TopOpeBRepBuildLoopBase::from_block(&bi));
            self.my_loop_set
                .change_list_of_loop()
                .append(block_loop.cast::<dyn TopOpeBRepBuildLoop>());
            self.my_block_builder.next_block();
        }
        Ok(())
    }

    /// `Face()`: the face of the builder; `fb.face()`.
    pub fn face(&self) -> &Face {
        &self.my_face
    }

    /// `InitFace()`: at the first area, their count; `fb.init_face()`.
    pub fn init_face(&mut self) -> i32 {
        self.my_face_area_builder.init_area()
    }

    /// `MoreFace()`; `fb.more_face()`.
    pub fn more_face(&self) -> bool {
        self.my_face_area_builder.more_area()
    }

    /// `NextFace()`; `fb.next_face()`.
    pub fn next_face(&mut self) {
        self.my_face_area_builder.next_area();
    }

    /// `InitWire()`: at the first loop of the area, their count; `fb.init_wire()`.
    pub fn init_wire(&mut self) -> i32 {
        self.my_face_area_builder.init_loop()
    }

    /// `MoreWire()`; `fb.more_wire()`.
    pub fn more_wire(&self) -> bool {
        self.my_face_area_builder.more_loop()
    }

    /// `NextWire()`; `fb.next_wire()`.
    pub fn next_wire(&mut self) {
        self.my_face_area_builder.next_loop();
    }

    /// `IsOldWire()`: the current loop is a shape (an old wire); `fb.is_old_wire()?`.
    pub fn is_old_wire(&self) -> Result<bool, Failure> {
        Ok(self.my_face_area_builder.loop_()?.is_shape())
    }

    /// `OldWire()`: the shape of the current loop; `fb.old_wire()?`.
    pub fn old_wire(&self) -> Result<Shape, Failure> {
        Ok(self.my_face_area_builder.loop_()?.shape())
    }

    /// `FindNextValidElement()`: the block iterator moved to a valid element (the
    /// current loop natively read first, raising without one); `fb.find_next_valid_element()?`.
    pub fn find_next_valid_element(&mut self) -> Result<(), Failure> {
        self.my_face_area_builder.loop_()?;
        while self.my_block_iterator.more() {
            if self
                .my_block_builder
                .element_is_valid(self.my_block_iterator.value())
            {
                break;
            }
            self.my_block_iterator.next();
        }
        Ok(())
    }

    /// `InitEdge()`: at the first valid edge of the current block loop, the block
    /// extent; Standard_DomainError on a shape loop; `fb.init_edge()?`.
    pub fn init_edge(&mut self) -> Result<i32, Failure> {
        let l = self.my_face_area_builder.loop_()?.clone();
        if l.is_shape() {
            return Err(domain_error("TopOpeBRepBuild_FaceBuilder:InitEdge"));
        }
        self.my_block_iterator = l.block_iterator();
        self.my_block_iterator.initialize();
        self.find_next_valid_element()?;
        Ok(self.my_block_iterator.extent())
    }

    /// `MoreEdge()`; `fb.more_edge()`.
    pub fn more_edge(&self) -> bool {
        self.my_block_iterator.more()
    }

    /// `NextEdge()`: the next valid edge; `fb.next_edge()?`.
    pub fn next_edge(&mut self) -> Result<(), Failure> {
        self.my_block_iterator.next();
        self.find_next_valid_element()
    }

    /// `Edge()`: the current edge; Standard_Failure "OutOfRange" past the end and
    /// "Edge not Valid" on an invalid one; `fb.edge()?`.
    pub fn edge(&self) -> Result<&Shape, Failure> {
        if !self.my_block_iterator.more() {
            return Err(Failure::new(&format!(
                "OutOfRange: edge index {} past the block end",
                self.my_block_iterator.value()
            )));
        }
        let i = self.my_block_iterator.value();
        if !self.my_block_builder.element_is_valid(i) {
            return Err(Failure::new(&format!(
                "Edge not Valid: element {i}; expected a valid element"
            )));
        }
        self.my_block_builder.element(i)
    }

    /// `EdgeConnexity(E)`: 0, the management being deactivated (a debug build raises);
    /// `fb.edge_connexity(&e)`.
    pub fn edge_connexity(&self, _e: &Shape) -> i32 {
        0
    }

    /// `AddEdgeWire(E, W)`: `e` added to `w`, the count of added edges (1);
    /// `fb.add_edge_wire(&e, &mut w)?`.
    pub fn add_edge_wire(&self, e: &Shape, w: &mut Shape) -> Result<i32, Failure> {
        BRepBuilder::default().add(w, e)?;
        Ok(1)
    }
}
