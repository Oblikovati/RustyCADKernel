// Rust port of ApplicationFramework/TKTObj/TObj/TObj_TIntSparseArray.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_TIntSparseArray`: the OCAF attribute of a set of positive integers keyed by
//! positive IDs, with an own modification delta (the old values) for undo.
use rustycad_tkernel::{
    impl_transient,
    ncollection::SparseArray,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient, exceptions},
};
use rustycad_tklcaf::{
    guid_from_text,
    tdata_std::support::find_or_add,
    tdf::{
        TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfAttributeState,
        TdfDeltaOnModification, TdfLabel, TdfRelocationTable,
    },
};

/// `TObj_TIntSparseArray::GetID()`.
const SPARSE_ARRAY_ID: Guid = guid_from_text("7016dc0c-b118-4433-8ef3-aecdccc79198");

/// `AbsentValue`: the backup item of a value that did not exist.
const ABSENT_VALUE: i32 = -1;

/// The fields `myVector`, `myOldMap` and `myDoBackup`.
struct SparseState {
    vector: SparseArray<i32>,
    old_map: SparseArray<i32>,
    do_backup: bool,
}

/// `TObj_TIntSparseArray`; `TObjTIntSparseArray::set(&label)`.
///
/// ```
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tktobj::tobj::TObjTIntSparseArray;
/// let data = TdfData::new();
/// let array = TObjTIntSparseArray::set(&data.root()).unwrap();
/// array.set_value(5, 7).unwrap();
/// assert_eq!((array.size(), array.value(5).unwrap()), (1, 7));
/// ```
pub struct TObjTIntSparseArray {
    transient: StandardTransient,
    base: TdfAttributeBase,
    state: SharedState<SparseState>,
}
impl_transient!(TObjTIntSparseArray, "TObj_TIntSparseArray", dyn TdfAttribute, transient,
    interfaces: [dyn TdfAttribute]);

impl Default for TObjTIntSparseArray {
    /// `TObj_TIntSparseArray()`: blocks of 100 items, backup enabled.
    fn default() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            state: SharedState::new(SparseState {
                vector: SparseArray::new(100),
                old_map: SparseArray::new(100),
                do_backup: true,
            }),
        }
    }
}

impl TObjTIntSparseArray {
    /// `GetID`.
    pub fn get_id() -> Guid {
        SPARSE_ARRAY_ID
    }
    /// `TObj_TIntSparseArray()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Set(theLabel)` (static): the attribute of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<TObjTIntSparseArray>, Failure> {
        find_or_add(label, &SPARSE_ARRAY_ID, false, Self::new)
    }
    /// `Size`: the number of stored values.
    pub fn size(&self) -> usize {
        self.state.read().vector.size()
    }
    /// `GetIterator`: the `(ID, value)` pairs in increasing ID order (natively a const
    /// iterator on the array).
    pub fn get_iterator(&self) -> Vec<(usize, i32)> {
        let state = self.state.read();
        state
            .vector
            .entries()
            .map(|(id, value)| (id, *value))
            .collect()
    }
    /// `HasValue(theId)`.
    pub fn has_value(&self, id: usize) -> bool {
        self.state.read().vector.has_value(id)
    }
    /// `Value(theId)`: `Standard_OutOfRange` when no value has that ID.
    pub fn value(&self, id: usize) -> Result<i32, Failure> {
        let state = self.state.read();
        state.vector.value(id).copied().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "NCollection_SparseArray::Value(): no value at index {id}"
            ))
        })
    }
    /// `Standard_ImmutableObject` unless the framework allows modifications.
    fn check_modification_allowed(&self) -> Result<(), Failure> {
        let data = self.label().data();
        let data = rustycad_tkcdf::live(&data, "TObj_TIntSparseArray")?;
        if !data.is_modification_allowed() {
            return Err(exceptions::immutable_object(
                "Attribute TObj_TIntSparseArray is changed outside transaction",
            ));
        }
        Ok(())
    }
    /// True when changes must be recorded in the own delta: backup is on and the
    /// attribute was not created in the current transaction.
    fn records_backup(&self) -> bool {
        let label = self.label();
        let Some(data) = label.data().get().map(|data| data.transaction()) else {
            return false;
        };
        self.state.read().do_backup && self.transaction() < data
    }
    /// `SetValue(theId, theValue)`: `Standard_ImmutableObject` outside a transaction,
    /// `Standard_OutOfRange` for a non-positive ID or value.
    pub fn set_value(&self, id: usize, value: i32) -> Result<(), Failure> {
        self.check_modification_allowed()?;
        if id < 1 || value < 1 {
            return Err(exceptions::out_of_range(&format!(
                "TObj_TIntSparseArray::SetValue: id {id} and value {value}; expected both positive"
            )));
        }
        let old = {
            let mut state = self.state.write();
            let old = state.vector.value(id).copied();
            if old == Some(value) {
                return Ok(()); // no actual modification
            }
            state.vector.set_value(id, value);
            old.unwrap_or(ABSENT_VALUE)
        };
        if self.records_backup() {
            self.backup_value(id, old, value);
        }
        Ok(())
    }
    /// `UnsetValue(theId)`: `Standard_ImmutableObject` outside a transaction,
    /// `Standard_OutOfRange` for a zero ID.
    pub fn unset_value(&self, id: usize) -> Result<(), Failure> {
        self.check_modification_allowed()?;
        if id < 1 {
            return Err(exceptions::out_of_range(&format!(
                "TObj_TIntSparseArray::UnsetValue: id {id}; expected a positive id"
            )));
        }
        let old = {
            let mut state = self.state.write();
            let Some(old) = state.vector.value(id).copied() else {
                return Ok(()); // no actual modification
            };
            state.vector.unset_value(id);
            old
        };
        if self.records_backup() {
            self.backup_value(id, old, ABSENT_VALUE);
        }
        Ok(())
    }
    /// `Clear`: the values are backed up as absent, then removed.
    pub fn clear(&self) {
        if self.records_backup() {
            for (id, value) in self.get_iterator() {
                self.backup_value(id, value, ABSENT_VALUE);
            }
        }
        self.state.write().vector.clear();
    }
    /// `SetDoBackup(toDo)`.
    pub fn set_do_backup(&self, to_do: bool) {
        self.state.write().do_backup = to_do;
    }
    /// `ClearDelta`.
    pub fn clear_delta(&self) {
        self.state.write().old_map.clear();
    }
    /// `backupValue`: keeps the first old value of an ID; drops it once the new value
    /// equals it again.
    fn backup_value(&self, id: usize, current: i32, new_value: i32) {
        let mut state = self.state.write();
        match state.old_map.value(id).copied() {
            None => {
                state.old_map.bind(id, current);
            }
            Some(saved) if saved == new_value => {
                state.old_map.unbind(id);
            }
            Some(_) => {}
        }
    }
    /// The saved old values of the own delta, in ID order.
    fn old_values(&self) -> Vec<(usize, i32)> {
        let state = self.state.read();
        state
            .old_map
            .entries()
            .map(|(id, value)| (id, *value))
            .collect()
    }
}

impl TdfAttribute for TObjTIntSparseArray {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        SPARSE_ARRAY_ID
    }
    /// `BackupCopy`: a new attribute that takes this one's delta.
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        let copy = Self::new();
        {
            let mut state = self.state.write();
            if !state.old_map.is_empty() {
                copy.state.write().old_map.exchange(&mut state.old_map);
            }
        }
        Ok(Handle::new(copy).cast())
    }
    /// `Restore(theDelta)`: applies the old values saved in the delta.
    fn restore(&self, delta: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let delta: Handle<TObjTIntSparseArray> = delta.cast();
        let Some(delta) = delta.get() else {
            return Ok(());
        };
        for (id, old) in delta.old_values() {
            if old == ABSENT_VALUE {
                self.unset_value(id)?;
            } else {
                self.set_value(id, old)?;
            }
        }
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: the values (not the delta).
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target: Handle<TObjTIntSparseArray> = into.cast();
        let Some(target) = target.get() else {
            return Ok(());
        };
        let values = self.state.read().vector.clone();
        target.state.write().vector = values;
        Ok(())
    }
    /// `BeforeCommitTransaction`: a pending delta is moved into a backup.
    fn before_commit_transaction(&self) -> Result<(), Failure> {
        if !self.state.read().old_map.is_empty() {
            self.backup()?;
            self.clear_delta();
        }
        Ok(())
    }
    /// `DeltaOnModification(theDelta)`: restores without a backup (the backup data is
    /// formed inside `Restore`).
    fn apply_delta_on_modification(
        &self,
        delta: &Handle<dyn TdfDeltaOnModification>,
    ) -> Result<(), Failure> {
        let attribute =
            rustycad_tkcdf::live(delta, "TObj_TIntSparseArray::DeltaOnModification")?.attribute();
        self.restore(&attribute)
    }
    /// `AfterUndo`: the own delta is cleared.
    fn after_undo(
        &self,
        _delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        self.clear_delta();
        Ok(true)
    }
}
