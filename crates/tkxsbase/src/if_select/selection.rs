// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_Selection,
// IFSelect_SelectionIterator, IFSelect_SelectBase and IFSelect_Transformer (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_Selection`: a selection of entities in a graph; `IFSelect_SelectionIterator`:
//! the inputs of a selection; `IFSelect_SelectBase`: a selection without input;
//! `IFSelect_Transformer`: a transformation of a model.
use crate::interface::{
    InterfaceCheckIterator, InterfaceEntityIterator, InterfaceGraph, InterfaceGraphContent,
    InterfaceInterfaceModel, InterfaceProtocol,
};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, StandardTransient, Transient, exceptions},
    tcollection::AsciiString,
};

/// `IFSelect_Selection`, abstract: `RootResult`, `FillIterator` and `Label` are required;
/// `UniqueResult` is not virtual natively (do not override it).
///
/// ```
/// use rustycad_tkxsbase::if_select::{IFSelectSelection, IFSelectSelectModelEntities};
/// assert_eq!(IFSelectSelectModelEntities::new().label().as_bytes(), b"All Entities from Model");
/// ```
pub trait IFSelectSelection: Transient {
    /// `RootResult(G)`: the entities selected, without the ones they share.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure>;
    /// `CompleteResult(G)`: the root result and the entities it shares.
    fn complete_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        selection_complete_result(self, g)
    }
    /// `FillIterator(iter)`: the inputs of the selection.
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator);
    /// `Label`.
    fn label(&self) -> AsciiString;
    /// `HasUniqueResult` (protected): the root result has no duplicate.
    fn has_unique_result(&self) -> bool {
        false
    }
    /// `UniqueResult(G)`: the root result without duplicates (through a graph unless
    /// unique or the graph has no statuses).
    fn unique_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        let iter = self.root_result(g)?;
        if self.has_unique_result() || !g.mode_stat() {
            return Ok(iter);
        }
        let mut graph = InterfaceGraph::from_graph(g, false)?;
        graph.get_from_iter(&iter, 0)?;
        Ok(InterfaceGraphContent::from_graph(&graph)?.deref_iterator())
    }
}
impl_transient_interface!(
    dyn IFSelectSelection,
    "IFSelect_Selection",
    StandardTransient
);

/// `IFSelect_Selection::CompleteResult`: the root result with the shared entities.
pub fn selection_complete_result<S: IFSelectSelection + ?Sized>(
    sel: &S,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let iter = sel.root_result(g)?;
    let mut graph = InterfaceGraph::from_graph(g, false)?;
    for ent in iter.content() {
        graph.get_from_entity(&ent, true, 0)?;
    }
    Ok(InterfaceGraphContent::from_graph(&graph)?.deref_iterator())
}

/// The entity iterator of a graph content (the native slicing copy).
trait ContentIterator {
    fn deref_iterator(self) -> InterfaceEntityIterator;
}

impl ContentIterator for InterfaceGraphContent {
    fn deref_iterator(self) -> InterfaceEntityIterator {
        std::ops::Deref::deref(&self).clone()
    }
}

/// `IFSelect_SelectBase`, abstract: a selection without input (its `FillIterator` adds
/// nothing).
pub trait IFSelectSelectBase: IFSelectSelection {}
impl_transient_interface!(
    dyn IFSelectSelectBase,
    "IFSelect_SelectBase",
    dyn IFSelectSelection
);

/// `IFSelect_SelectBase::FillIterator`: nothing (a base selection has no input).
pub fn select_base_fill_iterator(_iter: &mut IFSelectSelectionIterator) {}

/// `IFSelect_SelectionIterator`: a list of selections.
///
/// ```
/// use rustycad_tkxsbase::if_select::IFSelectSelectionIterator;
/// let iter = IFSelectSelectionIterator::new();
/// assert!(!iter.more());
/// ```
#[derive(Clone, Debug)]
pub struct IFSelectSelectionIterator {
    thecurr: usize,
    thelist: Vec<Handle<dyn IFSelectSelection>>,
}

impl Default for IFSelectSelectionIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl IFSelectSelectionIterator {
    /// `IFSelect_SelectionIterator()`: empty.
    pub fn new() -> Self {
        Self {
            thecurr: 1,
            thelist: Vec::new(),
        }
    }
    /// `IFSelect_SelectionIterator(sel)`: the inputs of `sel`; `Standard_NullObject` for a
    /// null selection (a native null dereference).
    pub fn of_selection(sel: &Handle<dyn IFSelectSelection>) -> Result<Self, Failure> {
        let live = sel.get().ok_or_else(|| {
            exceptions::null_object(
                "IFSelect_SelectionIterator: null selection; expected a selection",
            )
        })?;
        let mut iter = Self::new();
        live.fill_iterator(&mut iter);
        Ok(iter)
    }
    /// `AddFromIter(iter)`: the remaining items of `iter` (which is consumed).
    pub fn add_from_iter(&mut self, iter: &mut IFSelectSelectionIterator) {
        while iter.more() {
            if let Ok(sel) = iter.value() {
                self.add_item(&sel);
            }
            iter.next();
        }
    }
    /// `AddItem(sel)`: nothing for null.
    pub fn add_item(&mut self, sel: &Handle<dyn IFSelectSelection>) {
        if sel.is_some() {
            self.thelist.push(sel.clone());
        }
    }
    /// `AddList(list)`: every item, null ones included (as natively).
    pub fn add_list(&mut self, list: &[Handle<dyn IFSelectSelection>]) {
        self.thelist.extend(list.iter().cloned());
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.thecurr <= self.thelist.len()
    }
    /// `Next`.
    pub fn next(&mut self) {
        self.thecurr += 1;
    }
    /// `Value`: `Standard_OutOfRange` at the end.
    pub fn value(&self) -> Result<Handle<dyn IFSelectSelection>, Failure> {
        self.thelist
            .get(self.thecurr.wrapping_sub(1))
            .cloned()
            .ok_or_else(|| {
                exceptions::out_of_range(&format!(
                    "IFSelect_SelectionIterator::Value: {}; expected [1, {}]",
                    self.thecurr,
                    self.thelist.len()
                ))
            })
    }
}

/// `IFSelect_Transformer`, abstract: a transformation of a model (`Perform`), with the
/// correspondence of the entities (`Updated`).
pub trait IFSelectTransformer: Transient {
    /// `Perform(G, protocol, checks, newmod)`: true when done; `newmod` is the new model
    /// (null when the model is changed in place).
    fn perform(
        &self,
        g: &InterfaceGraph,
        protocol: &Handle<dyn InterfaceProtocol>,
        checks: &mut InterfaceCheckIterator,
        newmod: &mut Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure>;
    /// `ChangeProtocol(newproto)`: false (the protocol is not changed) by default.
    fn change_protocol(&self, _newproto: &mut Handle<dyn InterfaceProtocol>) -> bool {
        false
    }
    /// `Updated(entfrom, entto)`: the image of `entfrom` (false when it has none).
    fn updated(
        &self,
        entfrom: &Handle<StandardTransient>,
        entto: &mut Handle<StandardTransient>,
    ) -> bool;
    /// `Label`.
    fn label(&self) -> AsciiString;
}
impl_transient_interface!(
    dyn IFSelectTransformer,
    "IFSelect_Transformer",
    StandardTransient
);
