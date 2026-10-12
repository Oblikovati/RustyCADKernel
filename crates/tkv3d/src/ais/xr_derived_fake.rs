//! Named application subclass keeps native inherited RTTI while overriding the real virtual interfaces.
use super::xr_recording_group::XrRecordingEvents;
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{
    Failure, Handle, InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, SharedState,
    StandardTransient, Transient, project_concrete,
};
use rustycad_tkservice::{aspect::AspectXRTrackedDeviceRole, graphic3d::*};
use std::{
    any::{Any, TypeId},
    sync::{Arc, OnceLock},
};
pub(super) struct XrDerivedFake {
    core: AisXrTrackedDevice,
    pub events: Arc<XrRecordingEvents>,
    pub hiding_calls: SharedState<i32>,
    pub fail_after_compute: SharedState<bool>,
}
impl XrDerivedFake {
    pub fn new(events: Arc<XrRecordingEvents>) -> Handle<Self> {
        let mesh = super::xr_state_fixture::xr_state_model(2);
        let object = Handle::new(Self {
            core: super::xr_tracked_device_object::unbound_xr_tracked_device_model(
                &mesh,
                &Handle::default(),
            )
            .unwrap(),
            events,
            hiding_calls: SharedState::new(0),
            fail_after_compute: SharedState::new(false),
        });
        prs_mgr_presentable_bind(&object.clone().cast());
        object
    }
    pub fn role(&self) -> AspectXRTrackedDeviceRole {
        // A native nonvirtual getter can be hidden by a subclass but never overrides base binding.
        AspectXRTrackedDeviceRole::Head
    }
    pub fn hidden_laser_ray(&self) {
        // Native hiding is deliberately distinct from the nonvirtual qualified base call.
        *self.hiding_calls.write() += 1;
    }
}
impl RuntimeTyped for XrDerivedFake {
    const TYPE_NAME: &'static str = "Martensite_XrDerivedFake";
    fn static_type() -> &'static RuntimeType {
        static CASTERS: OnceLock<Handle<RuntimeType>> = OnceLock::new();
        CASTERS
            .get_or_init(register_xr_derived_interfaces)
            .get()
            .unwrap()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        // Caster metadata projects the actual wrapper, not its composed core, retaining overrides.
        project_concrete::<Self>(object)
    }
}
fn register_xr_derived_interfaces() -> Handle<RuntimeType> {
    RuntimeType::register_owned_with_interfaces::<XrDerivedFake>(
        XrDerivedFake::TYPE_NAME,
        std::mem::size_of::<XrDerivedFake>(),
        Some(AisXrTrackedDevice::static_type_handle()),
        xr_derived_interfaces(),
    )
}
fn xr_derived_interfaces() -> Vec<InterfaceEntry> {
    vec![
        InterfaceEntry::new(InterfaceCaster::<dyn AisXrTrackedDeviceSource>::new(
            |value| Some(project_concrete::<XrDerivedFake>(value)?),
        )),
        InterfaceEntry::new(InterfaceCaster::<dyn AisInteractiveObject>::new(|value| {
            Some(project_concrete::<XrDerivedFake>(value)?)
        })),
        InterfaceEntry::new(InterfaceCaster::<dyn SelectMgrSelectableObject>::new(
            |value| Some(project_concrete::<XrDerivedFake>(value)?),
        )),
        InterfaceEntry::new(InterfaceCaster::<dyn PrsMgrPresentableObject>::new(
            |value| Some(project_concrete::<XrDerivedFake>(value)?),
        )),
    ]
}
impl Transient for XrDerivedFake {
    fn dynamic_type(&self) -> &'static RuntimeType {
        // Native subclasses without their own RTTI inherit the exact XR descriptor.
        AisXrTrackedDevice::static_type()
    }
    fn rust_interface_descriptor(&self) -> &RuntimeType {
        // Native identity and Rust callback projection remain separate responsibilities.
        Self::static_type()
    }
    fn transient_base(&self) -> &StandardTransient {
        // Core/base views retain the same native reference-count subobject.
        self.core.transient_base()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.core.project(requested)
    }
}
impl AisXrTrackedDeviceSource for XrDerivedFake {
    fn xr_tracked_device_base(&self) -> &AisXrTrackedDeviceBase {
        // Qualified native geometry and nonvirtual members borrow the actual inherited state.
        self.core.xr_tracked_device_base()
    }
}
impl AisInteractiveObject for XrDerivedFake {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Context/owner links remain shared with the inherited core.
        self.core.interactive_base()
    }
}
impl PrsMgrPresentableObject for XrDerivedFake {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Bound this() resolves to this application object, as native selection owners do.
        self.core.presentable_base()
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        // Only the actual virtual accept hook changes; native base acceptance remains mode zero.
        mode == 7 || self.core.accept_display_mode(mode)
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.events.callbacks.write().push(500 + mode);
        ais_xr_tracked_device_compute(self, prs, mode)?;
        if *self.fail_after_compute.read() {
            return Err(Failure::new(
                "XR derived compute fail=true; expected completed callback",
            ));
        }
        Ok(())
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for XrDerivedFake {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // Native selected children refer back to this object through the shared bound base.
        self.core.selectable_base()
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.events.callbacks.write().push(600 + mode);
        ais_xr_tracked_device_compute_selection(self, selection, mode)
    }
}
