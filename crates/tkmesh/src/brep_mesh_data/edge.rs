//! BRepMeshData_Edge: curve ownership and the native first/last pcurve lookup.
use super::{ArenaVec, DiscreteCurve, DiscretePCurve, MeshAllocator, arena_vector, mesh_index};
use crate::imesh_data::{
    shape::{ShapeState, mesh_tessellation_members},
    *,
};
use rustycad_tkbrep::{brep::BRepTool, topods::Edge};
use rustycad_tkernel::{
    impl_transient,
    ncollection::BaseAllocator,
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkg3d::top_abs::Orientation;
struct EdgeFields {
    curve: CurveHandle,
    angular: f64,
    same_param: bool,
    same_range: bool,
    degenerated: bool,
}
/// Native `BRepMeshData_Edge`; `DiscreteEdge::new(&edge, &allocator)?`.
pub struct DiscreteEdge {
    transient: StandardTransient,
    shape: ShapeState,
    status: StatusOwner,
    allocator: MeshAllocator,
    fields: SharedState<EdgeFields>,
    pcurves: SharedState<ArenaVec<PCurveHandle, BaseAllocator>>,
}
impl_transient!(DiscreteEdge, "BRepMeshData_Edge", dyn MeshEdge, transient,
    interfaces: [dyn MeshEdge, dyn TessellatedShape, dyn MeshShape]);
mesh_tessellation_members!(DiscreteEdge);
impl DiscreteEdge {
    /// Read topology flags and create the empty 3D curve; `DiscreteEdge::new(&edge, &allocator)?`.
    pub fn new(edge: &Edge, allocator: &MeshAllocator) -> Result<Self, Failure> {
        Ok(Self {
            transient: StandardTransient::default(),
            shape: ShapeState::new(edge),
            status: StatusOwner::default(),
            allocator: allocator.clone(),
            fields: SharedState::new(EdgeFields {
                curve: Handle::new(DiscreteCurve::new(allocator)).cast(),
                angular: f64::MAX,
                same_param: BRepTool::same_parameter(edge)?,
                same_range: BRepTool::same_range(edge)?,
                degenerated: BRepTool::degenerated(edge)?,
            }),
            pcurves: SharedState::new(arena_vector(allocator)),
        })
    }
}
impl MeshEdge for DiscreteEdge {
    fn status(&self) -> &StatusOwner {
        &self.status
    }
    fn pcurves_nb(&self) -> i32 {
        self.pcurves.read().len() as i32
    }
    fn add_pcurve(&self, face: FacePtr, orientation: Orientation) -> PCurveHandle {
        let pcurve: PCurveHandle =
            Handle::new(DiscretePCurve::new(face, orientation, &self.allocator)).cast();
        self.pcurves.write().push(pcurve.clone());
        pcurve
    }
    fn get_pcurve(&self, index: i32) -> Result<PCurveHandle, Failure> {
        let pcurves = self.pcurves.read();
        Ok(pcurves[mesh_index(index, pcurves.len(), "GetPCurve")?].clone())
    }
    fn get_pcurves(&self, face: &FacePtr) -> Result<Vec<i32>, Failure> {
        let address = face.upgrade().object_address();
        let pcurves = self.pcurves.read();
        let indices: Vec<i32> = pcurves
            .iter()
            .enumerate()
            .filter_map(|(i, curve)| {
                (curve.get().unwrap().get_face().upgrade().object_address() == address)
                    .then_some(i as i32)
            })
            .collect();
        if indices.is_empty() {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                &format!("GetPCurves: face {address:p}; expected registered face"),
            ));
        }
        Ok(indices)
    }
    fn get_pcurve_for(
        &self,
        face: &FacePtr,
        orientation: Orientation,
    ) -> Result<PCurveHandle, Failure> {
        let indices = self.get_pcurves(face)?;
        let first = self.get_pcurve(indices[0])?;
        if first.get().unwrap().get_orientation() == orientation {
            return Ok(first);
        }
        self.get_pcurve(*indices.last().unwrap())
    }
    fn set_curve(&self, curve: CurveHandle) {
        self.fields.write().curve = curve;
    }
    fn get_curve(&self) -> CurveHandle {
        self.fields.read().curve.clone()
    }
    fn get_angular_deflection(&self) -> f64 {
        self.fields.read().angular
    }
    fn set_angular_deflection(&self, value: f64) {
        self.fields.write().angular = value;
    }
    fn get_same_param(&self) -> bool {
        self.fields.read().same_param
    }
    fn set_same_param(&self, value: bool) {
        self.fields.write().same_param = value;
    }
    fn get_same_range(&self) -> bool {
        self.fields.read().same_range
    }
    fn set_same_range(&self, value: bool) {
        self.fields.write().same_range = value;
    }
    fn get_degenerated(&self) -> bool {
        self.fields.read().degenerated
    }
    fn set_degenerated(&self, value: bool) {
        self.fields.write().degenerated = value;
    }
}
