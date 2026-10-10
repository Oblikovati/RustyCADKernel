//! The update members of `PrsMgr_PresentableObject` that go through the presentation manager:
//! `UpdatePresentations`, the deprecated `Update` and `ToBeUpdated(list)`, `recomputeComputed`
//! and `replaceAspects`.
use super::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers, PrsMgrPresentation};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsReplaceMap, Graphic3dStructureMembers as _, Graphic3dStructureSource as _,
};

/// The update members of a presentable object.
pub trait PrsMgrPresentableObjectUpdates: PrsMgrPresentableObjectMembers {
    /// `UpdatePresentations`: recomputes the presentations to update that are displayed or
    /// highlighted (or all of them when asked); true when one was recomputed.
    /// Example: `object.update_presentations(false)?`.
    fn update_presentations(&self, to_include_hidden: bool) -> Result<bool, Failure> {
        let this = self.presentable_base().this();
        let mut has_updates = false;
        for prs in self.presentations().iter().filter_map(Handle::get) {
            if !prs.must_be_updated() || !(to_include_hidden || prs.is_shown_by_manager(&this)) {
                continue;
            }
            has_updates = true;
            prs.presentation_manager().update(&this, prs.mode())?;
        }
        Ok(has_updates)
    }
    /// `Update(toIncludeHidden)` (deprecated): `SetToUpdate()` then `UpdatePresentations`.
    /// Example: `object.update_all(false)?`.
    fn update_all(&self, to_include_hidden: bool) -> Result<(), Failure> {
        self.set_to_update_all();
        self.update_presentations(to_include_hidden).map(|_| ())
    }
    /// `Update(mode, toClearOther)` (deprecated): recomputes a shown presentation of the mode
    /// (or flags it), and forgets the other presentations when asked.
    /// Example: `object.update_mode(0, false)?`.
    fn update_mode(&self, mode: i32, to_clear_other: bool) -> Result<(), Failure> {
        let this = self.presentable_base().this();
        let mut kept: Vec<Handle<PrsMgrPresentation>> = Vec::new();
        for handle in self.presentations() {
            let Some(prs) = handle.get() else {
                kept.push(handle.clone());
                continue;
            };
            if prs.mode() == mode {
                update_one(self, &this, prs, mode)?;
            } else if to_clear_other {
                // The native removal destroys the last reference, whose destructor erases.
                prs.erase();
                continue;
            }
            kept.push(handle.clone());
        }
        self.presentable_base().fields.write().presentations = kept;
        Ok(())
    }
    /// `ToBeUpdated(list)` (deprecated): the distinct modes of the presentations to update.
    /// Example: `object.to_be_updated_modes()`.
    fn to_be_updated_modes(&self) -> Vec<i32> {
        let mut modes = Vec::new();
        for prs in self.presentations().iter().filter_map(Handle::get) {
            if prs.must_be_updated() && !modes.contains(&prs.mode()) {
                modes.push(prs.mode());
            }
        }
        modes
    }
    /// `recomputeComputed`: recomputes the computed structures of every presentation.
    /// Example: `object.recompute_computed()`.
    fn recompute_computed(&self) {
        self.presentations()
            .iter()
            .filter_map(Handle::get)
            .for_each(|p| p.re_compute());
    }
    /// `replaceAspects`: replaces aspects in the groups of every presentation; nothing for an
    /// empty map. Example: `object.replace_aspects(&map)`.
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap) {
        if map.is_empty() {
            return;
        }
        for group in self.all_groups() {
            group.replace_aspects(map);
        }
    }
}
impl<T: PrsMgrPresentableObjectMembers + ?Sized> PrsMgrPresentableObjectUpdates for T {}

/// One presentation of `Update(mode, ...)`: recomputed when shown, else flagged.
fn update_one<O: PrsMgrPresentableObject + ?Sized>(
    object: &O,
    this: &Handle<dyn PrsMgrPresentableObject>,
    prs: &PrsMgrPresentation,
    mode: i32,
) -> Result<(), Failure> {
    if prs.is_shown_by_manager(this) {
        prs.presentation_manager().update(this, mode)?;
        prs.set_update_status(false);
    } else {
        object.set_to_update(prs.mode());
    }
    Ok(())
}
