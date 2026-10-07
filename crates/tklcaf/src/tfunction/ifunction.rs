// Rust port of ApplicationFramework/TKLCAF/TFunction/TFunction_IFunction.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TFunction_IFunction`: the interface of one function (its driver, graph node and
//! dependencies), and the static members that create, delete and connect functions.
use super::{
    TFunctionDriver, TFunctionDriverTable, TFunctionExecutionStatus, TFunctionFunction,
    TFunctionFunctionMap, TFunctionGraphNode, TFunctionLogbook, TFunctionScope,
};
use crate::live;
use crate::tdf::{TdfLabel, TdfLabelMap};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Guid, Handle, exceptions},
};

/// `TFunction_IFunction`; `TFunctionIFunction::new(&label)`.
///
/// ```
/// use rustycad_tklcaf::{tfunction::TFunctionIFunction, tdf::TdfData, guid_from_text};
/// let data = TdfData::new();
/// let label = data.root().find_child(1, true).unwrap();
/// let id = guid_from_text("0f2a9a10-0000-4000-8000-000000000003");
/// assert!(!TFunctionIFunction::new_function(&label, &id).unwrap(), "no driver registered");
/// assert!(TFunctionIFunction::new(&label).get_driver(0).is_err());
/// ```
#[derive(Clone, Default)]
pub struct TFunctionIFunction {
    label: TdfLabel,
}

/// The scope of a label and its functions (ID and label) in map order.
type ScopeFunctions = (Handle<TFunctionScope>, Vec<(i32, TdfLabel)>);

/// The labels of an `NCollection_DoubleMap<int, TDF_Label>` with their IDs, in map order.
fn scope_functions(label: &TdfLabel) -> Result<ScopeFunctions, Failure> {
    let scope = TFunctionScope::set(label)?;
    let functions = live(&scope, "TFunction_IFunction")?
        .get_functions()
        .items()
        .map(|(id, label)| (*id, label.clone()))
        .collect();
    Ok((scope, functions))
}

/// `Find2(L)` of the scope's functions; `Standard_NoSuchObject` for an unknown label.
fn id_of(scope: &Handle<TFunctionScope>, label: &TdfLabel) -> Result<i32, Failure> {
    live(scope, "TFunction_IFunction")?.get_function_id(label)
}

impl TFunctionIFunction {
    /// `NewFunction(L, ID)`: a function with that driver, a reset graph node and a place in
    /// the scope; true when the driver table knows the driver.
    pub fn new_function(label: &TdfLabel, driver_id: &Guid) -> Result<bool, Failure> {
        let function = TFunctionFunction::set_with_driver(label, driver_id)?;
        live(&function, "TFunction_IFunction::NewFunction")?.set_failure(0)?;
        let graph_node = TFunctionGraphNode::set(label)?;
        let graph_node = live(&graph_node, "TFunction_IFunction::NewFunction")?;
        graph_node.remove_all_previous()?;
        graph_node.remove_all_next()?;
        graph_node.set_status(TFunctionExecutionStatus::WrongDefinition)?;
        live(
            &TFunctionScope::set(label)?,
            "TFunction_IFunction::NewFunction",
        )?
        .add_function(label)?;
        Ok(live(
            &TFunctionDriverTable::get(),
            "TFunction_IFunction::NewFunction",
        )?
        .has_driver(driver_id, 0))
    }
    /// `DeleteFunction(L)`: forgets the function and its graph node (disconnecting its
    /// previous and next functions) and removes it from the scope; `Standard_NoSuchObject`
    /// when the scope does not know it.
    pub fn delete_function(label: &TdfLabel) -> Result<bool, Failure> {
        let mut function: Handle<TFunctionFunction> = Handle::default();
        if label.find_attribute_as(&TFunctionFunction::get_id(), &mut function)? {
            label.forget_attribute(&function.cast())?;
        }
        let scope = TFunctionScope::set(label)?;
        let function_id = id_of(&scope, label)?;
        let mut graph_node: Handle<TFunctionGraphNode> = Handle::default();
        if label.find_attribute_as(&TFunctionGraphNode::get_id(), &mut graph_node)? {
            disconnect(&scope, &graph_node, function_id)?;
            label.forget_attribute(&graph_node.cast())?;
        }
        live(&scope, "TFunction_IFunction::DeleteFunction")?.remove_function_label(label)?;
        Ok(true)
    }
    /// `UpdateDependencies(Access)`: rebuilds every graph node of the scope from the
    /// drivers' arguments and results.
    pub fn update_all_dependencies(access: &TdfLabel) -> Result<bool, Failure> {
        let (_, functions) = scope_functions(access)?;
        let mut table: DataMap<TdfLabel, Vec<TdfLabel>> = DataMap::default();
        for (_, label) in &functions {
            let function = TFunctionIFunction::new(label);
            let mut results = Vec::new();
            live(
                &function.get_driver(0)?,
                "TFunction_IFunction::UpdateDependencies",
            )?
            .results(&mut results);
            table.bind(label.clone(), results);
            let graph_node = function.get_graph_node()?;
            let graph_node = live(&graph_node, "TFunction_IFunction::UpdateDependencies")?;
            graph_node.remove_all_previous()?;
            graph_node.remove_all_next()?;
        }
        let table: Vec<(TdfLabel, Vec<TdfLabel>)> =
            table.items().map(|(l, r)| (l.clone(), r.clone())).collect();
        for (function_id, label) in &functions {
            connect_to_producers(*function_id, label, &table)?;
        }
        Ok(true)
    }
    /// `TFunction_IFunction()`: no label.
    pub fn empty() -> Self {
        Self::default()
    }
    /// `TFunction_IFunction(L)`.
    pub fn new(label: &TdfLabel) -> Self {
        Self {
            label: label.clone(),
        }
    }
    /// `Init(L)`.
    pub fn init(&mut self, label: &TdfLabel) {
        self.label = label.clone();
    }
    /// `Label`.
    pub fn label(&self) -> TdfLabel {
        self.label.clone()
    }
    /// `UpdateDependencies()`: connects this function with every other function of the
    /// scope that consumes its results or produces its arguments.
    pub fn update_dependencies(&self) -> Result<bool, Failure> {
        let driver = self.get_driver(0)?;
        let driver = live(&driver, "TFunction_IFunction::UpdateDependencies")?;
        let (mut arguments, mut results) = (Vec::new(), Vec::new());
        driver.arguments(&mut arguments);
        driver.results(&mut results);
        let (arguments, results) = (label_map(&arguments), label_map(&results));
        let (scope, functions) = scope_functions(&self.label)?;
        for (_, other) in functions.iter().filter(|(_, other)| *other != self.label) {
            self.connect_with(&scope, other, &arguments, &results)?;
        }
        Ok(true)
    }
    /// The loop body of `UpdateDependencies()` for one other function.
    fn connect_with(
        &self,
        scope: &Handle<TFunctionScope>,
        other: &TdfLabel,
        arguments: &TdfLabelMap,
        results: &TdfLabelMap,
    ) -> Result<(), Failure> {
        let other_function = TFunctionIFunction::new(other);
        let driver = other_function.get_driver(0)?;
        let driver = live(&driver, "TFunction_IFunction::UpdateDependencies")?;
        let mut other_arguments = Vec::new();
        driver.arguments(&mut other_arguments);
        for _ in other_arguments
            .iter()
            .filter(|argument| results.contains(argument))
        {
            live(&self.get_graph_node()?, "TFunction_IFunction")?.add_next(id_of(scope, other)?)?;
            live(&other_function.get_graph_node()?, "TFunction_IFunction")?
                .add_previous(id_of(scope, &self.label)?)?;
        }
        let mut other_results = Vec::new();
        driver.results(&mut other_results);
        for _ in other_results
            .iter()
            .filter(|result| arguments.contains(result))
        {
            live(&self.get_graph_node()?, "TFunction_IFunction")?
                .add_previous(id_of(scope, other)?)?;
            live(&other_function.get_graph_node()?, "TFunction_IFunction")?
                .add_next(id_of(scope, &self.label)?)?;
        }
        Ok(())
    }
    /// `Arguments(args)`: the driver's arguments.
    pub fn arguments(&self, arguments: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        live(&self.get_driver(0)?, "TFunction_IFunction::Arguments")?.arguments(arguments);
        Ok(())
    }
    /// `Results(res)`: the driver's results.
    pub fn results(&self, results: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        live(&self.get_driver(0)?, "TFunction_IFunction::Results")?.results(results);
        Ok(())
    }
    /// `GetPrevious(prev)`: the labels of the previous functions the scope knows.
    pub fn get_previous(&self, previous: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        let node = self.get_graph_node()?;
        let ids: Vec<i32> = live(&node, "TFunction_IFunction::GetPrevious")?
            .get_previous()
            .iter()
            .copied()
            .collect();
        append_known(&self.label, &ids, previous)
    }
    /// `GetNext(next)`: the labels of the next functions the scope knows.
    pub fn get_next(&self, next: &mut Vec<TdfLabel>) -> Result<(), Failure> {
        let node = self.get_graph_node()?;
        let ids: Vec<i32> = live(&node, "TFunction_IFunction::GetNext")?
            .get_next()
            .iter()
            .copied()
            .collect();
        append_known(&self.label, &ids, next)
    }
    /// `GetStatus`.
    pub fn get_status(&self) -> Result<TFunctionExecutionStatus, Failure> {
        Ok(live(&self.get_graph_node()?, "TFunction_IFunction::GetStatus")?.get_status())
    }
    /// `SetStatus(status)`.
    pub fn set_status(&self, status: TFunctionExecutionStatus) -> Result<(), Failure> {
        live(&self.get_graph_node()?, "TFunction_IFunction::SetStatus")?.set_status(status)
    }
    /// `GetAllFunctions`: a copy of the scope's functions.
    pub fn get_all_functions(&self) -> Result<TFunctionFunctionMap, Failure> {
        let scope = TFunctionScope::set(&self.label)?;
        Ok(live(&scope, "TFunction_IFunction::GetAllFunctions")?
            .get_functions()
            .clone())
    }
    /// `GetLogbook`: the logbook of the scope.
    pub fn get_logbook(&self) -> Result<Handle<TFunctionLogbook>, Failure> {
        live(
            &TFunctionScope::set(&self.label)?,
            "TFunction_IFunction::GetLogbook",
        )?
        .get_logbook()
    }
    /// `GetDriver(thread)`: the driver of the function's GUID, initialized with the label;
    /// `Standard_NoSuchObject` without function or registered driver.
    pub fn get_driver(&self, thread: i32) -> Result<Handle<dyn TFunctionDriver>, Failure> {
        let mut function: Handle<TFunctionFunction> = Handle::default();
        if !self
            .label
            .find_attribute_as(&TFunctionFunction::get_id(), &mut function)?
        {
            return Err(exceptions::no_such_object(
                "TFunction_IFunction::GetDriver(): A Function is not found attached to this label",
            ));
        }
        let guid = live(&function, "TFunction_IFunction::GetDriver")?.get_driver_guid();
        let mut driver: Handle<dyn TFunctionDriver> = Handle::default();
        if !live(
            &TFunctionDriverTable::get(),
            "TFunction_IFunction::GetDriver",
        )?
        .find_driver(&guid, &mut driver, thread)
        {
            let message = format!(
                "TFunction_IFunction::GetDriver(): A driver is not found for this ID {guid}"
            );
            return Err(exceptions::no_such_object(&message));
        }
        live(&driver, "TFunction_IFunction::GetDriver")?.init(&self.label);
        Ok(driver)
    }
    /// `GetGraphNode`; `Standard_NoSuchObject` without graph node.
    pub fn get_graph_node(&self) -> Result<Handle<TFunctionGraphNode>, Failure> {
        let mut graph_node: Handle<TFunctionGraphNode> = Handle::default();
        if !self
            .label
            .find_attribute_as(&TFunctionGraphNode::get_id(), &mut graph_node)?
        {
            return Err(exceptions::no_such_object(
                "TFunction_IFunction::GetStatus(): A graph node is not found attached to this label",
            ));
        }
        Ok(graph_node)
    }
}

/// A label map of `labels` (the native "maps for fast searching").
fn label_map(labels: &[TdfLabel]) -> TdfLabelMap {
    let mut map = TdfLabelMap::default();
    for label in labels {
        map.add(label.clone());
    }
    map
}

/// Appends the labels of the IDs the scope of `access` knows, in the given order.
fn append_known(access: &TdfLabel, ids: &[i32], labels: &mut Vec<TdfLabel>) -> Result<(), Failure> {
    let scope = TFunctionScope::set(access)?;
    let scope = live(&scope, "TFunction_IFunction")?;
    for id in ids.iter().filter(|id| scope.has_function(**id)) {
        labels.push(scope.get_function(*id)?);
    }
    Ok(())
}

/// The graph part of `DeleteFunction`: the previous functions forget this one as next,
/// the next ones forget it as previous.
fn disconnect(
    scope: &Handle<TFunctionScope>,
    graph_node: &Handle<TFunctionGraphNode>,
    function_id: i32,
) -> Result<(), Failure> {
    let node = live(graph_node, "TFunction_IFunction::DeleteFunction")?;
    let previous: Vec<i32> = node.get_previous().iter().copied().collect();
    let next: Vec<i32> = node.get_next().iter().copied().collect();
    for id in previous {
        if let Some(other) = graph_node_of(scope, id)?.get() {
            other.remove_next(function_id)?;
        }
    }
    for id in next {
        if let Some(other) = graph_node_of(scope, id)?.get() {
            other.remove_previous(function_id)?;
        }
    }
    Ok(())
}

/// The graph node of the function `id` (`Find1` raises `Standard_NoSuchObject` for an
/// unknown ID), null when its label has none.
fn graph_node_of(
    scope: &Handle<TFunctionScope>,
    id: i32,
) -> Result<Handle<TFunctionGraphNode>, Failure> {
    let label = live(scope, "TFunction_IFunction")?.get_function(id)?;
    let mut node: Handle<TFunctionGraphNode> = Handle::default();
    label.find_attribute_as(&TFunctionGraphNode::get_id(), &mut node)?;
    Ok(node)
}

/// The second loop of `UpdateDependencies(Access)` for one function: every other function
/// whose results include an argument of this one becomes its previous function.
fn connect_to_producers(
    function_id: i32,
    label: &TdfLabel,
    table: &[(TdfLabel, Vec<TdfLabel>)],
) -> Result<(), Failure> {
    let function = TFunctionIFunction::new(label);
    let mut arguments = Vec::new();
    live(
        &function.get_driver(0)?,
        "TFunction_IFunction::UpdateDependencies",
    )?
    .arguments(&mut arguments);
    let arguments = label_map(&arguments);
    for (other, results) in table.iter().filter(|(other, _)| other != label) {
        for _ in results.iter().filter(|result| arguments.contains(result)) {
            live(&function.get_graph_node()?, "TFunction_IFunction")?.add_previous_label(other)?;
            live(
                &TFunctionIFunction::new(other).get_graph_node()?,
                "TFunction_IFunction",
            )?
            .add_next(function_id)?;
        }
    }
    Ok(())
}
