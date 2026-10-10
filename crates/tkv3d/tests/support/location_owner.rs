//! Named highlight owner verifies callbacks can read the context without a retained guard.
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkv3d::{
    ais::AisInteractiveContext, prs_mgr::PrsMgrPresentationManager, select_mgr::*, v3d::V3dViewer,
};
pub struct LocationOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
    context: WeakHandle<AisInteractiveContext>,
    pub modes: SharedState<Vec<i32>>,
}
rustycad_tkernel::impl_transient!(LocationOwner,"LocationOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn SelectMgrEntityOwnerSource]);
impl LocationOwner {
    pub fn new(
        object: &Handle<dyn SelectMgrSelectableObject>,
        context: &Handle<AisInteractiveContext>,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(object, 0),
            context: context.downgrade(),
            modes: SharedState::default(),
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
}
impl SelectMgrEntityOwnerSource for LocationOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    fn update_highlight_trsf(
        &self,
        viewer: &Handle<V3dViewer>,
        manager: &Handle<PrsMgrPresentationManager>,
        mode: i32,
    ) -> Result<(), Failure> {
        let context = self.context.upgrade();
        assert_eq!(*context.current_viewer(), *viewer);
        assert_eq!(*context.main_prs_mgr(), *manager);
        self.modes.write().push(mode);
        Ok(())
    }
}
