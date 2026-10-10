//! Named callback fixtures inspect selection state and exercise reentrant and failing filters.
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, WeakHandle};
use rustycad_tkv3d::{ais::*, select_mgr::*};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

pub struct AisStateOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
    pub forced: AtomicBool,
    pub selection: SharedState<WeakHandle<dyn AisSelectionSource>>,
    pub observed: SharedState<Vec<i32>>,
}
rustycad_tkernel::impl_transient!(AisStateOwner,"Test_AisStateOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn SelectMgrEntityOwnerSource]);
impl AisStateOwner {
    pub fn new(object: &Handle<dyn SelectMgrSelectableObject>, priority: i32) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(object, priority),
            forced: AtomicBool::new(false),
            selection: SharedState::default(),
            observed: SharedState::default(),
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
}
impl SelectMgrEntityOwnerSource for AisStateOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    fn is_forced_hilight(&self) -> bool {
        if let Some(selection) = self.selection.read().upgrade().get() {
            self.observed.write().push(selection_cursor_tag(selection));
        }
        self.forced.load(Ordering::Acquire)
    }
}
pub struct AisStateFilter {
    transient: StandardTransient,
    pub mode: AtomicI32,
    pub selection: WeakHandle<dyn AisSelectionSource>,
    pub extra: AisSelectedOwner,
    pub observed: SharedState<Vec<(i32, i32, i32)>>,
}
rustycad_tkernel::impl_transient!(AisStateFilter,"Test_AisStateFilter",dyn SelectMgrFilter,transient,interfaces:[dyn SelectMgrFilter]);
impl AisStateFilter {
    pub fn new(
        selection: &Handle<dyn AisSelectionSource>,
        extra: &AisSelectedOwner,
    ) -> Handle<Self> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            mode: AtomicI32::new(0),
            selection: selection.downgrade(),
            extra: extra.clone(),
            observed: SharedState::default(),
        })
    }
}
impl SelectMgrFilter for AisStateFilter {
    fn is_ok(&self, owner: &AisSelectedOwner) -> Result<bool, Failure> {
        let selection = self.selection.upgrade();
        self.observed.write().push((
            owner.priority(),
            selection.extent(),
            selection_cursor_tag(&*selection),
        ));
        match self.mode.load(Ordering::Acquire) {
            0 => Ok(true),
            1 => Ok(false),
            2 => Ok(owner.priority() % 2 == 0),
            3 => Err(Failure::with_message(
                "Standard_Failure",
                "filter mode 3; expected a non-failing mode",
            )),
            4 => {
                selection.add_select(&self.extra)?;
                Ok(true)
            }
            _ => unreachable!(),
        }
    }
}
pub fn selection_cursor_tag(selection: &dyn AisSelectionSource) -> i32 {
    if !selection.more() {
        return -1;
    }
    selection.value().unwrap().priority()
}
