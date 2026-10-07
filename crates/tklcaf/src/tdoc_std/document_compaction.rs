// Rust port of ApplicationFramework/TKLCAF/TDocStd/TDocStd_Document.cxx (delta compaction
// and DumpJson) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The delta compaction (`SRN_DELTA_COMPACT`) and the JSON dump of `TDocStd_Document`.
use super::{TDocStdCompoundDelta, TDocStdDocument};
use crate::live;
use crate::tdf::{GuidMap, TdfDelta, TdfLabel};
use rustycad_tkcdf::cdm::CdmDocumentLifecycle;
use rustycad_tkcdf::json_dump::{
    base_class, class_begin, field_dumped, field_integer, field_string,
};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Guid, Handle, dump::DumpStream},
};
use std::io;

/// `NCollection_DataMap<TDF_Label, NCollection_Map<Standard_GUID>>`: the (label, ID) pairs
/// already in a compound delta.
#[derive(Default)]
pub(crate) struct LabelIdMap {
    map: DataMap<TdfLabel, GuidMap>,
}
impl LabelIdMap {
    /// Adds a pair; true when it was not there.
    pub(crate) fn add(&mut self, label: &TdfLabel, id: Guid) -> bool {
        if !self.map.is_bound(label) {
            self.map.bind(label.clone(), GuidMap::default());
        }
        self.map.change_find(label).is_ok_and(|ids| ids.add(id))
    }
    /// Whether the pair is there.
    pub(crate) fn contains(&self, label: &TdfLabel, id: &Guid) -> bool {
        self.map.find(label).is_ok_and(|ids| ids.contains(id))
    }
}

impl TDocStdDocument {
    /// `InitDeltaCompaction`: the last undo (and the first redo) start a compaction; false
    /// without undo limit or undos.
    pub fn init_delta_compaction(&self) -> bool {
        let mut state = self.write();
        state.from_redo = Handle::default();
        if state.undo_limit == 0 || state.undos.is_empty() {
            state.from_undo = Handle::default();
            return false;
        }
        state.from_undo = state.undos.last().cloned().unwrap_or_default();
        if let Some(first) = state.redos.first().cloned() {
            state.from_redo = first;
        }
        true
    }
    /// `PerformDeltaCompaction`: replaces the undos after the compaction start by one
    /// compound delta (one attribute delta per label and ID), and drops the redos before
    /// the start's redo.
    pub fn perform_delta_compaction(&self) -> Result<bool, Failure> {
        let (from_undo, from_redo, undos, redos) = {
            let s = self.read();
            (
                s.from_undo.clone(),
                s.from_redo.clone(),
                s.undos.clone(),
                s.redos.clone(),
            )
        };
        if from_undo.is_null() {
            return Ok(false);
        }
        let split = undos
            .iter()
            .position(|delta| *delta == from_undo)
            .map_or(undos.len(), |at| at + 1);
        let compound = compound_of(&undos[split..], undos.last())?;
        let mut kept = undos[..split].to_vec();
        kept.push(compound.cast());
        let kept_redos = if from_redo.is_null() {
            Vec::new()
        } else {
            let end = redos
                .iter()
                .position(|delta| *delta == from_redo)
                .map_or(redos.len(), |at| at + 1);
            redos[..end].to_vec()
        };
        let mut state = self.write();
        state.undos = kept;
        state.redos = kept_redos;
        Ok(true)
    }
    /// `DumpJson`: the CDM dump, the state flags, the delta lists and the counters.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDocStd_Document")?;
        base_class(stream, depth, &|s, d| {
            CdmDocumentLifecycle::dump_json(self, s, d)
        })?;
        let state = self.read();
        field_string(stream, "myStorageFormat", &state.storage_format.to_utf8())?;
        drop(state);
        dump_flags(self, stream)?;
        let state = self.read();
        dump_deltas(stream, depth, "anUndo.get()", &state.undos)?;
        dump_deltas(stream, depth, "aRedo.get()", &state.redos)?;
        if let Some(data) = state.data.get() {
            field_dumped(
                stream,
                depth,
                "myData.get()",
                Some(&|s: &mut DumpStream, d: i32| data.dump_json(s, d)),
            )?;
        }
        field_integer(stream, "myUndoLimit", state.undo_limit)?;
        dump_undo_transaction(stream, depth, &state.data, state.undo_transaction)?;
        dump_deltas(
            stream,
            depth,
            "myFromUndo.get()",
            std::slice::from_ref(&state.from_undo),
        )?;
        dump_deltas(
            stream,
            depth,
            "myFromRedo.get()",
            std::slice::from_ref(&state.from_redo),
        )?;
        field_integer(stream, "mySaveTime", state.save_time)?;
        field_integer(
            stream,
            "myIsNestedTransactionMode",
            i32::from(state.nested_transaction_mode),
        )?;
        dump_deltas(stream, depth, "anUndoFILO.get()", &state.undo_filo)?;
        field_integer(
            stream,
            "myOnlyTransactionModification",
            i32::from(state.only_transaction_modification),
        )?;
        field_integer(
            stream,
            "mySaveEmptyLabels",
            i32::from(state.save_empty_labels),
        )?;
        field_integer(
            stream,
            "myStorageFormatVersion",
            state.storage_format_version as i32,
        )
    }
}

/// The compound delta of the undos after the compaction start.
fn compound_of(
    deltas: &[Handle<TdfDelta>],
    last: Option<&Handle<TdfDelta>>,
) -> Result<Handle<TDocStdCompoundDelta>, Failure> {
    let compound = Handle::new(TDocStdCompoundDelta::new());
    let target = live(&compound, "TDocStd_Document::PerformDeltaCompaction")?.delta();
    let mut seen = LabelIdMap::default();
    for (index, delta) in deltas.iter().enumerate() {
        let delta = live(delta, "TDocStd_Document::PerformDeltaCompaction")?;
        if index == 0 {
            let end = live(
                last.unwrap_or(&Handle::default()),
                "TDocStd_Document::PerformDeltaCompaction",
            )?
            .end_time();
            target.validity(delta.begin_time(), end);
        }
        for attribute_delta in delta.attribute_deltas() {
            let attribute = live(&attribute_delta, "TDocStd_Document::PerformDeltaCompaction")?;
            if seen.add(&attribute.label(), attribute.id()?) {
                target.add_attribute_delta(&attribute_delta);
            }
        }
    }
    Ok(compound)
}

/// The derived flags of `DumpJson` (computed members, as natively).
fn dump_flags(document: &TDocStdDocument, stream: &mut DumpStream) -> io::Result<()> {
    let flag = |result: Result<bool, Failure>| i32::from(result.unwrap_or(false));
    field_integer(stream, "IsSaved()", i32::from(document.is_saved()))?;
    field_integer(stream, "IsChanged()", flag(document.is_changed()))?;
    field_integer(stream, "IsEmpty()", flag(document.is_empty()))?;
    field_integer(stream, "IsValid()", flag(document.is_valid()))?;
    field_integer(
        stream,
        "GetAvailableUndos()",
        document.get_available_undos(),
    )?;
    field_integer(
        stream,
        "GetAvailableRedos()",
        document.get_available_redos(),
    )?;
    field_integer(
        stream,
        "HasOpenCommand()",
        i32::from(document.has_open_command()),
    )
}

/// One `OCCT_DUMP_FIELD_VALUES_DUMPED` per non-null delta.
fn dump_deltas(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    deltas: &[Handle<TdfDelta>],
) -> io::Result<()> {
    for delta in deltas.iter().filter_map(Handle::get) {
        field_dumped(
            stream,
            depth,
            field,
            Some(&|s: &mut DumpStream, d: i32| delta.dump_json(s, d)),
        )?;
    }
    Ok(())
}

/// The dump of `&myUndoTransaction`: `TDF_Transaction::DumpJson` of the "UNDO" transaction.
fn dump_undo_transaction(
    stream: &mut DumpStream,
    depth: i32,
    data: &Handle<crate::tdf::TdfData>,
    until: i32,
) -> io::Result<()> {
    let child = |s: &mut DumpStream, d: i32| -> io::Result<()> {
        class_begin(s, "TDF_Transaction")?;
        if let Some(data) = data.get() {
            field_dumped(
                s,
                d,
                "myDF.get()",
                Some(&|s2: &mut DumpStream, d2: i32| data.dump_json(s2, d2)),
            )?;
        }
        field_integer(s, "myUntilTransaction", until)?;
        field_string(s, "myName", "UNDO")
    };
    field_dumped(stream, depth, "&myUndoTransaction", Some(&child))
}
