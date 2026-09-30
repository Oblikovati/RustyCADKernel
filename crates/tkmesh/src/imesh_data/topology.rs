//! Interfaces and inherited concrete behavior of model, face, wire, and edge.
use super::*;
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorSurface,
    topods::{self, Edge, Face, Wire},
};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle},
};
use rustycad_tkg3d::top_abs::Orientation;
/// Discrete edge contract; `edge.is_free()`.
pub trait MeshEdge: TessellatedShape {
    /// Edge topology; `edge.get_edge()?`.
    fn get_edge(&self) -> Result<Edge, Failure> {
        topods::edge(&self.get_shape())
    }
    /// Status extension; `edge.status().get_status_mask()`.
    fn status(&self) -> &StatusOwner;
    /// Number of pcurves; `edge.pcurves_nb()`.
    fn pcurves_nb(&self) -> i32;
    /// Add another pcurve, including duplicate face/orientation; `edge.add_pcurve(face, orientation)`.
    fn add_pcurve(&self, face: FacePtr, orientation: Orientation) -> PCurveHandle;
    /// Indexed pcurve; `edge.get_pcurve(0)?`.
    fn get_pcurve(&self, index: i32) -> Result<PCurveHandle, Failure>;
    /// First or last pcurve for face according to orientation; `edge.get_pcurve_for(&face, orientation)?`.
    fn get_pcurve_for(
        &self,
        face: &FacePtr,
        orientation: Orientation,
    ) -> Result<PCurveHandle, Failure>;
    /// Pcurve indices for face; `edge.get_pcurves(&face)?`.
    fn get_pcurves(&self, face: &FacePtr) -> Result<Vec<i32>, Failure>;
    /// Replace 3D discretization; `edge.set_curve(curve)`.
    fn set_curve(&self, curve: CurveHandle);
    /// Shared 3D curve; `edge.get_curve()`.
    fn get_curve(&self) -> CurveHandle;
    /// Angular deflection; `edge.get_angular_deflection()`.
    fn get_angular_deflection(&self) -> f64;
    /// Assign angular deflection; `edge.set_angular_deflection(0.5)`.
    fn set_angular_deflection(&self, value: f64);
    /// Same-parameter flag copied from topology; `edge.get_same_param()`.
    fn get_same_param(&self) -> bool;
    /// Change model flag only; `edge.set_same_param(false)`.
    fn set_same_param(&self, value: bool);
    /// Same-range flag; `edge.get_same_range()`.
    fn get_same_range(&self) -> bool;
    /// Change model flag only; `edge.set_same_range(false)`.
    fn set_same_range(&self, value: bool);
    /// Degenerated flag; `edge.get_degenerated()`.
    fn get_degenerated(&self) -> bool;
    /// Change model flag only; `edge.set_degenerated(true)`.
    fn set_degenerated(&self, value: bool);
    /// No associated pcurves; `edge.is_free()`.
    fn is_free(&self) -> bool {
        self.pcurves_nb() == 0
    }
    /// Clear all curves, retaining endpoints when requested; `edge.clear(true)?`.
    fn clear(&self, keep_endpoints: bool) -> Result<(), Failure> {
        let curve = self.get_curve();
        curve
            .get()
            .ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "null curve; expected attached 3D curve",
                )
            })?
            .clear(keep_endpoints);
        for index in 0..self.pcurves_nb() {
            self.get_pcurve(index)?.get().unwrap().clear(keep_endpoints);
        }
        Ok(())
    }
}
impl_transient_interface!(dyn MeshEdge, "IMeshData_Edge", dyn TessellatedShape);
/// Ordered discrete wire; `wire.edges_nb()`.
pub trait MeshWire: TessellatedShape {
    /// Associated topology; `wire.get_wire()?`.
    fn get_wire(&self) -> Result<Wire, Failure> {
        topods::wire(&self.get_shape())
    }
    /// Status extension; `wire.status()`.
    fn status(&self) -> &StatusOwner;
    /// Number of edges; `wire.edges_nb()`.
    fn edges_nb(&self) -> i32;
    /// Append non-owning edge and return its index; `wire.add_edge(edge, orientation)`.
    fn add_edge(&self, edge: EdgePtr, orientation: Orientation) -> i32;
    /// Edge pointer by index; `wire.get_edge(0)?`.
    fn get_edge(&self, index: i32) -> Result<EdgePtr, Failure>;
    /// Stored orientation; `wire.get_edge_orientation(0)?`.
    fn get_edge_orientation(&self, index: i32) -> Result<Orientation, Failure>;
}
impl_transient_interface!(dyn MeshWire, "IMeshData_Wire", dyn TessellatedShape);
/// Discrete face, whose first wire is outer; `face.wires_nb()`.
pub trait MeshFace: TessellatedShape {
    /// Status extension; `face.status()`.
    fn status(&self) -> &StatusOwner;
    /// Wire count; `face.wires_nb()`.
    fn wires_nb(&self) -> i32;
    /// Append wire with allocation hint; `face.add_wire(&wire, 0)`.
    fn add_wire(&self, wire: &Wire, edge_nb: i32) -> WireHandle;
    /// Indexed wire; `face.get_wire(0)?`.
    fn get_wire(&self, index: i32) -> Result<WireHandle, Failure>;
    /// Unrestricted surface adaptor built at construction; `face.get_surface()`.
    fn get_surface(&self) -> Handle<BRepAdaptorSurface>;
    /// Associated face topology; `face.get_face()?`.
    fn get_face(&self) -> Result<Face, Failure> {
        topods::face(&self.get_shape())
    }
    /// Exact masks 0, 8, and 16 are valid; their combinations are not; `face.is_valid()`.
    fn is_valid(&self) -> bool {
        let status = self.status();
        status.is_equal(Status::NoError)
            || status.is_equal(Status::ReMesh)
            || status.is_equal(Status::UnorientedWire)
    }
}
impl_transient_interface!(dyn MeshFace, "IMeshData_Face", dyn TessellatedShape);
/// Complete discrete shape model; `model.faces_nb()`.
pub trait MeshModel: MeshShape {
    /// Stored bounding-box size; `model.get_max_size()`.
    fn get_max_size(&self) -> f64;
    /// Face count; `model.faces_nb()`.
    fn faces_nb(&self) -> i32;
    /// Append face; `model.add_face(&face)?`.
    fn add_face(&self, face: &Face) -> Result<FaceHandle, Failure>;
    /// Indexed face; `model.get_face(0)?`.
    fn get_face(&self, index: i32) -> Result<FaceHandle, Failure>;
    /// Edge count; `model.edges_nb()`.
    fn edges_nb(&self) -> i32;
    /// Append edge; `model.add_edge(&edge)?`.
    fn add_edge(&self, edge: &Edge) -> Result<EdgeHandle, Failure>;
    /// Indexed edge; `model.get_edge(0)?`.
    fn get_edge(&self, index: i32) -> Result<EdgeHandle, Failure>;
}
impl_transient_interface!(dyn MeshModel, "IMeshData_Model", dyn MeshShape);
