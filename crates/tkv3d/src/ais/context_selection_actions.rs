//! Native selection schemes compute ordered highlight differences around virtual selection.
use super::*;
use crate::select_basics::SelectBasicsSelectingVolumeManager;
use rustycad_tkernel::{
    ncollection::{Array1, IndexedMap, List},
    standard::Failure,
};
impl AisInteractiveContext {
    /// Add an eligible owner without filtering or viewer updates; `context.add_select(&owner)?`.
    pub fn add_select(&self, owner: &AisSelectedOwner) -> Result<AisStatusOfPick, Failure> {
        self.selected_container().add_select(owner)?;
        Ok(self.selected_status())
    }
    /// Add an object's global owner; `context.add_select_object(&object)?`.
    pub fn add_select_object(
        &self,
        object: &rustycad_tkernel::standard::Handle<dyn AisInteractiveObject>,
    ) -> Result<AisStatusOfPick, Failure> {
        self.add_select(&object.global_sel_owner())
    }
    /// Apply a native selection scheme and selected highlight differences; `context.select_owners(&owners,AisSelectionScheme::Replace)?`.
    pub fn select_owners(
        &self,
        owners: &Array1<'_, AisSelectedOwner>,
        scheme: AisSelectionScheme,
    ) -> Result<AisStatusOfPick, Failure> {
        let mut previous = self.previous_selected_owners();
        let selector = self.detection_selector();
        let volume = selector
            .viewer_selector_base()
            .selecting_volume
            .read()
            .clone();
        let filter = self.fields.read().filters.cast();
        self.selected_container().select_owners(
            owners,
            scheme,
            volume.is_overlap_allowed(),
            &filter,
        )?;
        let automatic = self.fields.read().automatic_hilight;
        if automatic {
            self.update_selected_highlights(&mut previous, scheme)?;
        }
        Ok(self.selected_status())
    }
    fn previous_selected_owners(&self) -> IndexedMap<AisSelectedOwner> {
        let mut previous = IndexedMap::default();
        let automatic = self.fields.read().automatic_hilight;
        if !automatic {
            return previous;
        }
        previous.resize(self.selected_container().objects().len());
        self.clear_dynamic_highlight();
        let mut walk = self.selected_walk();
        while let Some(owner) = walk.current() {
            previous.add(owner);
            walk.next();
        }
        previous
    }
    fn update_selected_highlights(
        &self,
        previous: &mut IndexedMap<AisSelectedOwner>,
        scheme: AisSelectionScheme,
    ) -> Result<(), Failure> {
        let (mut clear, mut highlight) = (List::default(), List::default());
        let mut walk = self.selected_walk();
        while let Some(owner) = walk.current() {
            selected_highlight_difference(&owner, previous, scheme, &mut clear, &mut highlight);
            walk.next();
        }
        for owner in previous.iter() {
            clear.append(owner.clone());
        }
        self.unhighlight_owners(&clear, false)?;
        self.highlight_owners(&highlight, &Default::default())
    }
}
fn selected_highlight_difference(
    owner: &AisSelectedOwner,
    previous: &mut IndexedMap<AisSelectedOwner>,
    scheme: AisSelectionScheme,
    clear: &mut List<AisSelectedOwner>,
    highlight: &mut List<AisSelectedOwner>,
) {
    if !previous.remove_key(owner) {
        highlight.append(owner.clone());
        return;
    }
    if !owner.is_auto_hilight()
        && !matches!(scheme, AisSelectionScheme::Xor | AisSelectionScheme::Add)
    {
        clear.append(owner.clone());
        highlight.append(owner.clone());
    } else if owner.is_forced_hilight() || !owner.is_auto_hilight() {
        highlight.append(owner.clone());
    }
}
