//! HLRBRep_FaceIterator.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::FaceData;
use crate::hlr_algo::{EdgesBlock, WiresBlock};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;

/// HLRBRep_FaceIterator: the edges of the wires of a face, wire by wire. The native
/// file is compiled with `No_Exception`, so an out-of-range read is undefined
/// there; the port returns Standard_OutOfRange and, for a face without wires,
/// Standard_NullObject.
///
/// ```
/// use rustycad_tkhlr::hlr_brep::{FaceData, FaceIterator};
/// let mut it = FaceIterator::new();
/// let err = it.init_edge(&FaceData::new()).unwrap_err();
/// assert_eq!(err.exception_type(), "Standard_NullObject");
/// ```
#[derive(Default)]
pub struct FaceIterator {
    i_wire: i32,
    nb_wires: i32,
    i_edge: i32,
    nb_edges: i32,
    wires: Handle<WiresBlock>,
    edges: Handle<EdgesBlock>,
}

fn null_block(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("HLRBRep_FaceIterator: no {what}; expected InitEdge on a set face"),
        "",
    )
}

impl FaceIterator {
    /// HLRBRep_FaceIterator(): before InitEdge (the native counters are
    /// uninitialized; 0 here); `FaceIterator::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// InitEdge(fd): the first edge of the first wire; `it.init_edge(&face)?`.
    pub fn init_edge(&mut self, fd: &FaceData) -> Result<(), Failure> {
        self.i_wire = 0;
        self.wires = fd.wires().clone();
        self.nb_wires = self
            .wires
            .get()
            .ok_or_else(|| null_block("wires"))?
            .nb_wires();
        self.i_edge = 0;
        self.nb_edges = 0;
        self.next_edge()
    }

    /// MoreEdge(); `it.more_edge()`.
    pub fn more_edge(&self) -> bool {
        self.i_wire <= self.nb_wires
    }

    /// NextEdge(): the next edge, or the first edge of the next wire;
    /// `it.next_edge()?`.
    pub fn next_edge(&mut self) -> Result<(), Failure> {
        self.i_edge += 1;
        if self.i_edge <= self.nb_edges {
            return Ok(());
        }
        self.i_wire += 1;
        if self.i_wire <= self.nb_wires {
            self.i_edge = 1;
            self.edges = self.wires_block()?.wire(self.i_wire)?;
            self.nb_edges = self.edges_block()?.nb_edges();
        }
        Ok(())
    }

    /// BeginningOfWire(); `it.beginning_of_wire()`.
    pub fn beginning_of_wire(&self) -> bool {
        self.i_edge == 1
    }

    /// EndOfWire(); `it.end_of_wire()`.
    pub fn end_of_wire(&self) -> bool {
        self.i_edge == self.nb_edges
    }

    /// SkipWire(): the first edge of the next wire; `it.skip_wire()?`.
    pub fn skip_wire(&mut self) -> Result<(), Failure> {
        self.i_edge = self.nb_edges;
        self.next_edge()
    }

    fn wires_block(&self) -> Result<&WiresBlock, Failure> {
        self.wires.get().ok_or_else(|| null_block("wires"))
    }

    fn edges_block(&self) -> Result<&EdgesBlock, Failure> {
        self.edges.get().ok_or_else(|| null_block("edge block"))
    }

    /// Wire(): the edge block of the current wire; `it.wire()?`.
    pub fn wire(&self) -> Result<Handle<EdgesBlock>, Failure> {
        self.wires_block()?.wire(self.i_wire)
    }
    /// Edge(): the data index of the current edge; `it.edge()?`.
    pub fn edge(&self) -> Result<i32, Failure> {
        self.edges_block()?.edge(self.i_edge)
    }
    /// Orientation(); `it.orientation()?`.
    pub fn orientation(&self) -> Result<Orientation, Failure> {
        self.edges_block()?.orientation(self.i_edge)
    }
    /// OutLine(); `it.out_line()?`.
    pub fn out_line(&self) -> Result<bool, Failure> {
        self.edges_block()?.out_line(self.i_edge)
    }
    /// Internal(); `it.internal()?`.
    pub fn internal(&self) -> Result<bool, Failure> {
        self.edges_block()?.internal(self.i_edge)
    }
    /// Double(); `it.double()?`.
    pub fn double(&self) -> Result<bool, Failure> {
        self.edges_block()?.double(self.i_edge)
    }
    /// IsoLine(); `it.iso_line()?`.
    pub fn iso_line(&self) -> Result<bool, Failure> {
        self.edges_block()?.iso_line(self.i_edge)
    }
}
