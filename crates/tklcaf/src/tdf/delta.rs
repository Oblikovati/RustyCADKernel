// Rust port of ApplicationFramework/TKLCAF/TDF/TDF_Delta.hxx, .lxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TDF_Delta`: the attribute deltas of one committed transaction, valid between two times.
use super::{TdfAttributeDelta, TdfLabel};
use crate::live;
use rustycad_tkcdf::json_dump::{class_begin, field_dumped, field_integer, field_string};
use rustycad_tkernel::{
    impl_transient,
    ncollection::Map,
    standard::{Failure, Handle, SharedState, StandardTransient, dump::DumpStream},
    tcollection::ExtendedString,
};
use std::io;

#[derive(Default)]
struct DeltaState {
    begin_time: i32,
    end_time: i32,
    attribute_deltas: Vec<Handle<dyn TdfAttributeDelta>>,
    name: ExtendedString,
}

/// `TDF_Delta`; made by `TdfData` commits (`TdfDelta::new()` is the empty delta).
#[derive(Default)]
pub struct TdfDelta {
    transient: StandardTransient,
    state: SharedState<DeltaState>,
}
impl_transient!(TdfDelta, "TDF_Delta", StandardTransient, transient);

impl TdfDelta {
    /// `TDF_Delta()`: empty, valid from time 0 to 0.
    pub fn new() -> Self {
        Self::default()
    }
    /// `IsEmpty`: no attribute delta.
    pub fn is_empty(&self) -> bool {
        self.state.read().attribute_deltas.is_empty()
    }
    /// `IsApplicable(aCurrentTime)`: the delta ends at that time.
    pub fn is_applicable(&self, current_time: i32) -> bool {
        self.state.read().end_time == current_time
    }
    /// `BeginTime`.
    pub fn begin_time(&self) -> i32 {
        self.state.read().begin_time
    }
    /// `EndTime`.
    pub fn end_time(&self) -> i32 {
        self.state.read().end_time
    }
    /// `Labels(aLabelList)`: the labels touched by the delta, merged with the labels already
    /// in the list, in the iteration order of the native label map.
    pub fn labels(&self, labels: &mut Vec<TdfLabel>) {
        let mut map: Map<TdfLabel> = Map::default();
        for label in labels.iter() {
            map.add(label.clone());
        }
        for delta in self.attribute_deltas().iter().filter_map(Handle::get) {
            map.add(delta.label());
        }
        labels.clear();
        labels.extend(map.iter().cloned());
    }
    /// `AttributeDeltas`: a copy of the list, in commit order.
    pub fn attribute_deltas(&self) -> Vec<Handle<dyn TdfAttributeDelta>> {
        self.state.read().attribute_deltas.clone()
    }
    /// `Name`; `delta.name()`.
    pub fn name(&self) -> ExtendedString {
        self.state.read().name.clone()
    }
    /// `SetName`; `delta.set_name(&name)`.
    pub fn set_name(&self, name: &ExtendedString) {
        self.state.write().name = name.clone();
    }
    /// Protected `Validity`: the times between which the delta applies.
    pub fn validity(&self, begin_time: i32, end_time: i32) {
        let mut state = self.state.write();
        state.begin_time = begin_time;
        state.end_time = end_time;
    }
    /// Protected `AddAttributeDelta`: appends a non-null attribute delta.
    pub fn add_attribute_delta(&self, attribute_delta: &Handle<dyn TdfAttributeDelta>) {
        if attribute_delta.is_some() {
            self.state
                .write()
                .attribute_deltas
                .push(attribute_delta.clone());
        }
    }
    /// Private `ReplaceDeltaList`.
    pub(crate) fn replace_delta_list(&self, list: Vec<Handle<dyn TdfAttributeDelta>>) {
        self.state.write().attribute_deltas = list;
    }
    /// Private `Apply`: applies each attribute delta in order.
    pub(crate) fn apply(&self) -> Result<(), Failure> {
        for delta in self.attribute_deltas() {
            live(&delta, "TDF_Delta::Apply")?.apply()?;
        }
        Ok(())
    }
    /// Private `BeforeOrAfterApply`: calls `BeforeUndo` (or `AfterUndo`) until every
    /// attribute accepts; when a round accepts none (a dead lock), forces the rest.
    pub(crate) fn before_or_after_apply(&self, before: bool) -> Result<(), Failure> {
        let mut pending = self.attribute_deltas();
        let mut count = pending.len();
        let mut no_dead_lock = true;
        while no_dead_lock && count != 0 {
            let mut kept = Vec::with_capacity(pending.len());
            for delta in pending {
                if !undo_callback(&delta, before, false)? {
                    kept.push(delta);
                }
            }
            pending = kept;
            no_dead_lock = count > pending.len();
            count = pending.len();
        }
        if !no_dead_lock {
            for delta in &pending {
                undo_callback(delta, before, true)?;
            }
        }
        Ok(())
    }
    /// `Dump`: the validity, the count and one line per attribute delta.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        let deltas = self.attribute_deltas();
        let (begin, end) = (self.begin_time(), self.end_time());
        writeln!(
            stream,
            "DELTA available from time \t#{begin} to time \t#{end}"
        )?;
        writeln!(stream, "Nb Attribute Delta(s): {}", deltas.len())?;
        for delta in deltas.iter().filter_map(Handle::get) {
            write!(stream, "| ")?;
            delta.dump(stream)?;
            writeln!(stream)?;
        }
        Ok(())
    }
    /// `DumpJson`: the times, the attribute deltas and the name.
    pub fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TDF_Delta")?;
        field_integer(stream, "myBeginTime", self.begin_time())?;
        field_integer(stream, "myEndTime", self.end_time())?;
        for delta in self.attribute_deltas().iter().filter_map(Handle::get) {
            let child = |s: &mut DumpStream, d: i32| delta.dump_json(s, d);
            field_dumped(stream, depth, "anAttDeltaList.get()", Some(&child))?;
        }
        field_string(stream, "myName", &self.name().to_utf8())
    }
}

/// One `BeforeUndo`/`AfterUndo` call on the attribute of a delta.
fn undo_callback(
    delta: &Handle<dyn TdfAttributeDelta>,
    before: bool,
    force: bool,
) -> Result<bool, Failure> {
    let attribute = live(delta, "TDF_Delta::BeforeOrAfterApply")?.attribute();
    let attribute = live(&attribute, "TDF_Delta::BeforeOrAfterApply")?;
    if before {
        attribute.before_undo(delta, force)
    } else {
        attribute.after_undo(delta, force)
    }
}
