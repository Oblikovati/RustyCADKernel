//! Application group composition preserves inherited native RTTI and genuine virtual protected hooks.
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase, Select3DSensitiveGroup,
    Select3DSensitiveGroupSource, Select3DSensitiveSet, Select3DSensitiveSetBase,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{
    Failure, Handle, InterfaceCaster, InterfaceEntry, RuntimeType, RuntimeTyped, SharedState,
    StandardTransient, Transient,
};
use std::{
    any::{Any, TypeId},
    io,
    sync::OnceLock,
};
pub(super) struct GroupHookFake {
    pub(super) core: Select3DSensitiveGroup,
    pub(super) trace: SharedState<Vec<i32>>,
    pub(super) failure: i32,
    pub(super) geometry: i32,
}
impl GroupHookFake {
    pub(super) fn new(core: Select3DSensitiveGroup, failure: i32) -> Self {
        // Native application subclasses without RTTI inherit the exact Group descriptor.
        Self {
            core,
            trace: SharedState::new(Vec::new()),
            failure,
            geometry: 0,
        }
    }
    fn check_hook_failure(&self, stage: i32) -> Result<(), Failure> {
        if self.failure == stage {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!("group protected callback {stage}: expected successful selection"),
            ));
        }
        Ok(())
    }
}
impl RuntimeTyped for GroupHookFake {
    const TYPE_NAME: &'static str = "Martensite_GroupHookFake";
    fn static_type() -> &'static RuntimeType {
        static RUST_INTERFACES: OnceLock<Handle<RuntimeType>> = OnceLock::new();
        // A separate Rust caster descriptor must not replace the inherited native descriptor.
        RUST_INTERFACES
            .get_or_init(register_group_hook_interfaces)
            .get()
            .unwrap()
    }
    fn project_from(object: &dyn Transient) -> Option<&Self> {
        // Projection recovers the real wrapper for all registered virtual interface casters.
        rustycad_tkernel::standard::project_concrete::<Self>(object)
    }
}
fn register_group_hook_interfaces() -> Handle<RuntimeType> {
    RuntimeType::register_owned_with_interfaces::<GroupHookFake>(
        GroupHookFake::TYPE_NAME,
        std::mem::size_of::<GroupHookFake>(),
        Some(Select3DSensitiveGroup::static_type_handle()),
        group_hook_interfaces(),
    )
}
impl Transient for GroupHookFake {
    fn dynamic_type(&self) -> &'static RuntimeType {
        // Match native subclasses that inherit DynamicType rather than redeclaring RTTI.
        Select3DSensitiveGroup::static_type()
    }
    fn rust_interface_descriptor(&self) -> &RuntimeType {
        // Typed Rust views retain this wrapper's actual virtual hook implementations.
        Self::static_type()
    }
    fn project(&self, requested: TypeId) -> Option<&dyn Any> {
        if requested == TypeId::of::<Self>() {
            return Some(self);
        }
        self.core.project(requested)
    }
    fn transient_base(&self) -> &StandardTransient {
        // All interface projections own the same inherited reference-count subobject.
        self.core.transient_base()
    }
}
impl Select3DSensitiveGroupSource for GroupHookFake {
    fn sensitive_group(&self) -> &Select3DSensitiveGroup {
        // One composed state holds the same membership/cache/owner semantics as the native subobject.
        &self.core
    }
}
impl Select3DSensitiveEntity for GroupHookFake {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        // Owner and persistence remain the native group base fields.
        self.core.sensitive_base()
    }
    fn matches(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        // Forward through the source seam, not core.matches(), which would lose virtual hooks.
        super::select3d_group_matches(self, volume, pick)
    }
    fn nb_sub_elements(&self) -> i32 {
        // Native inherited NbSubElements remains distinct from Size after partial callback failure.
        if self.geometry == 3 {
            return 42;
        }
        self.core.nb_sub_elements()
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        // Inherited application groups use the same retained native cache.
        if self.geometry == 2 {
            return rustycad_tkmath::bvh::BvhBox::from_corners(
                rustycad_tkernel::ncollection::Vec3::new(-8., -7., -6.),
                rustycad_tkernel::ncollection::Vec3::new(9., 10., 11.),
            );
        }
        self.core.bounding_box()
    }
    fn center_of_geometry(&self) -> rustycad_tkmath::gp::Point3 {
        // Virtual distance helpers observe this method through the application interface.
        let center = self.core.center_of_geometry();
        if self.geometry == 1 {
            return rustycad_tkmath::gp::Point3::from(
                center.xyz() + rustycad_tkmath::gp::Xyz::new(1., 2., 3.),
            );
        }
        center
    }
    fn set(&self, owner: &Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource>) {
        // Inherited Group::Set propagates ownership to each child, unlike the entity default.
        self.core.set(owner);
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        // Native inherited copy construction returns an ordinary Group, not the application subtype.
        self.core.get_connected()
    }
    fn bvh(&self) -> Result<(), Failure> {
        // Builder callbacks must use this application source's geometry hooks.
        super::triangulation_failure::catch_triangulation_failure(|| {
            super::select3d_set_build_bvh(self)
        })
    }
    fn to_build_bvh(&self) -> bool {
        // Initial native dirty state remains visible through every projected interface.
        self.core.to_build_bvh()
    }
    fn clear(&self) {
        // Native Clear resets membership/geometry while deliberately leaving dirty state alone.
        self.core.clear();
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // The qualified native Group dump retains virtual base-state queries on this source.
        super::select3d_group_dump_json(self, stream, depth)
    }
}
impl Select3DSensitiveSet for GroupHookFake {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        // Detection and BVH lifecycle remain in the inherited native group subobject.
        self.core.set_base()
    }
    fn size(&self) -> i32 {
        // Primitive membership indices remain independently ordered by the same BVH.
        self.core.size()
    }
    fn element_box(&self, index: i32) -> Select3DBndBox3d {
        // This wrapper overrides selection hooks while retaining ordinary native geometry.
        self.core.element_box(index)
    }
    fn center(&self, index: i32, axis: i32) -> Result<f64, Failure> {
        // Native non-XYZ coordinate indices intentionally select Z.
        self.core.center(index, axis)
    }
    fn swap(&self, first: i32, second: i32) {
        // Only primitive order changes; membership/node identity remains stable.
        self.core.swap(first, second);
    }
    fn overlaps_element(
        &self,
        pick: &mut SelectBasicsPickResult,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        index: i32,
        inside: bool,
    ) -> Result<bool, Failure> {
        self.trace.write().push(100 + index);
        let hit = super::select3d_group_overlaps_element(self, pick, volume, index, inside)?;
        self.check_hook_failure(1)?;
        Ok(hit)
    }
    fn element_is_inside(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        index: i32,
        inside: bool,
    ) -> Result<bool, Failure> {
        self.trace.write().push(200 + index);
        let hit = super::select3d_group_element_is_inside(self, volume, index, inside)?;
        self.check_hook_failure(2)?;
        Ok(hit)
    }
    fn distance_to_cog(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        self.trace.write().push(300);
        let distance = super::select3d_group_distance_to_cog(self, volume)?;
        self.check_hook_failure(3)?;
        Ok(distance + 17.)
    }
}

fn group_hook_interfaces() -> Vec<InterfaceEntry> {
    vec![
        InterfaceEntry::new(InterfaceCaster::<dyn Select3DSensitiveEntity>::new(
            |object| Some(rustycad_tkernel::standard::project_concrete::<GroupHookFake>(object)?),
        )),
        InterfaceEntry::new(InterfaceCaster::<dyn Select3DSensitiveSet>::new(|object| {
            Some(rustycad_tkernel::standard::project_concrete::<GroupHookFake>(object)?)
        })),
        InterfaceEntry::new(InterfaceCaster::<dyn Select3DSensitiveGroupSource>::new(
            |object| Some(rustycad_tkernel::standard::project_concrete::<GroupHookFake>(object)?),
        )),
    ]
}
