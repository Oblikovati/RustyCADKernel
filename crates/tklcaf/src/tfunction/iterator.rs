// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_Iterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TFunction_Iterator`: walks the functions of a scope from the roots along the "next"
//! links, level by level, optionally following the execution status.
use super::{TFunctionExecutionStatus, TFunctionGraphNode, TFunctionIFunction, TFunctionScope};
use crate::live;
use crate::tdata_std::{TDataStdGenericExtString, TDataStdName};
use crate::tdf::{TdfLabel, TdfLabelMap};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle},
    tcollection::AsciiString,
};
use std::io;

/// `TFunction_Iterator`; `TFunctionIterator::new(&access)`.
///
/// ```
/// use rustycad_tklcaf::{tfunction::TFunctionIterator, tdf::TdfData};
/// let data = TdfData::new();
/// let iterator = TFunctionIterator::new(&data.root()).unwrap();
/// assert!(!iterator.more().unwrap());
/// ```
#[derive(Clone, Default)]
pub struct TFunctionIterator {
    current: Vec<TdfLabel>,
    usage_of_execution_status: bool,
    passed_functions: TdfLabelMap,
    scope: Handle<TFunctionScope>,
}

/// The graph node of a function label; `Standard_NoSuchObject` without one.
fn graph_node(label: &TdfLabel) -> Result<Handle<TFunctionGraphNode>, Failure> {
    TFunctionIFunction::new(label).get_graph_node()
}

impl TFunctionIterator {
    /// `TFunction_Iterator()`: no scope, nothing to iterate.
    pub fn empty() -> Self {
        Self::default()
    }
    /// `TFunction_Iterator(Access)`: `Init(Access)`.
    pub fn new(access: &TdfLabel) -> Result<Self, Failure> {
        let mut iterator = Self::empty();
        iterator.init(access)?;
        Ok(iterator)
    }
    /// `Init(Access)` (virtual natively, never overridden): the root functions (without
    /// previous function; in execution mode, only the not executed ones).
    pub fn init(&mut self, access: &TdfLabel) -> Result<(), Failure> {
        self.current.clear();
        self.passed_functions.clear(false);
        self.scope = TFunctionScope::set(access)?;
        for label in self.scope_labels()? {
            let node = graph_node(&label)?;
            let node = live(&node, "TFunction_Iterator::Init")?;
            let status = node.get_status();
            let is_root = node.get_previous().is_empty();
            if !is_root
                || (self.usage_of_execution_status
                    && status != TFunctionExecutionStatus::NotExecuted)
            {
                continue;
            }
            self.current.push(label.clone());
            if !self.usage_of_execution_status {
                self.passed_functions.add(label);
            }
        }
        Ok(())
    }
    /// The function labels of the scope, in map order; `Standard_NullObject` without scope.
    fn scope_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        let scope = live(&self.scope, "TFunction_Iterator")?;
        Ok(scope
            .get_functions()
            .items()
            .map(|(_, label)| label.clone())
            .collect())
    }
    /// `SetUsageOfExecutionStatus(usage)`.
    pub fn set_usage_of_execution_status(&mut self, usage: bool) {
        self.usage_of_execution_status = usage;
    }
    /// `GetUsageOfExecutionStatus`.
    pub fn get_usage_of_execution_status(&self) -> bool {
        self.usage_of_execution_status
    }
    /// `GetMaxNbThreads` (virtual natively, never overridden): the largest number of
    /// functions current at once, walking from the current ones without execution status.
    /// The walk has no scope natively, so reaching a next function is `Standard_NullObject`
    /// (a null dereference natively).
    pub fn get_max_nb_threads(&self) -> Result<i32, Failure> {
        let mut walker = Self::empty();
        walker.current = self.current.clone();
        let mut threads = 0;
        while !walker.current().is_empty() {
            threads = threads.max(walker.current().len() as i32);
            walker.next()?;
        }
        Ok(threads)
    }
    /// `Current` (virtual natively, never overridden): the current functions.
    pub fn current(&self) -> &[TdfLabel] {
        &self.current
    }
    /// `More` (virtual natively, never overridden): in execution mode, a function of the
    /// scope is not executed; else there are current functions.
    pub fn more(&self) -> Result<bool, Failure> {
        if !self.usage_of_execution_status {
            return Ok(!self.current.is_empty());
        }
        for label in self.scope_labels()? {
            if self.get_status(&label)? == TFunctionExecutionStatus::NotExecuted {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// `Next` (virtual natively, never overridden): the next functions of the current
    /// ones become current (in execution mode: the waiting ones stay, failed ones stop,
    /// and a next function waits for all its previous functions to succeed).
    #[allow(clippy::should_implement_trait)] // the native name Next, which can fail here
    pub fn next(&mut self) -> Result<(), Failure> {
        let mut next_current = TdfLabelMap::default();
        for label in self.current.clone() {
            self.advance_from(&label, &mut next_current)?;
        }
        self.current = next_current.iter().cloned().collect();
        Ok(())
    }
    /// The loop body of `Next` for one current function.
    fn advance_from(
        &mut self,
        label: &TdfLabel,
        next_current: &mut TdfLabelMap,
    ) -> Result<(), Failure> {
        let node = graph_node(label)?;
        let node = live(&node, "TFunction_Iterator::Next")?;
        let status = node.get_status();
        if self.usage_of_execution_status {
            use TFunctionExecutionStatus as Status;
            if matches!(status, Status::NotExecuted | Status::Executing) {
                next_current.add(label.clone());
                return Ok(());
            }
            if matches!(status, Status::WrongDefinition | Status::Failed) {
                return Ok(());
            }
        }
        let next_ids: Vec<i32> = node.get_next().iter().copied().collect();
        for id in next_ids {
            let next = live(&self.scope, "TFunction_Iterator::Next")?.get_function(id)?;
            if self.usage_of_execution_status && !self.is_ready(&next)? {
                continue;
            }
            if !self.usage_of_execution_status && self.passed_functions.contains(&next) {
                continue;
            }
            next_current.add(next.clone());
            if !self.usage_of_execution_status {
                self.passed_functions.add(next);
            }
        }
        Ok(())
    }
    /// The execution-mode check of a next function: it waits (not executed or executing)
    /// and all its previous functions succeeded.
    fn is_ready(&self, next: &TdfLabel) -> Result<bool, Failure> {
        let node = graph_node(next)?;
        let node = live(&node, "TFunction_Iterator::Next")?;
        if !matches!(
            node.get_status(),
            TFunctionExecutionStatus::NotExecuted | TFunctionExecutionStatus::Executing
        ) {
            return Ok(false);
        }
        let previous_ids: Vec<i32> = node.get_previous().iter().copied().collect();
        for id in previous_ids {
            let previous = live(&self.scope, "TFunction_Iterator::Next")?.get_function(id)?;
            let mut previous_node: Handle<TFunctionGraphNode> = Handle::default();
            previous.find_attribute_as(&TFunctionGraphNode::get_id(), &mut previous_node)?;
            if live(&previous_node, "TFunction_Iterator::Next")?.get_status()
                != TFunctionExecutionStatus::Succeeded
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
    /// `GetStatus(func)`: the status of the function's graph node.
    pub fn get_status(&self, function: &TdfLabel) -> Result<TFunctionExecutionStatus, Failure> {
        TFunctionIFunction::new(function).get_status()
    }
    /// `SetStatus(func, status)`.
    pub fn set_status(
        &self,
        function: &TdfLabel,
        status: TFunctionExecutionStatus,
    ) -> Result<(), Failure> {
        TFunctionIFunction::new(function).set_status(status)
    }
    /// `Dump`: `"Functions:"`, then one line per execution level with the names of its
    /// functions; the statuses are set to "not executed" for the walk and restored after.
    pub fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "Functions:")?;
        let Some(first) = self.current.first() else {
            return Ok(());
        };
        let saved = save_statuses(first).map_err(io::Error::other)?;
        let outcome = dump_levels(first, stream);
        restore_statuses(&saved).map_err(io::Error::other)?;
        outcome
    }
}

/// The status of every function of the scope of `access`, each set to "not executed".
fn save_statuses(access: &TdfLabel) -> Result<DataMap<TdfLabel, i32>, Failure> {
    let mut saved = DataMap::default();
    let scope = TFunctionScope::set(access)?;
    let labels: Vec<TdfLabel> = live(&scope, "TFunction_Iterator::Dump")?
        .get_functions()
        .items()
        .map(|(_, l)| l.clone())
        .collect();
    for label in labels {
        let mut node: Handle<TFunctionGraphNode> = Handle::default();
        if label.find_attribute_as(&TFunctionGraphNode::get_id(), &mut node)? {
            let node = live(&node, "TFunction_Iterator::Dump")?;
            saved.bind(label, node.get_status() as i32);
            node.set_status(TFunctionExecutionStatus::NotExecuted)?;
        }
    }
    Ok(saved)
}

/// The execution walk of `Dump`: each level's function names, each marked "succeeded".
fn dump_levels(first: &TdfLabel, stream: &mut dyn io::Write) -> io::Result<()> {
    let mut walker = TFunctionIterator::new(first).map_err(io::Error::other)?;
    walker.usage_of_execution_status = true;
    while walker.more().map_err(io::Error::other)? {
        for label in walker.current().to_vec() {
            dump_function(&label, stream)?;
            stream.write_all(b"\t")?;
        }
        walker.next().map_err(io::Error::other)?;
        writeln!(stream)?;
    }
    Ok(())
}

/// The name of one function (when it has one), then its status set to "succeeded".
fn dump_function(label: &TdfLabel, stream: &mut dyn io::Write) -> io::Result<()> {
    let mut name: Handle<TDataStdName> = Handle::default();
    let found = label
        .find_attribute_as(&TDataStdName::get_id(), &mut name)
        .map_err(io::Error::other)?;
    if let Some(name) = name.get().filter(|_| found) {
        stream.write_all(
            AsciiString::from_extended(&TDataStdGenericExtString::get(name), None).as_bytes(),
        )?;
    }
    let mut node: Handle<TFunctionGraphNode> = Handle::default();
    let found = label
        .find_attribute_as(&TFunctionGraphNode::get_id(), &mut node)
        .map_err(io::Error::other)?;
    if let Some(node) = node.get().filter(|_| found) {
        node.set_status(TFunctionExecutionStatus::Succeeded)
            .map_err(io::Error::other)?;
    }
    Ok(())
}

/// Puts back the statuses `save_statuses` recorded.
fn restore_statuses(saved: &DataMap<TdfLabel, i32>) -> Result<(), Failure> {
    for (label, status) in saved.items() {
        let mut node: Handle<TFunctionGraphNode> = Handle::default();
        if label.find_attribute_as(&TFunctionGraphNode::get_id(), &mut node)? {
            live(&node, "TFunction_Iterator::Dump")?.set_status(status_from(*status))?;
        }
    }
    Ok(())
}

/// The `(TFunction_ExecutionStatus)int` cast of `Dump`.
fn status_from(value: i32) -> TFunctionExecutionStatus {
    use TFunctionExecutionStatus as Status;
    [
        Status::WrongDefinition,
        Status::NotExecuted,
        Status::Executing,
        Status::Succeeded,
        Status::Failed,
    ]
    .get(usize::try_from(value).unwrap_or(0))
    .copied()
    .unwrap_or_default()
}
