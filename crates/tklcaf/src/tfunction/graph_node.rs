// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_GraphNode.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TFunction_GraphNode`: the previous and next functions (by scope ID) of a function, and
//! its execution status.
use super::{TFunctionExecutionStatus, TFunctionScope};
use crate::guid_from_text;
use crate::live;
use crate::tdata_std::support::cast_attribute;
use crate::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeState, TdfLabel, TdfRelocationTable, dump_attribute,
};
use rustycad_tkernel::{
    impl_transient,
    ncollection::Map,
    standard::{Failure, Guid, Handle, SharedState, StandardTransient},
};
use std::io;
use std::sync::RwLockReadGuard;

/// `TFunction_GraphNode::GetID()`.
const GRAPH_NODE_ID: Guid = guid_from_text("DD51FA86-E171-41a4-A2C1-3A0FBF286798");

/// `TFunction_GraphNode`; `TFunctionGraphNode::set(&label)`. `GetPrevious`/`GetNext` lend
/// the read guard of the map (natively a reference to it).
///
/// ```
/// use rustycad_tklcaf::{tfunction::{TFunctionExecutionStatus, TFunctionGraphNode}, tdf::TdfData};
/// let data = TdfData::new();
/// let node = TFunctionGraphNode::set(&data.root()).unwrap();
/// assert_eq!(node.get_status(), TFunctionExecutionStatus::WrongDefinition);
/// assert!(node.add_next(2).unwrap() && !node.add_next(2).unwrap());
/// ```
#[derive(Default)]
pub struct TFunctionGraphNode {
    transient: StandardTransient,
    base: TdfAttributeBase,
    previous: SharedState<Map<i32>>,
    next: SharedState<Map<i32>>,
    status: SharedState<TFunctionExecutionStatus>,
}
impl_transient!(TFunctionGraphNode, "TFunction_GraphNode", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

/// The scope ID of a function label, when the scope knows it (the label overloads).
fn function_id(function: &TdfLabel) -> Result<Option<i32>, Failure> {
    let scope = TFunctionScope::set(function)?;
    let functions = live(&scope, "TFunction_GraphNode")?.get_functions();
    Ok(functions.seek2(function).copied())
}

impl TFunctionGraphNode {
    /// `Set(L)`: the graph node of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<Self>, Failure> {
        let mut found: Handle<Self> = Handle::default();
        if label.find_attribute_as(&GRAPH_NODE_ID, &mut found)? {
            return Ok(found);
        }
        let created = Handle::new(Self::new());
        label.add_attribute(&created.cast(), false)?;
        Ok(created)
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        GRAPH_NODE_ID
    }
    /// `TFunction_GraphNode()`: no links, `TFunction_ES_WrongDefinition`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Adds `id` to `map` through a backup; false when present.
    fn add_to(&self, map: &SharedState<Map<i32>>, id: i32) -> Result<bool, Failure> {
        if map.read().contains(&id) {
            return Ok(false);
        }
        self.backup()?;
        Ok(map.write().add(id))
    }
    /// Removes `id` from `map` through a backup; false when absent.
    fn remove_from(&self, map: &SharedState<Map<i32>>, id: i32) -> Result<bool, Failure> {
        if !map.read().contains(&id) {
            return Ok(false);
        }
        self.backup()?;
        Ok(map.write().remove(&id))
    }
    /// Empties `map` through a backup, unless it is empty.
    fn clear_map(&self, map: &SharedState<Map<i32>>) -> Result<(), Failure> {
        if map.read().is_empty() {
            return Ok(());
        }
        self.backup()?;
        map.write().clear(false);
        Ok(())
    }
    /// `AddPrevious(funcID)`.
    pub fn add_previous(&self, function_id: i32) -> Result<bool, Failure> {
        self.add_to(&self.previous, function_id)
    }
    /// `AddPrevious(func)`: false when the scope does not know the function.
    pub fn add_previous_label(&self, function: &TdfLabel) -> Result<bool, Failure> {
        match function_id(function)? {
            Some(id) => self.add_previous(id),
            None => Ok(false),
        }
    }
    /// `RemovePrevious(funcID)`.
    pub fn remove_previous(&self, function_id: i32) -> Result<bool, Failure> {
        self.remove_from(&self.previous, function_id)
    }
    /// `RemovePrevious(func)`: false when the scope does not know the function.
    pub fn remove_previous_label(&self, function: &TdfLabel) -> Result<bool, Failure> {
        match function_id(function)? {
            Some(id) => self.remove_previous(id),
            None => Ok(false),
        }
    }
    /// `GetPrevious`.
    pub fn get_previous(&self) -> RwLockReadGuard<'_, Map<i32>> {
        self.previous.read()
    }
    /// `RemoveAllPrevious`.
    pub fn remove_all_previous(&self) -> Result<(), Failure> {
        self.clear_map(&self.previous)
    }
    /// `AddNext(funcID)`.
    pub fn add_next(&self, function_id: i32) -> Result<bool, Failure> {
        self.add_to(&self.next, function_id)
    }
    /// `AddNext(func)`: false when the scope does not know the function.
    pub fn add_next_label(&self, function: &TdfLabel) -> Result<bool, Failure> {
        match function_id(function)? {
            Some(id) => self.add_next(id),
            None => Ok(false),
        }
    }
    /// `RemoveNext(funcID)`.
    pub fn remove_next(&self, function_id: i32) -> Result<bool, Failure> {
        self.remove_from(&self.next, function_id)
    }
    /// `RemoveNext(func)`: false when the scope does not know the function.
    pub fn remove_next_label(&self, function: &TdfLabel) -> Result<bool, Failure> {
        match function_id(function)? {
            Some(id) => self.remove_next(id),
            None => Ok(false),
        }
    }
    /// `GetNext`.
    pub fn get_next(&self) -> RwLockReadGuard<'_, Map<i32>> {
        self.next.read()
    }
    /// `RemoveAllNext`.
    pub fn remove_all_next(&self) -> Result<(), Failure> {
        self.clear_map(&self.next)
    }
    /// `GetStatus`.
    pub fn get_status(&self) -> TFunctionExecutionStatus {
        *self.status.read()
    }
    /// `SetStatus(status)`: backed up when it changes.
    pub fn set_status(&self, status: TFunctionExecutionStatus) -> Result<(), Failure> {
        if self.get_status() == status {
            return Ok(());
        }
        self.backup()?;
        *self.status.write() = status;
        Ok(())
    }
    /// The three fields assigned to `target` (the bodies of `Restore` and `Paste`); the
    /// maps are assigned as the native `operator=` does (`Assign`), which keeps the
    /// target's buckets.
    fn copy_into(&self, target: &TFunctionGraphNode) {
        target.previous.write().assign(&self.previous.read());
        target.next.write().assign(&self.next.read());
        *target.status.write() = self.get_status();
    }
}

impl TdfAttribute for TFunctionGraphNode {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        GRAPH_NODE_ID
    }
    /// `Restore(other)`: the other's links and status.
    fn restore(&self, with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        let other = cast_attribute::<Self>(with, "TFunction_GraphNode::Restore")?;
        live(&other, "TFunction_GraphNode::Restore")?.copy_into(self);
        Ok(())
    }
    /// `Paste(into, RT)`: `into` takes the links and status, without backup, as natively.
    fn paste(
        &self,
        into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        let target = cast_attribute::<Self>(into, "TFunction_GraphNode::Paste")?;
        self.copy_into(live(&target, "TFunction_GraphNode::Paste")?);
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Dump`: the attribute dump.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        dump_attribute(self, stream)
    }
}
