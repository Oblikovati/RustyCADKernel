// Rust translation of ModelingAlgorithms/TKShHealing/ShapeExtend/ShapeExtend_WireData.hxx/.cxx
// (constructors, Init, Clear, the queries and Wire/WireAPIMake) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeExtend_WireData`: a wire as an ordered list of edges, with its non-manifold edges
//! and its seams (docs/shhealing-design.md section 2).
mod edit;
mod seams;

use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{HSequence, PackedMap, Sequence};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire};

/// The fields of a `ShapeExtend_WireData`.
struct WireDataFields {
    /// `myEdges`: the manifold edges in wire order (and, without the manifold mode, the
    /// non-manifold ones after them).
    edges: Sequence<Shape>,
    /// `myNonmanifoldEdges`: shared with the handle `NonmanifoldEdges` returns.
    nonmanifold: Handle<HSequence<Shape>>,
    /// `mySeams`: the other seam pairs (null natively until `ComputeSeams`).
    seams: Option<Sequence<i32>>,
    /// `mySeamsCache`: the indices of `seams`, for `IsSeam`.
    seams_cache: PackedMap<i32>,
    /// `mySeamF`: the FORWARD edge of the first seam pair; -1 when the seams must be
    /// computed again, 0 when there is no seam.
    seam_f: i32,
    /// `mySeamR`: the REVERSED edge of the first seam pair.
    seam_r: i32,
    /// `myManifoldMode`.
    manifold_mode: bool,
}

impl WireDataFields {
    /// The state `Clear` leaves: no edge, no seam known, the manifold mode. The seam
    /// cache keeps its content, as natively (it is refilled before it is read again).
    fn clear(&mut self) {
        self.edges = Sequence::default();
        self.nonmanifold = Handle::new(HSequence::default());
        self.seam_f = -1;
        self.seam_r = -1;
        self.seams = None;
        self.manifold_mode = true;
    }
}

/// `ShapeExtend_WireData`: the edges of a wire as an ordered list, the non-manifold
/// (INTERNAL and EXTERNAL) edges beside it in the manifold mode, and the seam edges (an
/// edge present twice, FORWARD and REVERSED).
///
/// It is a transient class shared through handles; every member takes `&self` and locks
/// the fields, as docs/geom-design.md section 3 fixes. Edge numbers are 1-based, and a
/// negative number in `Edge` gives the edge reversed.
///
/// ```
/// use rustycad_tkshhealing::shape_extend::ShapeExtendWireData;
/// let data = ShapeExtendWireData::new();
/// assert_eq!(data.nb_edges(), 0);
/// assert!(data.manifold_mode());
/// ```
pub struct ShapeExtendWireData {
    parent: StandardTransient,
    fields: SharedState<WireDataFields>,
}
impl_transient!(
    ShapeExtendWireData,
    "ShapeExtend_WireData",
    StandardTransient,
    parent
);

impl Default for ShapeExtendWireData {
    /// `ShapeExtend_WireData()`: an empty list (`Clear`).
    fn default() -> Self {
        let mut fields = WireDataFields {
            edges: Sequence::default(),
            nonmanifold: Handle::default(),
            seams: None,
            seams_cache: PackedMap::new(1),
            seam_f: -1,
            seam_r: -1,
            manifold_mode: true,
        };
        fields.clear();
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(fields),
        }
    }
}

/// The 1-based position `num` of a sequence; Standard_OutOfRange for a number below 1
/// (natively the sequence raises it).
pub(crate) fn position(num: i32, member: &str) -> Result<usize, Failure> {
    usize::try_from(num)
        .ok()
        .filter(|&n| n >= 1)
        .ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                &format!("{member}: index {num}; expected an index >= 1"),
            )
        })
}

impl ShapeExtendWireData {
    /// `ShapeExtend_WireData()`; `ShapeExtendWireData::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ShapeExtend_WireData(wire, chained, theManifoldMode)`: [`Self::init_wire`] on a
    /// new list; `ShapeExtendWireData::from_wire(&wire, true, true)?`.
    pub fn from_wire(wire: &Wire, chained: bool, manifold_mode: bool) -> Result<Self, Failure> {
        let data = Self::new();
        data.init_wire(wire, chained, manifold_mode)?;
        Ok(data)
    }

    /// `Init(other)`: this list becomes a copy of `other`: its edges, its non-manifold
    /// edges (added through `Add`) and its manifold mode. The list is cleared first, so
    /// `other` being this list gives an empty list; `data.init(&other)?`.
    pub fn init(&self, other: &ShapeExtendWireData) -> Result<(), Failure> {
        self.clear();
        let (edges, nonmanifold, manifold) = other.snapshot();
        for edge in edges.iter().chain(nonmanifold.iter()) {
            self.add_edge(&topods::edge(edge)?, 0)?;
        }
        self.set_manifold_mode(manifold);
        Ok(())
    }

    /// The edges, the non-manifold edges and the manifold mode, read under one lock.
    fn snapshot(&self) -> (Vec<Shape>, Vec<Shape>, bool) {
        let fields = self.fields.read();
        let edges = fields.edges.iter().cloned().collect();
        let nonmanifold = fields.nonmanifold.sequence(|s| s.iter().cloned().collect());
        (edges, nonmanifold, fields.manifold_mode)
    }

    /// `Clear()`: no edge, no seam known, the manifold mode on; `data.clear()`.
    pub fn clear(&self) {
        self.fields.write().clear();
    }

    /// `NbEdges()`: the number of edges; `data.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.fields.read().edges.length()
    }

    /// `NbNonManifoldEdges()`: the number of non-manifold edges kept apart;
    /// `data.nb_non_manifold_edges()`.
    pub fn nb_non_manifold_edges(&self) -> i32 {
        self.fields.read().nonmanifold.sequence(|s| s.length())
    }

    /// `Edge(num)`: the edge of rank `num`, reversed for a negative `num`;
    /// Standard_OutOfRange outside the list; `data.edge(1)?`.
    pub fn edge(&self, num: i32) -> Result<Edge, Failure> {
        if num < 0 {
            let mut edge = self.edge(-num)?;
            edge.reverse();
            return Ok(edge);
        }
        let index = position(num, "ShapeExtend_WireData::Edge")?;
        let fields = self.fields.read();
        topods::edge(fields.edges.value(index)?)
    }

    /// `NonmanifoldEdge(num)`: the non-manifold edge of rank `num`, a null edge for a
    /// negative `num`; Standard_OutOfRange for 0 or past the end;
    /// `data.nonmanifold_edge(1)?`.
    pub fn nonmanifold_edge(&self, num: i32) -> Result<Edge, Failure> {
        if num < 0 {
            return Ok(Edge::default());
        }
        let index = position(num, "ShapeExtend_WireData::NonmanifoldEdge")?;
        let nonmanifold = self.nonmanifold_edges();
        nonmanifold.sequence(|s| topods::edge(s.value(index)?))
    }

    /// `NonmanifoldEdges()`: the handle of the non-manifold edges, shared with the list
    /// until the next `Clear`; `data.nonmanifold_edges()`.
    pub fn nonmanifold_edges(&self) -> Handle<HSequence<Shape>> {
        self.fields.read().nonmanifold.clone()
    }

    /// `ManifoldMode()` read; `data.manifold_mode()`.
    pub fn manifold_mode(&self) -> bool {
        self.fields.read().manifold_mode
    }

    /// `ManifoldMode() = mode` (the member returns a reference natively);
    /// `data.set_manifold_mode(false)`.
    pub fn set_manifold_mode(&self, mode: bool) {
        self.fields.write().manifold_mode = mode;
    }

    /// `Index(edge)`: the rank of the first edge that is the same as `edge` with the same
    /// orientation, or with any orientation when that rank is not a seam; 0 when none;
    /// `data.index(&edge)?`.
    pub fn index(&self, edge: &Edge) -> Result<i32, Failure> {
        for i in 1..=self.nb_edges() {
            let candidate = self.edge(i)?;
            if candidate.is_same(edge)
                && (candidate.orientation() == edge.orientation() || !self.is_seam(i))
            {
                return Ok(i);
            }
        }
        Ok(0)
    }

    /// `Wire()`: a new wire of the edges in order, closed when it is manifold and its end
    /// vertices are the same, then the non-manifold edges in the manifold mode;
    /// `data.wire()?`.
    pub fn wire(&self) -> Result<Wire, Failure> {
        let builder = BRepBuilder::default();
        let mut wire = Wire::default();
        builder.make_wire(&mut wire);
        let mut is_manifold = true;
        for i in 1..=self.nb_edges() {
            let edge = self.edge(i)?;
            is_manifold &= matches!(
                edge.orientation(),
                Orientation::Forward | Orientation::Reversed
            );
            builder.add(&mut wire, &edge)?;
        }
        if is_manifold {
            close_if_ends_meet(&wire)?;
        }
        if self.manifold_mode() {
            for i in 1..=self.nb_non_manifold_edges() {
                let edge = self.nonmanifold_edge(i)?;
                builder.add(&mut wire, &edge)?;
            }
        }
        Ok(wire)
    }

    /// `WireAPIMake()`: the wire `BRepBuilderAPI_MakeWire` builds from the edges (then
    /// the non-manifold edges in the manifold mode); a null wire when it fails;
    /// `data.wire_api_make()?`.
    pub fn wire_api_make(&self) -> Result<Wire, Failure> {
        let mut maker = BRepBuilderAPIMakeWire::new();
        for i in 1..=self.nb_edges() {
            maker.add_edge(&self.edge(i)?)?;
        }
        if self.manifold_mode() {
            for i in 1..=self.nb_non_manifold_edges() {
                maker.add_edge(&self.nonmanifold_edge(i)?)?;
            }
        }
        if maker.is_done() {
            return maker.wire();
        }
        Ok(Wire::default())
    }
}

/// `TopExp::Vertices(W, vf, vl)`, then `W.Closed(true)` when both are the same vertex.
fn close_if_ends_meet(wire: &Wire) -> Result<(), Failure> {
    let (first, last) = topexp::wire_vertices(wire);
    if !first.is_null() && !last.is_null() && first.is_same(&last) {
        wire.set_closed(true)?;
    }
    Ok(())
}

/// The FORWARD and REVERSED vertices of an edge, the last of each found by the iterator.
fn oriented_vertices(edge: &Edge) -> Result<(Vertex, Vertex), Failure> {
    let (mut first, mut last) = (Vertex::default(), Vertex::default());
    let mut it = ShapeIterator::new(edge, true, true);
    while it.more() {
        let vertex = topods::vertex(it.value()?)?;
        match vertex.orientation() {
            Orientation::Forward => first = vertex,
            Orientation::Reversed => last = vertex,
            _ => {}
        }
        it.next();
    }
    Ok((first, last))
}

impl ShapeExtendWireData {
    /// `Init(wire, chained, theManifoldMode)`: the edges of `wire` in its order (reversed
    /// order for a REVERSED wire), its INTERNAL and EXTERNAL edges apart (after the others
    /// without the manifold mode). Returns false when two consecutive edges are not
    /// connected (only checked in the manifold mode); then, unless `chained`, the list is
    /// rebuilt in the order of `BRepTools_WireExplorer`; `data.init_wire(&w, true, true)?`.
    pub fn init_wire(
        &self,
        wire: &Wire,
        chained: bool,
        manifold_mode: bool,
    ) -> Result<bool, Failure> {
        self.clear();
        self.set_manifold_mode(manifold_mode);
        let ok = self.collect_wire_edges(wire, chained, manifold_mode)?;
        if !manifold_mode {
            self.fields.write().move_nonmanifold_to_edges();
        }
        if ok || chained {
            return Ok(ok);
        }
        self.clear();
        let mut explorer = WireExplorer::with_wire(wire)?;
        while explorer.more() {
            let edge = explorer.current().clone().into_shape();
            self.fields.write().edges.append(edge);
            explorer.next()?;
        }
        Ok(ok)
    }

    /// The loop of `Init(wire, ...)` over the edges of the wire; false when the chain
    /// breaks (the loop stops there unless `chained`).
    fn collect_wire_edges(
        &self,
        wire: &Wire,
        chained: bool,
        manifold: bool,
    ) -> Result<bool, Failure> {
        let (mut ok, mut last) = (true, Vertex::default());
        let reversed = wire.orientation() == Orientation::Reversed;
        let mut it = ShapeIterator::new(wire, true, true);
        while it.more() {
            let edge = topods::edge(it.value()?)?;
            it.next();
            if !matches!(
                edge.orientation(),
                Orientation::Forward | Orientation::Reversed
            ) {
                self.fields.read().nonmanifold.append(edge.into_shape());
                continue;
            }
            let (first, end) = oriented_vertices(&edge)?;
            if !last.is_null() && !last.is_same(&first) && manifold {
                ok = false;
                if !chained {
                    break;
                }
            }
            last = end;
            self.fields.write().push_edge(edge.into_shape(), reversed);
        }
        Ok(ok)
    }
}

impl WireDataFields {
    /// `myEdges->Prepend(E)` for a REVERSED wire, `Append(E)` otherwise.
    fn push_edge(&mut self, edge: Shape, prepend: bool) {
        if prepend {
            self.edges.prepend(edge);
        } else {
            self.edges.append(edge);
        }
    }

    /// Without the manifold mode, `Init` moves the non-manifold edges after the others.
    fn move_nonmanifold_to_edges(&mut self) {
        let moved: Vec<Shape> = self.nonmanifold.sequence(|s| s.iter().cloned().collect());
        for edge in moved {
            self.edges.append(edge);
        }
        self.nonmanifold.change_sequence(|s| s.clear(None));
    }
}
