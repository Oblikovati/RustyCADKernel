//! Native bulk transitions retain live object/owner traversal across callbacks.
use super::{context_object_walk::AisObjectWalk, *};
use crate::prs_mgr::PrsMgrDisplayStatus;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::Failure;
impl AisInteractiveContext {
    /// Hide displayed objects and update once; `context.erase_all(true)?`.
    pub fn erase_all(&self, update: bool) -> Result<(), Failure> {
        self.change_all_displayed(PrsMgrDisplayStatus::Displayed, update)
    }
    /// Display erased objects using their defaults and update once; `context.display_all(true)?`.
    pub fn display_all(&self, update: bool) -> Result<(), Failure> {
        self.change_all_displayed(PrsMgrDisplayStatus::Erased, update)
    }
    fn change_all_displayed(
        &self,
        status: PrsMgrDisplayStatus,
        update: bool,
    ) -> Result<(), Failure> {
        let mut walk = AisObjectWalk::new(self)?;
        while let Some(object) = walk.current() {
            self.change_one_displayed(object, status)?;
            walk.next(self)?;
        }
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn change_one_displayed(
        &self,
        object: &rustycad_tkernel::standard::Handle<dyn AisInteractiveObject>,
        status: PrsMgrDisplayStatus,
    ) -> Result<(), Failure> {
        if object.display_status() != status {
            return Ok(());
        }
        if status == PrsMgrDisplayStatus::Erased {
            self.display(object, false)
        } else {
            self.erase(object, false)
        }
    }
    /// Display the selected owners' objects in live list order; `context.display_selected(true)?`.
    pub fn display_selected(&self, update: bool) -> Result<(), Failure> {
        let mut walk = self.selected_walk();
        while let Some(owner) = walk.current() {
            self.display(&owner.selectable().cast(), false)?;
            walk.next();
        }
        if update && self.nb_selected() != 0 {
            self.update_current_viewer();
        }
        Ok(())
    }
    /// Erase from the current selection head until empty; `context.erase_selected(true)?`.
    pub fn erase_selected(&self, update: bool) -> Result<(), Failure> {
        let mut found = false;
        while let Some(owner) = self.selected_walk().current() {
            self.erase(&owner.selectable().cast(), false)?;
            found = true;
        }
        if update && found {
            self.update_current_viewer();
        }
        Ok(())
    }
}
