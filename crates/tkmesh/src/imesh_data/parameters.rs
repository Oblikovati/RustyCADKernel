//! Parameter-list and curve interfaces, plus the zero-based array adaptor.
use super::FacePtr;
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, RuntimeType, RuntimeTyped, StandardTransient, Transient},
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};
/// Parameter sequence contract; `curve.parameters_nb()`.
pub trait ParametersList: Transient {
    /// Read a parameter; `curve.get_parameter(0)?`.
    fn get_parameter(&self, index: i32) -> Result<f64, Failure>;
    /// Write through the native mutable parameter reference; `curve.set_parameter(0, 2.0)?`.
    fn set_parameter(&self, index: i32, value: f64) -> Result<(), Failure>;
    /// Parameter count; `curve.parameters_nb()`.
    fn parameters_nb(&self) -> i32;
    /// Clear all, or retain endpoints; `curve.clear(true)`.
    fn clear(&self, keep_endpoints: bool);
    /// Native protected parameter removal; used by `RemovePoint` implementations.
    #[doc(hidden)]
    fn remove_parameter(&self, index: i32) -> Result<(), Failure>;
}
impl_transient_interface!(
    dyn ParametersList,
    "IMeshData_ParametersList",
    StandardTransient
);
/// Discrete 3D curve; `curve.add_point(Point3::origin(), 0.0)`.
pub trait MeshCurve: ParametersList {
    /// Insert at a zero-based position; `curve.insert_point(0, point, 0.0)?`.
    fn insert_point(&self, position: i32, point: Point3, parameter: f64) -> Result<(), Failure>;
    /// Append a point/parameter pair; `curve.add_point(point, 1.0)`.
    fn add_point(&self, point: Point3, parameter: f64);
    /// Read a point; `curve.get_point(0)?`.
    fn get_point(&self, index: i32) -> Result<Point3, Failure>;
    /// Write through the native mutable point reference; `curve.set_point(0, point)?`.
    fn set_point(&self, index: i32, point: Point3) -> Result<(), Failure>;
    /// Remove the point and corresponding parameter; `curve.remove_point(0)?`.
    fn remove_point(&self, index: i32) -> Result<(), Failure>;
}
impl_transient_interface!(dyn MeshCurve, "IMeshData_Curve", dyn ParametersList);
/// Discrete face pcurve; `pcurve.is_forward()`.
pub trait MeshPCurve: ParametersList {
    /// Insert a point with mesh index zero; `pcurve.insert_point(0, point, 0.0)?`.
    fn insert_point(&self, position: i32, point: Point2, parameter: f64) -> Result<(), Failure>;
    /// Append a point with mesh index zero; `pcurve.add_point(point, 1.0)`.
    fn add_point(&self, point: Point2, parameter: f64);
    /// Read a point; `pcurve.get_point(0)?`.
    fn get_point(&self, index: i32) -> Result<Point2, Failure>;
    /// Write through the native mutable point reference; `pcurve.set_point(0, point)?`.
    fn set_point(&self, index: i32, point: Point2) -> Result<(), Failure>;
    /// Mesh node index; `pcurve.get_index(0)?`.
    fn get_index(&self, index: i32) -> Result<i32, Failure>;
    /// Write through the native mutable index reference; `pcurve.set_index(0, 12)?`.
    fn set_index(&self, index: i32, value: i32) -> Result<(), Failure>;
    /// Remove point, parameter, and mesh index; `pcurve.remove_point(0)?`.
    fn remove_point(&self, index: i32) -> Result<(), Failure>;
    /// Attached face pointer; `pcurve.get_face()`.
    fn get_face(&self) -> FacePtr;
    /// Edge orientation on the face; `pcurve.get_orientation()`.
    fn get_orientation(&self) -> Orientation;
    /// Everything except REVERSED counts as forward; `pcurve.is_forward()`.
    fn is_forward(&self) -> bool {
        self.get_orientation() != Orientation::Reversed
    }
    /// INTERNAL orientation only; `pcurve.is_internal()`.
    fn is_internal(&self) -> bool {
        self.get_orientation() == Orientation::Internal
    }
}
impl_transient_interface!(dyn MeshPCurve, "IMeshData_PCurve", dyn ParametersList);
/// Array facade over any parameter-list handle; `ParametersListArrayAdaptor::new(curve)`.
pub struct ParametersListArrayAdaptor {
    transient: StandardTransient,
    parameters: Handle<dyn ParametersList>,
}
// The native template declares no RTTI of its own (3d097a0).
impl RuntimeTyped for ParametersListArrayAdaptor {
    const TYPE_NAME: &'static str = StandardTransient::TYPE_NAME;
    fn static_type() -> &'static RuntimeType {
        StandardTransient::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        rustycad_tkernel::standard::project_concrete::<Self>(object)
    }
}
impl Transient for ParametersListArrayAdaptor {
    fn dynamic_type(&self) -> &'static RuntimeType {
        StandardTransient::static_type()
    }
    fn project(&self, requested: std::any::TypeId) -> Option<&dyn std::any::Any> {
        if requested == std::any::TypeId::of::<Self>() {
            return Some(self);
        }
        self.transient.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        &self.transient
    }
}
impl ParametersListArrayAdaptor {
    /// Shares parameters; `ParametersListArrayAdaptor::new(curve.cast())`.
    pub fn new(parameters: Handle<dyn ParametersList>) -> Self {
        Self {
            transient: StandardTransient::default(),
            parameters,
        }
    }
    /// Native zero lower bound, even when empty; `adaptor.lower() == 0`.
    pub fn lower(&self) -> i32 {
        0
    }
    /// Last index, minus one when empty; `adaptor.upper()`.
    pub fn upper(&self) -> i32 {
        self.parameters
            .get()
            .expect("null parameter list; expected live handle")
            .parameters_nb()
            - 1
    }
    /// Parameter at index; `adaptor.value(0)?`.
    pub fn value(&self, index: i32) -> Result<f64, Failure> {
        self.parameters
            .get()
            .ok_or_else(|| {
                Failure::with_message(
                    "Standard_NullObject",
                    "null parameters; expected live handle",
                )
            })?
            .get_parameter(index)
    }
}
