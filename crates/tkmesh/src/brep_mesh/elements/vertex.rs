use super::DegreeOfFreedom;
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkmath::gp::Xy;
/// Parametric mesh vertex with external 3D index; `BRepMeshVertex::default()`.
#[derive(Clone, Copy, Debug, Default)]
pub struct BRepMeshVertex {
    uv: Xy,
    location_3d: i32,
    movability: DegreeOfFreedom,
}
impl BRepMeshVertex {
    /// Associate a parametric position with a 3D index; `BRepMeshVertex::new(Xy::new(1.,2.),4,DegreeOfFreedom::Free)`.
    pub fn new(uv: Xy, location_3d: i32, movability: DegreeOfFreedom) -> Self {
        Self {
            uv,
            location_3d,
            movability,
        }
    }
    /// Create an unassociated parametric vertex; `BRepMeshVertex::from_uv(1.,2.,DegreeOfFreedom::Fixed)`.
    pub fn from_uv(u: f64, v: f64, movability: DegreeOfFreedom) -> Self {
        Self::new(Xy::new(u, v), 0, movability)
    }
    /// Replace all native fields; `vertex.initialize(Xy::default(),3,DegreeOfFreedom::Free)`.
    pub fn initialize(&mut self, uv: Xy, location_3d: i32, movability: DegreeOfFreedom) {
        *self = Self::new(uv, location_3d, movability);
    }
    /// Read the parametric coordinates; `vertex.coord()`.
    pub fn coord(&self) -> &Xy {
        &self.uv
    }
    /// Mutate the stored coordinates directly; `vertex.change_coord().set_x(2.)`.
    pub fn change_coord(&mut self) -> &mut Xy {
        &mut self.uv
    }
    /// Read the external 3D point index; `vertex.location_3d()`.
    pub fn location_3d(&self) -> i32 {
        self.location_3d
    }
    /// Read the native constraint flag; `vertex.movability()`.
    pub fn movability(&self) -> DegreeOfFreedom {
        self.movability
    }
    /// Replace the constraint flag; `vertex.set_movability(DegreeOfFreedom::Deleted)`.
    pub fn set_movability(&mut self, movability: DegreeOfFreedom) {
        self.movability = movability;
    }
    /// Compare coordinates at PConfusion, excluding deleted vertices; `vertex.is_equal(&other)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.movability != DegreeOfFreedom::Deleted
            && other.movability != DegreeOfFreedom::Deleted
            && self
                .uv
                .is_equal(&other.uv, rustycad_tkernel::precision::p_confusion())
    }
}
impl PartialEq for BRepMeshVertex {
    fn eq(&self, other: &Self) -> bool {
        self.is_equal(other)
    }
}
impl HashKey for BRepMeshVertex {
    fn occt_hash(&self) -> usize {
        ((1e5 * self.uv.x()).floor() * (1e5 * self.uv.y()).floor()).occt_hash()
    }
}
