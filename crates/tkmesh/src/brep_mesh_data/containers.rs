//! Default discrete wire, face, and model containers retain insertion order.
use super::{ArenaVec, DiscreteEdge, MeshAllocator, arena_vector, mesh_index};
use crate::imesh_data::{
    shape::{ShapeState, mesh_shape_members, mesh_tessellation_members},
    *,
};
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorSurface,
    topods::{Edge, Face, Shape, Wire},
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::{BaseAllocator, IncrementalAllocatorObject},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::top_abs::Orientation;
/// Native `BRepMeshData_Wire`; `DiscreteWire::new(&wire, 0, &allocator)`.
pub struct DiscreteWire {
    transient: StandardTransient,
    shape: ShapeState,
    status: StatusOwner,
    edges: SharedState<ArenaVec<(EdgePtr, Orientation), BaseAllocator>>,
}
impl_transient!(DiscreteWire, "BRepMeshData_Wire", dyn MeshWire, transient,
    interfaces: [dyn MeshWire, dyn TessellatedShape, dyn MeshShape]);
mesh_tessellation_members!(DiscreteWire);
impl DiscreteWire {
    /// Empty ordered wire; edge count is an allocation hint; `DiscreteWire::new(&wire, 4, &allocator)`.
    pub fn new(wire: &Wire, edge_nb: i32, allocator: &MeshAllocator) -> Self {
        let mut edges = arena_vector(allocator);
        edges.reserve(if edge_nb > 0 { edge_nb as usize } else { 256 });
        Self {
            transient: StandardTransient::default(),
            shape: ShapeState::new(wire),
            status: StatusOwner::default(),
            edges: SharedState::new(edges),
        }
    }
}
impl MeshWire for DiscreteWire {
    fn status(&self) -> &StatusOwner {
        &self.status
    }
    fn edges_nb(&self) -> i32 {
        self.edges.read().len() as i32
    }
    fn add_edge(&self, edge: EdgePtr, orientation: Orientation) -> i32 {
        let mut edges = self.edges.write();
        let index = edges.len() as i32;
        edges.push((edge, orientation));
        index
    }
    fn get_edge(&self, index: i32) -> Result<EdgePtr, Failure> {
        let edges = self.edges.read();
        Ok(edges[mesh_index(index, edges.len(), "GetEdge")?].0.clone())
    }
    fn get_edge_orientation(&self, index: i32) -> Result<Orientation, Failure> {
        let edges = self.edges.read();
        Ok(edges[mesh_index(index, edges.len(), "GetEdgeOrientation")?].1)
    }
}
/// Native `BRepMeshData_Face`; `DiscreteFace::new(&face, &allocator)?`.
pub struct DiscreteFace {
    transient: StandardTransient,
    shape: ShapeState,
    status: StatusOwner,
    allocator: MeshAllocator,
    surface: Handle<BRepAdaptorSurface>,
    wires: SharedState<ArenaVec<WireHandle, BaseAllocator>>,
}
impl_transient!(DiscreteFace, "BRepMeshData_Face", dyn MeshFace, transient,
    interfaces: [dyn MeshFace, dyn TessellatedShape, dyn MeshShape]);
mesh_tessellation_members!(DiscreteFace);
impl DiscreteFace {
    /// Builds the unrestricted surface adaptor once; `DiscreteFace::new(&face, &allocator)?`.
    pub fn new(face: &Face, allocator: &MeshAllocator) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            shape: ShapeState::new(face),
            status: StatusOwner::default(),
            allocator: allocator.clone(),
            surface: Handle::new(BRepAdaptorSurface::with_face(face, false)?),
            wires: SharedState::new(arena_vector(allocator)),
        })
    }
}
impl MeshFace for DiscreteFace {
    fn status(&self) -> &StatusOwner {
        &self.status
    }
    fn wires_nb(&self) -> i32 {
        self.wires.read().len() as i32
    }
    fn add_wire(&self, wire: &Wire, edge_nb: i32) -> WireHandle {
        let wire: WireHandle =
            Handle::new(DiscreteWire::new(wire, edge_nb, &self.allocator)).cast();
        self.wires.write().push(wire.clone());
        wire
    }
    fn get_wire(&self, index: i32) -> Result<WireHandle, Failure> {
        let wires = self.wires.read();
        Ok(wires[mesh_index(index, wires.len(), "GetWire")?].clone())
    }
    fn get_surface(&self) -> Handle<BRepAdaptorSurface> {
        self.surface.clone()
    }
}
/// Native `BRepMeshData_Model`; `DiscreteModel::new(&shape)`.
pub struct DiscreteModel {
    transient: StandardTransient,
    shape: ShapeState,
    max_size: SharedState<f64>,
    allocator: MeshAllocator,
    faces: SharedState<ArenaVec<FaceHandle, BaseAllocator>>,
    edges: SharedState<ArenaVec<EdgeHandle, BaseAllocator>>,
}
impl_transient!(DiscreteModel, "BRepMeshData_Model", dyn MeshModel, transient,
    interfaces: [dyn MeshModel, dyn MeshShape]);
mesh_shape_members!(DiscreteModel);
impl DiscreteModel {
    /// Create the native arena shared by all model entities; `DiscreteModel::new(&shape)`.
    pub fn new(shape: &Shape) -> Self {
        let allocator = Handle::new(IncrementalAllocatorObject::new(MEMORY_BLOCK_SIZE_HUGE));
        Self {
            transient: StandardTransient::default(),
            shape: ShapeState::new(shape),
            max_size: SharedState::new(0.0),
            faces: SharedState::new(arena_vector(&allocator)),
            edges: SharedState::new(arena_vector(&allocator)),
            allocator,
        }
    }
    /// Assign bounding-box size without recomputing it; `model.set_max_size(10.0)`.
    pub fn set_max_size(&self, value: f64) {
        *self.max_size.write() = value;
    }
}
impl MeshModel for DiscreteModel {
    fn get_max_size(&self) -> f64 {
        *self.max_size.read()
    }
    fn faces_nb(&self) -> i32 {
        self.faces.read().len() as i32
    }
    fn add_face(&self, face: &Face) -> Result<FaceHandle, Failure> {
        let face: FaceHandle = Handle::new(DiscreteFace::new(face, &self.allocator)?).cast();
        self.faces.write().push(face.clone());
        Ok(face)
    }
    fn get_face(&self, index: i32) -> Result<FaceHandle, Failure> {
        let faces = self.faces.read();
        Ok(faces[mesh_index(index, faces.len(), "GetFace")?].clone())
    }
    fn edges_nb(&self) -> i32 {
        self.edges.read().len() as i32
    }
    fn add_edge(&self, edge: &Edge) -> Result<EdgeHandle, Failure> {
        let edge: EdgeHandle = Handle::new(DiscreteEdge::new(edge, &self.allocator)?).cast();
        self.edges.write().push(edge.clone());
        Ok(edge)
    }
    fn get_edge(&self, index: i32) -> Result<EdgeHandle, Failure> {
        let edges = self.edges.read();
        Ok(edges[mesh_index(index, edges.len(), "GetEdge")?].clone())
    }
}
