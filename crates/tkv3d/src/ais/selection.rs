//! Virtual AIS selection protocol over stable native list cursors (OCCT 3d097a0).
use super::{AisSelectStatus, AisSelectionScheme};
use crate::select_mgr::{SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource, SelectMgrFilter};
use rustycad_tkernel::{
    ncollection::{Array1, DataMap, List, ListCursor},
    standard::{
        Failure, Handle, SharedState, StandardTransient, Transient, exceptions::null_object,
    },
};
/// Owner handle shared by native selection state; `AisSelectedOwner::default()`.
pub type AisSelectedOwner = Handle<dyn SelectMgrEntityOwnerSource>;
/// Protected native fields; `selection.selection_base().fields.read().cursor.more()`.
pub struct AisSelectionFields {
    /// `myresult`: stable nodes in insertion order, including native duplicate-owner cases.
    pub owners: List<AisSelectedOwner>,
    /// `myIterator`: independent live iteration state.
    pub cursor: ListCursor,
    /// `myResultMap`: the remembered position of each owner.
    pub positions: DataMap<AisSelectedOwner, ListCursor>,
}
impl Default for AisSelectionFields {
    fn default() -> Self {
        let mut positions = DataMap::default();
        positions.resize(100_000);
        Self {
            owners: List::default(),
            cursor: ListCursor::default(),
            positions,
        }
    }
}
/// State embedded by concrete and derived selections; `AisSelectionBase::new()`.
#[derive(Default)]
pub struct AisSelectionBase {
    /// Native protected state, shared by every handle to this selection.
    pub fields: SharedState<AisSelectionFields>,
}
impl AisSelectionBase {
    /// Initialize the native 100,000-entry map hint; `AisSelectionBase::new()`.
    pub fn new() -> Self {
        Self::default()
    }
}
/// Virtual selection members; internal calls preserve subclass dispatch.
pub trait AisSelectionSource: Transient {
    /// Embedded protected state; `selection.selection_base()`.
    fn selection_base(&self) -> &AisSelectionBase;
    /// Clear owner flags, list, map and iterator; `selection.clear()?`.
    fn clear(&self) -> Result<(), Failure> {
        super::selection_edit::ais_selection_clear(self.selection_base())
    }
    /// Apply the native owner/filter selection protocol; `selection.select(&owner,&filter,AisSelectionScheme::Xor,true)?`.
    fn select(
        &self,
        owner: &AisSelectedOwner,
        filter: &Handle<dyn SelectMgrFilter>,
        scheme: AisSelectionScheme,
        detected: bool,
    ) -> Result<AisSelectStatus, Failure> {
        super::selection_edit::ais_selection_select(
            self.selection_base(),
            owner,
            filter,
            scheme,
            detected,
        )
    }
    /// Append an eligible unbound owner without filtering; `selection.add_select(&owner)?`.
    fn add_select(&self, owner: &AisSelectedOwner) -> Result<AisSelectStatus, Failure> {
        super::selection_edit::ais_selection_add_select(self.selection_base(), owner)
    }
    /// Dispatch virtual Clear then Select(Add), ignoring its status; `selection.clear_and_select(&owner,&filter,true)?`.
    fn clear_and_select(
        &self,
        owner: &AisSelectedOwner,
        filter: &Handle<dyn SelectMgrFilter>,
        detected: bool,
    ) -> Result<(), Failure> {
        self.clear()?;
        self.select(owner, filter, AisSelectionScheme::Add, detected)?;
        Ok(())
    }
    /// Apply a picked array in array order; overlap is ignored natively.
    /// Example: `selection.select_owners(&picked,AisSelectionScheme::Replace,false,&filter)?`.
    fn select_owners(
        &self,
        picked: &Array1<'_, AisSelectedOwner>,
        scheme: AisSelectionScheme,
        overlap: bool,
        filter: &Handle<dyn SelectMgrFilter>,
    ) -> Result<(), Failure> {
        let _ = overlap;
        if super::selection_edit::ais_replace_extra_matches(self.selection_base(), picked, scheme) {
            return self.clear();
        }
        if matches!(
            scheme,
            AisSelectionScheme::Replace
                | AisSelectionScheme::ReplaceExtra
                | AisSelectionScheme::Clear
        ) {
            self.clear()?;
        }
        for owner in picked.as_slice() {
            self.select(owner, filter, scheme, true)?;
        }
        Ok(())
    }
    /// Protected filter-checked append; null filter is invalid for a valid owner.
    /// Example: `selection.append_owner(&owner,&filter)?`.
    fn append_owner(
        &self,
        owner: &AisSelectedOwner,
        filter: &Handle<dyn SelectMgrFilter>,
    ) -> Result<AisSelectStatus, Failure> {
        if owner.get().is_none_or(|owner| !owner.has_selectable()) {
            return Ok(AisSelectStatus::NotDone);
        }
        let filter = filter.get().ok_or_else(|| {
            null_object(
                "AIS appendOwner filter is null; expected a non-null filter for a selectable owner",
            )
        })?;
        if !filter.is_ok(owner)? {
            return Ok(AisSelectStatus::NotDone);
        }
        self.add_select(owner)
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisSelectionSource,
    "AIS_Selection",
    StandardTransient
);
/// Concrete native selection; `let selected = AisSelection::new()`.
pub struct AisSelection {
    transient: StandardTransient,
    base: AisSelectionBase,
}
rustycad_tkernel::impl_transient!(AisSelection,"AIS_Selection",StandardTransient,transient,interfaces:[dyn AisSelectionSource]);
impl AisSelection {
    /// Start empty with native cursor and map defaults; `AisSelection::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: AisSelectionBase::new(),
        }
    }
}
impl Default for AisSelection {
    fn default() -> Self {
        Self::new()
    }
}
impl AisSelectionSource for AisSelection {
    fn selection_base(&self) -> &AisSelectionBase {
        &self.base
    }
}
