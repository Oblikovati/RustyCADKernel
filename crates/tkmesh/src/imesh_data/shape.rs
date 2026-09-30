//! Native shape and deflection bases for handled discrete models.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{SharedState, StandardTransient, Transient},
};
/// Shape association (`IMeshData_Shape`); `model.get_shape()`.
pub trait MeshShape: Transient {
    /// Associated topology; `model.get_shape()`.
    fn get_shape(&self) -> Shape;
    /// Replace topology; `model.set_shape(&shape)`.
    fn set_shape(&self, shape: &Shape);
}
impl_transient_interface!(dyn MeshShape, "IMeshData_Shape", StandardTransient);
/// Deflection associated with a shaped model; `edge.get_deflection()`.
pub trait TessellatedShape: MeshShape {
    /// Current deflection, initially `RealLast()`; `edge.get_deflection()`.
    fn get_deflection(&self) -> f64;
    /// Assign without clamping; `edge.set_deflection(0.1)`.
    fn set_deflection(&self, value: f64);
}
impl_transient_interface!(
    dyn TessellatedShape,
    "IMeshData_TessellatedShape",
    dyn MeshShape
);
/// Shared implementation of the shape and tessellation bases.
pub(crate) struct ShapeState {
    pub(crate) shape: SharedState<Shape>,
    pub(crate) deflection: SharedState<f64>,
}
impl ShapeState {
    pub(crate) fn new(shape: &Shape) -> Self {
        Self {
            shape: SharedState::new(shape.clone()),
            deflection: SharedState::new(f64::MAX),
        }
    }
}
macro_rules! mesh_shape_members {
    ($name:ty) => {
        impl $crate::imesh_data::MeshShape for $name {
            fn get_shape(&self) -> rustycad_tkbrep::topods::Shape {
                self.shape.shape.snapshot()
            }
            fn set_shape(&self, shape: &rustycad_tkbrep::topods::Shape) {
                *self.shape.shape.write() = shape.clone();
            }
        }
    };
}
macro_rules! mesh_tessellation_members {
    ($name:ty) => {
        $crate::imesh_data::shape::mesh_shape_members!($name);
        impl $crate::imesh_data::TessellatedShape for $name {
            fn get_deflection(&self) -> f64 {
                *self.shape.deflection.read()
            }
            fn set_deflection(&self, value: f64) {
                *self.shape.deflection.write() = value;
            }
        }
    };
}
pub(crate) use {mesh_shape_members, mesh_tessellation_members};
