//! `SelectMgr_SensitiveEntity` and `SelectMgr_Selection`: the sensitive entities of one
//! selection mode of a selectable object, with their activity and update states.
use super::{
    SelectMgrEntityOwnerSource, SelectMgrStateOfSelection, SelectMgrTypeOfBvhUpdate,
    SelectMgrTypeOfUpdate,
};
use crate::select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers as _};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::exceptions::null_object;
use rustycad_tkernel::standard::{Failure, Handle, HandleObserver, SharedState, StandardTransient};
use rustycad_tkservice::json_dump::{class_begin, field_dumped, field_integer};
use std::io;

/// A sensitive entity of a selection with its activity flag.
///
/// Example: `let entity = SelectMgrSensitiveEntity::new(&sensitive); entity.set_active_for_selection();`.
pub struct SelectMgrSensitiveEntity {
    transient: StandardTransient,
    sensitive: SharedState<Handle<dyn Select3DSensitiveEntity>>,
    is_active_for_selection: SharedState<bool>,
}
rustycad_tkernel::impl_transient!(
    SelectMgrSensitiveEntity,
    "SelectMgr_SensitiveEntity",
    StandardTransient,
    transient
);

impl SelectMgrSensitiveEntity {
    /// The entity of a sensitive, inactive; `SelectMgrSensitiveEntity::new(&sensitive)`.
    pub fn new(entity: &Handle<dyn Select3DSensitiveEntity>) -> Self {
        Self {
            transient: StandardTransient::default(),
            sensitive: SharedState::new(entity.clone()),
            is_active_for_selection: SharedState::new(false),
        }
    }
    /// `Clear`: clears and forgets the sensitive; `entity.clear()`.
    pub fn clear(&self) {
        let sensitive = std::mem::take(&mut *self.sensitive.write());
        if let Some(value) = sensitive.get() {
            value.clear();
        }
    }
    /// `BaseSensitive`: the sensitive, null once cleared; `entity.base_sensitive()`.
    pub fn base_sensitive(&self) -> Handle<dyn Select3DSensitiveEntity> {
        self.sensitive.read().clone()
    }
    /// `IsActiveForSelection`; `entity.is_active_for_selection()`.
    pub fn is_active_for_selection(&self) -> bool {
        *self.is_active_for_selection.read()
    }
    /// `ResetSelectionActiveStatus`; `entity.reset_selection_active_status()`.
    pub fn reset_selection_active_status(&self) {
        *self.is_active_for_selection.write() = false;
    }
    /// `SetActiveForSelection`; `entity.set_active_for_selection()`.
    pub fn set_active_for_selection(&self) {
        *self.is_active_for_selection.write() = true;
    }
    /// `DumpJson`; `entity.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_SensitiveEntity")?;
        if let Some(sensitive) = self.base_sensitive().get() {
            field_dumped(stream, depth, "mySensitive.get()", &|s, d| {
                sensitive.dump_json(s, d)
            })?;
        }
        field_integer(
            stream,
            "myIsActiveForSelection",
            i32::from(self.is_active_for_selection()),
        )
    }
}

/// The mutable fields of a selection.
#[derive(Clone)]
struct SelectionState {
    entities: Vec<Handle<SelectMgrSensitiveEntity>>,
    update_status: SelectMgrTypeOfUpdate,
    selection_state: SelectMgrStateOfSelection,
    bvh_update_status: SelectMgrTypeOfBvhUpdate,
    sens_factor: i32,
    is_custom_sens: bool,
}

/// The sensitive entities of one selection mode.
///
/// Example: `let selection = SelectMgrSelection::new(0); selection.add(&sensitive)?;`.
pub struct SelectMgrSelection {
    transient: StandardTransient,
    mode: i32,
    state: SharedState<SelectionState>,
}
rustycad_tkernel::impl_transient!(
    SelectMgrSelection,
    "SelectMgr_Selection",
    StandardTransient,
    transient
);

impl SelectMgrSelection {
    /// An empty selection of a mode; `SelectMgrSelection::new(0)`.
    pub fn new(mode_idx: i32) -> Self {
        let state = SelectionState {
            entities: Vec::new(),
            // The native constructor leaves myUpdateStatus uninitialized; the port starts at 0 (Full).
            update_status: SelectMgrTypeOfUpdate::Full,
            selection_state: SelectMgrStateOfSelection::Unknown,
            bvh_update_status: SelectMgrTypeOfBvhUpdate::None,
            sens_factor: 2,
            is_custom_sens: false,
        };
        Self {
            transient: StandardTransient::default(),
            mode: mode_idx,
            state: SharedState::new(state),
        }
    }
    /// `Destroy`: the sensitives forget their owner and the sensitivity is reset to 2.
    /// Example: `selection.destroy()`.
    pub fn destroy(&self) {
        for sensitive in self
            .entities()
            .iter()
            .filter_map(Handle::get)
            .map(SelectMgrSensitiveEntity::base_sensitive)
        {
            if let Some(value) = sensitive.get() {
                value.set(&Handle::default());
            }
        }
        self.state.write().sens_factor = 2;
    }
    /// `Add`: a new entity of the sensitive, active when the selection is activated, with the
    /// custom sensitivity (or raising the selection's); a null sensitive raises
    /// `Standard_NullObject`. Example: `selection.add(&sensitive)?`.
    pub fn add(&self, sensitive: &Handle<dyn Select3DSensitiveEntity>) -> Result<(), Failure> {
        let Some(value) = sensitive.get() else {
            return Err(null_object(
                "Null sensitive entity is added to the selection; expected a sensitive entity",
            ));
        };
        let entity = Handle::new(SelectMgrSensitiveEntity::new(sensitive));
        let (activated, custom, factor) = {
            let mut state = self.state.write();
            state.entities.push(entity.clone());
            (
                state.selection_state == SelectMgrStateOfSelection::Activated,
                state.is_custom_sens,
                state.sens_factor,
            )
        };
        if activated && !entity.is_active_for_selection() {
            entity.set_active_for_selection();
        }
        if custom {
            return value.set_sensitivity_factor(factor);
        }
        let mut state = self.state.write();
        state.sens_factor = state.sens_factor.max(value.sensitivity_factor());
        Ok(())
    }
    /// `Clear`: clears and forgets every entity; `selection.clear()`.
    pub fn clear(&self) {
        let entities = std::mem::take(&mut self.state.write().entities);
        entities
            .iter()
            .filter_map(Handle::get)
            .for_each(SelectMgrSensitiveEntity::clear);
    }
    /// `IsEmpty`; `selection.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.state.read().entities.is_empty()
    }
    /// `Mode`; `selection.mode()`.
    pub fn mode(&self) -> i32 {
        self.mode
    }
    /// `Entities`; `selection.entities()`.
    pub fn entities(&self) -> Vec<Handle<SelectMgrSensitiveEntity>> {
        self.state.read().entities.clone()
    }
    /// `ChangeEntities`: edits the entities; `selection.change_entities(&mut |e| e.clear())`.
    pub fn change_entities(
        &self,
        edit: &mut dyn FnMut(&mut Vec<Handle<SelectMgrSensitiveEntity>>),
    ) {
        edit(&mut self.state.write().entities);
    }
    /// `UpdateStatus()`; `selection.update_status()`.
    pub fn update_status(&self) -> SelectMgrTypeOfUpdate {
        self.state.read().update_status
    }
    /// `UpdateStatus(status)`; `selection.set_update_status(SelectMgrTypeOfUpdate::Full)`.
    pub fn set_update_status(&self, status: SelectMgrTypeOfUpdate) {
        self.state.write().update_status = status;
    }
    /// `UpdateBVHStatus`; `selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Add)`.
    pub fn update_bvh_status(&self, status: SelectMgrTypeOfBvhUpdate) {
        self.state.write().bvh_update_status = status;
    }
    /// `BVHUpdateStatus`; `selection.bvh_update_status()`.
    pub fn bvh_update_status(&self) -> SelectMgrTypeOfBvhUpdate {
        self.state.read().bvh_update_status
    }
    /// `GetSelectionState`; `selection.get_selection_state()`.
    pub fn get_selection_state(&self) -> SelectMgrStateOfSelection {
        self.state.read().selection_state
    }
    /// `SetSelectionState`; `selection.set_selection_state(SelectMgrStateOfSelection::Activated)`.
    pub fn set_selection_state(&self, state: SelectMgrStateOfSelection) {
        self.state.write().selection_state = state;
    }
    /// `Sensitivity`; `selection.sensitivity()`.
    pub fn sensitivity(&self) -> i32 {
        self.state.read().sens_factor
    }
    /// `SetSensitivity`: a custom sensitivity given to every sensitive; a negative value raises
    /// `Standard_ProgramError` at the first sensitive. Example: `selection.set_sensitivity(4)?`.
    pub fn set_sensitivity(&self, new_sens: i32) -> Result<(), Failure> {
        {
            let mut state = self.state.write();
            (state.sens_factor, state.is_custom_sens) = (new_sens, true);
        }
        for sensitive in self
            .entities()
            .iter()
            .filter_map(Handle::get)
            .map(SelectMgrSensitiveEntity::base_sensitive)
        {
            if let Some(value) = sensitive.get() {
                value.set_sensitivity_factor(new_sens)?;
            }
        }
        Ok(())
    }
    /// `DumpJson`: the distinct owners, the entities and the states.
    /// Example: `selection.dump_json(&mut stream, -1)?`.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "SelectMgr_Selection")?;
        let state = self.state.read().clone();
        let mut owners: Vec<Handle<dyn SelectMgrEntityOwnerSource>> = Vec::new();
        for sensitive in state
            .entities
            .iter()
            .filter_map(Handle::get)
            .map(SelectMgrSensitiveEntity::base_sensitive)
        {
            let Some(value) = sensitive.get() else {
                continue;
            };
            let owner = value.owner_id();
            if owners
                .iter()
                .any(|o| o.object_address() == owner.object_address())
            {
                continue;
            }
            if let Some(o) = owner.get() {
                field_dumped(stream, depth, "anOwner.get()", &|s, d| o.dump_json(s, d))?;
            }
            owners.push(owner);
        }
        for entity in state.entities.iter().filter_map(Handle::get) {
            field_dumped(stream, depth, "anEntity.get()", &|s, d| {
                entity.dump_json(s, d)
            })?;
        }
        field_integer(stream, "myMode", self.mode)?;
        field_integer(stream, "mySelectionState", state.selection_state as i32)?;
        field_integer(stream, "mySensFactor", state.sens_factor)?;
        field_integer(stream, "myUpdateStatus", state.update_status as i32)?;
        field_integer(stream, "myBVHUpdateStatus", state.bvh_update_status as i32)?;
        field_integer(stream, "myIsCustomSens", i32::from(state.is_custom_sens))
    }
}

impl Drop for SelectMgrSelection {
    /// The destructor calls `Destroy`.
    fn drop(&mut self) {
        self.destroy();
    }
}

impl SelectMgrSelection {
    pub(crate) fn native_entity_count(&self) -> usize {
        let state = self.state.read();
        state.entities.len()
    }
    pub(crate) fn native_owner_at(
        &self,
        index: usize,
    ) -> Option<HandleObserver<dyn SelectMgrEntityOwnerSource>> {
        let state = self.state.read();
        let entity = state.entities.get(index)?.observe();
        drop(state);
        let sensitive = entity.sensitive.read().observe();
        let fields = sensitive.sensitive_base().fields.read();
        Some(fields.owner_id.observe())
    }
}
