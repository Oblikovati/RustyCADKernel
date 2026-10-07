// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectModelRoots,
// IFSelect_SelectModelEntities, IFSelect_SelectEntityNumber, IFSelect_SelectShared and
// IFSelect_SelectSharing (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The selections of the model (roots, all entities, one entity by number) and the
//! one-level deductions (shared, sharing).
use super::{
    IFSelectIntParam, IFSelectSelectBase, IFSelectSelectDeduct, IFSelectSelectDeductBase,
    IFSelectSelection, IFSelectSelectionIterator, deduct_fill_iterator, select_base_fill_iterator,
};
use crate::interface::{
    InterfaceEntityIterator, InterfaceGraph, InterfaceGraphContent, InterfaceModelEntities,
};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::AsciiString,
};

/// The entities of the model of a graph (a native null dereference without model).
fn model_entities(g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
    let model = g.model().get().ok_or_else(|| {
        exceptions::null_object("IFSelect: graph without model; expected a model")
    })?;
    Ok(model.entities())
}

/// Declares a selection without field other than its transient base.
macro_rules! plain_selection {
    ($(#[$doc:meta])* $name:ident, $native:literal, $parent:ty, [$($iface:ty),*]) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
        }
        impl_transient!($name, $native, $parent, transient, interfaces: [$($iface),*]);
        impl $name {
            #[doc = concat!("`", $native, "()`.")]
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

plain_selection!(
    /// `IFSelect_SelectModelRoots`: the roots of the model (the entities no one shares).
    ///
    /// ```
    /// use rustycad_tkxsbase::if_select::{IFSelectSelectModelRoots, IFSelectSelection};
    /// assert_eq!(IFSelectSelectModelRoots::new().label().as_bytes(), b"Root (not shared) Entities from Model");
    /// ```
    IFSelectSelectModelRoots, "IFSelect_SelectModelRoots", dyn IFSelectSelectBase, [dyn IFSelectSelection, dyn IFSelectSelectBase]
);

impl IFSelectSelection for IFSelectSelectModelRoots {
    /// `RootResult(G)`: the root entities of the graph.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        g.root_entities()
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        select_base_fill_iterator(iter)
    }
    fn label(&self) -> AsciiString {
        AsciiString::new("Root (not shared) Entities from Model")
    }
}
impl IFSelectSelectBase for IFSelectSelectModelRoots {}

plain_selection!(
    /// `IFSelect_SelectModelEntities`: all the entities of the model.
    IFSelectSelectModelEntities, "IFSelect_SelectModelEntities", dyn IFSelectSelectBase, [dyn IFSelectSelection, dyn IFSelectSelectBase]
);

impl IFSelectSelection for IFSelectSelectModelEntities {
    /// `RootResult(G)`: the entities of the model.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        model_entities(g)
    }
    /// `CompleteResult(G)`: the entities of the model.
    fn complete_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        model_entities(g)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        select_base_fill_iterator(iter)
    }
    fn label(&self) -> AsciiString {
        AsciiString::new("All Entities from Model")
    }
}
impl IFSelectSelectBase for IFSelectSelectModelEntities {}

/// `IFSelect_SelectEntityNumber`: the entity of a number (an integer parameter).
#[derive(Default)]
pub struct IFSelectSelectEntityNumber {
    transient: StandardTransient,
    thenum: SharedState<Handle<IFSelectIntParam>>,
}
impl_transient!(IFSelectSelectEntityNumber, "IFSelect_SelectEntityNumber", dyn IFSelectSelectBase, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectBase]);

impl IFSelectSelectEntityNumber {
    /// `IFSelect_SelectEntityNumber()`: no number.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetNumber(num)`.
    pub fn set_number(&self, num: &Handle<IFSelectIntParam>) {
        *self.thenum.write() = num.clone();
    }
    /// `Number`.
    pub fn number(&self) -> Handle<IFSelectIntParam> {
        self.thenum.read().clone()
    }
    /// The number (0 without parameter).
    fn value(&self) -> i32 {
        self.number().get().map_or(0, IFSelectIntParam::value)
    }
}

impl IFSelectSelection for IFSelectSelectEntityNumber {
    /// `RootResult(G)`: the entity of the number (empty out of 1..Size).
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        let mut iter = InterfaceEntityIterator::new();
        let num = self.value();
        if num >= 1 && num <= g.size() {
            iter.get_one_item(&g.entity(num)?);
        }
        Ok(iter)
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        select_base_fill_iterator(iter)
    }
    /// `Label`: "Entity Number " and the number.
    fn label(&self) -> AsciiString {
        AsciiString::new(&format!("Entity Number {}", self.value()))
    }
}
impl IFSelectSelectBase for IFSelectSelectEntityNumber {}

/// Declares a deduction whose only field is its deduction base.
macro_rules! plain_deduct {
    ($(#[$doc:meta])* $name:ident, $native:literal) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: StandardTransient,
            base: IFSelectSelectDeductBase,
        }
        impl_transient!($name, $native, dyn IFSelectSelectDeduct, transient, interfaces: [dyn IFSelectSelection, dyn IFSelectSelectDeduct]);
        impl $name {
            #[doc = concat!("`", $native, "()`: no input.")]
            pub fn new() -> Self {
                Self::default()
            }
        }
        impl IFSelectSelectDeduct for $name {
            fn deduct_base(&self) -> &IFSelectSelectDeductBase {
                &self.base
            }
        }
    };
}

/// The present entities of a copy of `g` where the entities `step` gives for each input
/// entity are added (the copy keeps the statuses of `g`, as natively).
fn one_level<S: IFSelectSelectDeduct + ?Sized>(
    sel: &S,
    g: &InterfaceGraph,
    step: impl Fn(&Handle<StandardTransient>) -> Result<InterfaceEntityIterator, Failure>,
) -> Result<InterfaceEntityIterator, Failure> {
    let mut graph = InterfaceGraph::from_graph(g, false)?;
    for ent in sel.input_result(g)?.content() {
        graph.get_from_iter(&step(&ent)?, 0)?;
    }
    Ok(std::ops::Deref::deref(&InterfaceGraphContent::from_graph(&graph)?).clone())
}

plain_deduct!(
    /// `IFSelect_SelectShared`: the entities shared by the input ones (one level).
    IFSelectSelectShared, "IFSelect_SelectShared"
);

impl IFSelectSelection for IFSelectSelectShared {
    /// `RootResult(G)`: the entities the input entities share, on a copy of the graph.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        one_level(self, g, |ent| g.shareds(ent))
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    fn label(&self) -> AsciiString {
        AsciiString::new("Shared Entities (one level)")
    }
}

plain_deduct!(
    /// `IFSelect_SelectSharing`: the entities sharing the input ones (one level).
    IFSelectSelectSharing, "IFSelect_SelectSharing"
);

impl IFSelectSelection for IFSelectSelectSharing {
    /// `RootResult(G)`: the entities sharing the input entities, on a copy of the graph.
    fn root_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        one_level(self, g, |ent| g.sharings(ent))
    }
    fn fill_iterator(&self, iter: &mut IFSelectSelectionIterator) {
        deduct_fill_iterator(self, iter)
    }
    fn label(&self) -> AsciiString {
        AsciiString::new("Sharing Entities (one level)")
    }
}
