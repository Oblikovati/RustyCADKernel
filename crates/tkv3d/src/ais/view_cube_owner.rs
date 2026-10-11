//! Native owners force detection highlight and dispatch every mouse click to the widget.
use super::{AisViewCube, AisViewCubeSource};
use crate::{select_mgr::*, v3d::V3dTypeOfOrientation};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Handle, SharedState, StandardTransient},
};
use rustycad_tkservice::aspect::{AspectVKeyFlags, AspectVKeyMouse};
/// Owner identity and orientation; `owner.main_orientation()`.
pub struct AisViewCubeOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
    /// Native protected orientation, retained for picking and camera dispatch.
    pub orientation: SharedState<V3dTypeOfOrientation>,
}
rustycad_tkernel::impl_transient!(AisViewCubeOwner,"AIS_ViewCubeOwner",dyn SelectMgrEntityOwnerSource,transient,
    interfaces:[dyn SelectMgrEntityOwnerSource]);
impl AisViewCubeOwner {
    /// Own a part with native priority five; `AisViewCubeOwner::new(&cube,orientation,5)`.
    pub fn new(
        cube: &Handle<AisViewCube>,
        orientation: V3dTypeOfOrientation,
        priority: i32,
    ) -> Handle<Self> {
        Self::with_source(&cube.cast(), orientation, priority)
    }
    /// Retain a real widget subclass; `AisViewCubeOwner::with_source(&subclass.cast(),orientation,5)`.
    pub fn with_source(
        cube: &Handle<dyn AisViewCubeSource>,
        orientation: V3dTypeOfOrientation,
        priority: i32,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(&cube.cast(), priority),
            orientation: SharedState::new(orientation),
        });
        owner.base.fields.write().from_decomposition = true;
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
    /// Read the part's target orientation; `owner.main_orientation()`.
    pub fn main_orientation(&self) -> V3dTypeOfOrientation {
        *self.orientation.read()
    }
}
impl SelectMgrEntityOwnerSource for AisViewCubeOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    fn is_forced_hilight(&self) -> bool {
        true
    }
    fn handle_mouse_click(
        &self,
        _: Vec2<i32>,
        _: AspectVKeyMouse,
        _: AspectVKeyFlags,
        _: bool,
    ) -> bool {
        let cube = self.selectable().cast::<dyn AisViewCubeSource>();
        let Some(cube) = cube.get() else {
            return false;
        };
        // Native ignores all button/modifier inputs. Invalid detached owner failures are not successful clicks.
        cube.handle_click(&self.base.this().cast()).is_ok()
    }
}
