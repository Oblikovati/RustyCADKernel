//! `BOPAlgo_WireEdgeSet`: the edges handed to the wire splitter and the wires it built.
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::{BaseAllocator, List};

/// The face, the start edges and the built wires of one wire splitting.
///
/// ```
/// use rustycad_tkbo::bop_algo::WireEdgeSet;
/// let set = WireEdgeSet::new();
/// assert!(set.start_elements().is_empty() && set.shapes().is_empty());
/// ```
#[derive(Clone, Default)]
pub struct WireEdgeSet {
    face: Face,
    start_shapes: List<Shape>,
    shapes: List<Shape>,
}

impl WireEdgeSet {
    /// An empty set (`BOPAlgo_WireEdgeSet()`); `WireEdgeSet::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// An empty set whose lists use `allocator`; `WireEdgeSet::with_allocator(allocator)`.
    pub fn with_allocator(allocator: BaseAllocator) -> Self {
        Self {
            face: Face::default(),
            start_shapes: List::with_allocator(allocator.clone()),
            shapes: List::with_allocator(allocator),
        }
    }

    /// Forgets the start edges and the built wires, keeping the face; `set.clear()`.
    pub fn clear(&mut self) {
        self.start_shapes.clear();
        self.shapes.clear();
    }

    /// Sets the face the edges lie on; `set.set_face(&face)`.
    pub fn set_face(&mut self, face: &Face) {
        self.face = face.clone();
    }

    /// The face the edges lie on; `set.face()`.
    pub fn face(&self) -> &Face {
        &self.face
    }

    /// Appends one edge to split into wires; `set.add_start_element(&edge)`.
    pub fn add_start_element(&mut self, edge: &Shape) {
        self.start_shapes.append(edge.clone());
    }

    /// The edges to split; `set.start_elements().len()`.
    pub fn start_elements(&self) -> &List<Shape> {
        &self.start_shapes
    }

    /// Appends one built wire; `set.add_shape(&wire)`.
    pub fn add_shape(&mut self, wire: &Shape) {
        self.shapes.append(wire.clone());
    }

    /// The built wires; `set.shapes().len()`.
    pub fn shapes(&self) -> &List<Shape> {
        &self.shapes
    }
}
