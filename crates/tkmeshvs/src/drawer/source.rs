// SPDX-License-Identifier: GPL-2.0-only
//! Derived Assign dispatch uses the actual owned base and the original caller slot.
use super::MeshVSDrawer;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleObserver, HandleReference, ProjectedHandleReference, RuntimeType,
    RuntimeTyped, Transient, raise_native_failure,
};
/// Extend actual native storage; `source.drawer_base().set_integer(7,42)`.
pub trait MeshVSDrawerSource: Transient {
    /// Extension accessor; `source.drawer_base()` does not control qualified native operations.
    fn drawer_base(&self) -> &MeshVSDrawer;
    /// Native virtual Assign; `source.assign(&other)` preserves borrowed argument counts.
    fn assign(&self, other: &dyn HandleReference<dyn MeshVSDrawerSource>) {
        let actual = raise_native_failure(registered_drawer(self));
        meshvs_drawer_assign(actual, other);
    }
}
impl MeshVSDrawerSource for MeshVSDrawer {
    fn drawer_base(&self) -> &MeshVSDrawer {
        self
    }
}
impl RuntimeTyped for dyn MeshVSDrawerSource {
    const TYPE_NAME: &'static str = "MeshVS_Drawer";
    fn static_type() -> &'static RuntimeType {
        MeshVSDrawer::static_type()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        rustycad_tkernel::standard::project_interface::<Self>(object)
    }
}
impl std::ops::Deref for dyn MeshVSDrawerSource {
    type Target = MeshVSDrawer;
    fn deref(&self) -> &MeshVSDrawer {
        raise_native_failure(registered_drawer(self))
    }
}
impl HandleReference<dyn MeshVSDrawerSource> for Handle<MeshVSDrawer> {
    fn observe(&self) -> HandleObserver<dyn MeshVSDrawerSource> {
        ProjectedHandleReference::<MeshVSDrawer, dyn MeshVSDrawerSource>::new(self).observe()
    }
    fn reference_address(&self) -> *const () {
        std::ptr::from_ref(self).cast()
    }
    fn object_address(&self) -> *const () {
        // Standard_Handle.hxx reads the original pointer, without projecting a dying pointee.
        Handle::object_address(self)
    }
    fn is_null(&self) -> bool {
        // Metadata remains meaningful after a native virtual Delete failure at count zero.
        Handle::is_null(self)
    }
}
/// Execute the qualified base assignment; `meshvs_drawer_assign(&drawer,&other)`.
pub fn meshvs_drawer_assign(
    target: &MeshVSDrawer,
    other: &dyn HandleReference<dyn MeshVSDrawerSource>,
) {
    if other.is_null() {
        return;
    }
    assign_channel(target, other, |base| &base.maps.integers);
    assign_channel(target, other, |base| &base.maps.doubles);
    assign_channel(target, other, |base| &base.maps.booleans);
    assign_channel(target, other, |base| &base.maps.colors);
    assign_channel(target, other, |base| &base.maps.materials);
    assign_channel(target, other, |base| &base.maps.strings);
}
fn assign_channel<T: Clone>(
    target: &MeshVSDrawer,
    other: &dyn HandleReference<dyn MeshVSDrawerSource>,
    select: fn(
        &MeshVSDrawer,
    ) -> &rustycad_tkernel::standard::SharedState<
        rustycad_tkernel::ncollection::DataMap<i32, T>,
    >,
) {
    let observed = other.observe();
    let Some(source) = observed.get() else {
        return;
    };
    let source = raise_native_failure(registered_drawer(source));
    if std::ptr::eq(target, source) {
        return;
    }
    select(target).write().assign(&select(source).read());
}

/// Qualified native storage access must not dispatch the Rust extension accessor.
pub(crate) fn registered_drawer<T: Transient + ?Sized>(
    source: &T,
) -> Result<&MeshVSDrawer, Failure> {
    source
        .project(std::any::TypeId::of::<MeshVSDrawer>())
        .and_then(|actual| actual.downcast_ref::<MeshVSDrawer>())
        .ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::program_error(
                "MeshVS_Drawer base=missing: expected actual registered MeshVS_Drawer subobject",
            )
        })
}
