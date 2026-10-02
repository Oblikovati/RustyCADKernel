//! ComposeShell wire segments preserve OCCT 3d097a0 handle-sharing and patch intervals.
use crate::{shape_analysis::ShapeAnalysisEdge, shape_extend::ShapeExtendWireData};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::Orientation;
mod patches;
/// Patch intervals in native U-min, U-max, V-min, V-max order; `WirePatchIndex::default()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WirePatchIndex {
    /// First allowed U patch.
    pub u_min: i32,
    /// Last allowed U patch.
    pub u_max: i32,
    /// First allowed V patch.
    pub v_min: i32,
    /// Last allowed V patch.
    pub v_max: i32,
}
impl Default for WirePatchIndex {
    fn default() -> Self {
        Self {
            u_min: -32000,
            u_max: 32000,
            v_min: -32000,
            v_max: 32000,
        }
    }
}
/// A segment shares edge and patch storage on Clone; `ShapeFixWireSegment::new()`.
/// Clear detaches that storage while retaining traversal orientation.
#[derive(Clone)]
pub struct ShapeFixWireSegment {
    wire: Handle<ShapeExtendWireData>,
    vertex: Vertex,
    orientation: Orientation,
    patches: [Handle<HSequence<i32>>; 4],
}
impl Default for ShapeFixWireSegment {
    fn default() -> Self {
        let mut segment = Self {
            wire: Handle::default(),
            vertex: Vertex::default(),
            orientation: Orientation::Forward,
            patches: std::array::from_fn(|_| Handle::default()),
        };
        segment.clear();
        segment
    }
}
impl ShapeFixWireSegment {
    /// Create an empty forward segment with manifold mode off; `ShapeFixWireSegment::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Copy ordered wire edges, resetting patch limits; `ShapeFixWireSegment::from_wire_data(&wire,Orientation::External)?`.
    pub fn from_wire_data(
        wire: &Handle<ShapeExtendWireData>,
        orientation: Orientation,
    ) -> Result<Self, Failure> {
        let mut segment = Self::new();
        segment.load(wire)?;
        segment.orientation = orientation;
        Ok(segment)
    }
    /// Detach all shared storage, retaining orientation; `segment.clear()`.
    pub fn clear(&mut self) {
        self.wire = Handle::new(ShapeExtendWireData::new());
        self.wire.set_manifold_mode(false);
        self.patches = std::array::from_fn(|_| Handle::new(HSequence::default()));
        self.vertex = Vertex::default();
    }
    /// Clear before copying manifold mode and ordered edges; `segment.load(&wire)?`.
    pub fn load(&mut self, wire: &Handle<ShapeExtendWireData>) -> Result<(), Failure> {
        self.clear();
        let source = wire.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeFix_WireSegment::Load: null wire; expected ShapeExtend_WireData handle",
            )
        })?;
        self.wire.set_manifold_mode(source.manifold_mode());
        for index in 1..=source.nb_edges() {
            self.add_edge(index, &source.edge(index)?)?;
        }
        Ok(())
    }
    /// Native `Load(WireData())` aliases its handle member and clears it; `segment.reload()`.
    pub fn reload(&mut self) {
        self.clear();
    }
    /// Borrow the current wire handle; `segment.wire_data()`.
    pub fn wire_data(&self) -> &Handle<ShapeExtendWireData> {
        &self.wire
    }
    /// Set traversal parity independently of edge orientation; `segment.set_orientation(Orientation::Internal)`.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }
    /// Read traversal parity; `segment.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }
    /// First endpoint follows the edge orientation, not segment parity; `segment.first_vertex()?`.
    pub fn first_vertex(&self) -> Result<Vertex, Failure> {
        Ok(ShapeAnalysisEdge::new().first_vertex(&self.wire.edge(1)?))
    }
    /// Last endpoint follows the last edge orientation; `segment.last_vertex()?`.
    pub fn last_vertex(&self) -> Result<Vertex, Failure> {
        Ok(ShapeAnalysisEdge::new().last_vertex(&self.wire.edge(self.wire.nb_edges())?))
    }
    /// Compare endpoint identity, including native null/null equality; `segment.is_closed()?`.
    pub fn is_closed(&self) -> Result<bool, Failure> {
        let first = self.first_vertex()?;
        let last = self.last_vertex()?;
        Ok(first.is_same(&last))
    }
    /// Number of currently stored ordered edges; `segment.nb_edges()`.
    pub fn nb_edges(&self) -> i32 {
        self.wire.nb_edges()
    }
    /// Negative indices reverse the selected edge; `segment.edge(-1)?`.
    pub fn edge(&self, index: i32) -> Result<Edge, Failure> {
        self.wire.edge(index)
    }
    /// Replace geometry while retaining patch indices; `segment.set_edge(1,&edge)?`.
    pub fn set_edge(&self, index: i32, edge: &Edge) -> Result<(), Failure> {
        self.wire.set(edge, index)
    }
    /// Store vertex mode without changing edges or patch limits; `segment.set_vertex(&vertex)`.
    pub fn set_vertex(&mut self, vertex: &Vertex) {
        self.vertex = vertex.clone();
    }
    /// Read the independently stored vertex; `segment.vertex()`.
    pub fn vertex(&self) -> Vertex {
        self.vertex.clone()
    }
    /// Whether a non-null vertex is stored; `segment.is_vertex()`.
    pub fn is_vertex(&self) -> bool {
        !self.vertex.is_null()
    }
}
