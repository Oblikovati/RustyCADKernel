// Rust translation of the walk along two consecutive sections in
// BRepFill_Generator::Perform (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Generator.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parallel walk along the edges of two consecutive section wires.
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Wire};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;

/// What the first and the last sections are: whether the first wire is closed and
/// whether the end sections are punctual (all their edges degenerated).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SectionEnds {
    pub(super) u_closed: bool,
    pub(super) degen_first: bool,
    pub(super) degen_last: bool,
}

/// Two consecutive sections walked edge by edge (`ex1`, `ex2`), with the lateral
/// edges already built at each vertex (`Map` natively).
pub(super) struct SectionPair {
    pub(super) wire1: Wire,
    pub(super) wire2: Wire,
    pub(super) ex1: WireExplorer,
    pub(super) ex2: WireExplorer,
    /// The first wire is punctual: its explorer does not advance.
    pub(super) w_point1: bool,
    /// The second wire is punctual: its explorer does not advance.
    pub(super) w_point2: bool,
    pub(super) lateral_edges: DataMap<Shape, Shape, ShapeMapHasher>,
}

impl SectionPair {
    /// The pair of wires `wire1`, `wire2`; the punctual flags are computed only for
    /// the first wire of the first pair and the second wire of the last pair, which
    /// also record the ends; `SectionPair::new(&w1, &w2, true, false, &mut ends)?`.
    pub(super) fn new(
        wire1: &Wire,
        wire2: &Wire,
        is_first_pair: bool,
        is_last_pair: bool,
        ends: &mut SectionEnds,
    ) -> Result<Self, Failure> {
        let mut w_point1: bool = false;
        if is_first_pair {
            w_point1 = all_degenerated(wire1)?;
            ends.degen_first = w_point1;
            let (v1, v2) = topexp::wire_vertices(wire1);
            ends.u_closed = v1.is_same(&v2);
        }
        let mut w_point2: bool = false;
        if is_last_pair {
            w_point2 = all_degenerated(wire2)?;
            ends.degen_last = w_point2;
        }
        Ok(Self {
            wire1: wire1.clone(),
            wire2: wire2.clone(),
            ex1: WireExplorer::with_wire(wire1)?,
            ex2: WireExplorer::with_wire(wire2)?,
            w_point1,
            w_point2,
            lateral_edges: DataMap::default(),
        })
    }

    /// The first loop test: both explorers have an edge; `pair.starts()`.
    pub(super) fn starts(&self) -> bool {
        self.ex1.more() && self.ex2.more()
    }

    /// The loop test after a face: both explorers have an edge, or only the one of
    /// the non-punctual wire (the second wire wins when both are punctual, as natively).
    pub(super) fn continues(&self) -> bool {
        let mut tantque: bool = self.ex1.more() && self.ex2.more();
        if self.w_point1 {
            tantque = self.ex2.more();
        }
        if self.w_point2 {
            tantque = self.ex1.more();
        }
        tantque
    }

    /// Advances each explorer of a non-punctual wire (a non-punctual wire must not
    /// contain a punctual edge).
    pub(super) fn advance(&mut self) -> Result<(), Failure> {
        if !self.w_point1 {
            self.ex1.next()?;
        }
        if !self.w_point2 {
            self.ex2.next()?;
        }
        Ok(())
    }

    /// The current edge of the first explorer, as the native `anOrEdge1` reference
    /// sees it after `advance` (the next edge, or a null edge at the end).
    pub(super) fn current_edge1(&self) -> Edge {
        self.ex1.current().clone()
    }
}

/// Whether every edge of the wire is degenerated (true for an empty wire, as natively).
fn all_degenerated(wire: &Wire) -> Result<bool, Failure> {
    let mut explorer = WireExplorer::with_wire(wire)?;
    let mut w_point: bool = true;
    while explorer.more() {
        w_point = w_point && BRepTool::degenerated(explorer.current())?;
        explorer.next()?;
    }
    Ok(w_point)
}
