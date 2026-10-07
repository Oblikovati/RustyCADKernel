// Rust port of DataExchange/TKXSBase/Interface/Interface_Protocol (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_Protocol`: what a norm knows of its entities (case numbers, types,
//! models), and its resources (the protocols it builds on).
use super::{InterfaceCheck, InterfaceGraph, InterfaceInterfaceModel};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Handle, RuntimeType, RuntimeTyped, StandardTransient, Transient},
};
use std::sync::{OnceLock, PoisonError, RwLock};

/// `Interface_Protocol`, abstract. The pure virtual members are required methods, the
/// virtual ones provided methods with the native body; `Type` is non-virtual.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceProtocol;
/// fn resources(protocol: &dyn InterfaceProtocol) -> i32 { protocol.nb_resources() }
/// ```
pub trait InterfaceProtocol: Transient {
    /// `NbResources`: the count of protocols this one builds on.
    fn nb_resources(&self) -> i32;
    /// `Resource(num)`.
    fn resource(&self, num: i32) -> Handle<dyn InterfaceProtocol>;
    /// `CaseNumber(obj)`: 0 for null, else `TypeNumber(Type(obj))`.
    fn case_number(&self, obj: &Handle<StandardTransient>) -> i32 {
        if obj.is_null() {
            return 0;
        }
        self.type_number(self.entity_type(obj, 1))
    }
    /// `IsDynamicType(obj)`: true by default.
    fn is_dynamic_type(&self, _obj: &Handle<StandardTransient>) -> bool {
        true
    }
    /// `NbTypes(obj)`: 1 by default.
    fn nb_types(&self, _obj: &Handle<StandardTransient>) -> i32 {
        1
    }
    /// `Type(obj, nt = 1)`: the dynamic type (`Standard_Transient` for null).
    fn entity_type(&self, obj: &Handle<StandardTransient>, _nt: i32) -> &'static RuntimeType {
        obj.dynamic_type()
            .unwrap_or_else(StandardTransient::static_type)
    }
    /// `TypeNumber(atype)`: the case number of a type, 0 when not recognised.
    fn type_number(&self, atype: &RuntimeType) -> i32;
    /// `GlobalCheck(G, ach)`: true (nothing to check) by default.
    fn global_check(&self, _graph: &InterfaceGraph, _ach: &mut Handle<InterfaceCheck>) -> bool {
        true
    }
    /// `NewModel`.
    fn new_model(&self) -> Handle<dyn InterfaceInterfaceModel>;
    /// `IsSuitableModel(model)`.
    fn is_suitable_model(&self, model: &Handle<dyn InterfaceInterfaceModel>) -> bool;
    /// `UnknownEntity`.
    fn unknown_entity(&self) -> Handle<StandardTransient>;
    /// `IsUnknownEntity(ent)`.
    fn is_unknown_entity(&self, ent: &Handle<StandardTransient>) -> bool;
}
impl_transient_interface!(
    dyn InterfaceProtocol,
    "Interface_Protocol",
    StandardTransient
);

fn active_lock() -> &'static RwLock<Handle<dyn InterfaceProtocol>> {
    static ACTIVE: OnceLock<RwLock<Handle<dyn InterfaceProtocol>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(Handle::default()))
}

/// The static members of `Interface_Protocol` (the process-wide active protocol).
pub struct InterfaceProtocolBase;

impl InterfaceProtocolBase {
    /// `Active`: the active protocol (null when none).
    pub fn active() -> Handle<dyn InterfaceProtocol> {
        active_lock()
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// `SetActive(aprotocol)`.
    pub fn set_active(aprotocol: &Handle<dyn InterfaceProtocol>) {
        *active_lock()
            .write()
            .unwrap_or_else(PoisonError::into_inner) = aprotocol.clone();
    }
    /// `ClearActive`.
    pub fn clear_active() {
        active_lock()
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .nullify();
    }
}
