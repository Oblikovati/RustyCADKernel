// Rust port of DataExchange/TKXSBase/IFSelect/IFSelect_SelectDeduct and
// IFSelect_SelectExtract (.hxx, .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IFSelect_SelectDeduct`: a selection computed from the result of an input selection
//! (or of an alternate pointed list); `IFSelect_SelectExtract`: a deduction that keeps
//! (or removes) the input entities by a criterion (`Sort`).
use super::{IFSelectSelectPointed, IFSelectSelection, IFSelectSelectionIterator};
use crate::interface::{InterfaceEntityIterator, InterfaceGraph, InterfaceInterfaceModel};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, SharedState, StandardTransient},
    tcollection::AsciiString,
};

/// The fields of `IFSelect_SelectDeduct`: the input and the alternate list.
#[derive(Default)]
pub struct IFSelectSelectDeductBase {
    thesel: SharedState<Handle<dyn IFSelectSelection>>,
    thealt: SharedState<Handle<IFSelectSelectPointed>>,
}

/// `IFSelect_SelectDeduct`, abstract (its `FillIterator` is
/// [`deduct_fill_iterator`]).
pub trait IFSelectSelectDeduct: IFSelectSelection {
    /// The embedded fields.
    fn deduct_base(&self) -> &IFSelectSelectDeductBase;
    /// `SetInput(sel)`.
    fn set_input(&self, sel: &Handle<dyn IFSelectSelection>) {
        *self.deduct_base().thesel.write() = sel.clone();
    }
    /// `Input`.
    fn input(&self) -> Handle<dyn IFSelectSelection> {
        self.deduct_base().thesel.read().clone()
    }
    /// `HasInput`.
    fn has_input(&self) -> bool {
        self.input().is_some()
    }
    /// `HasAlternate`: the alternate list exists and is set.
    fn has_alternate(&self) -> bool {
        self.deduct_base()
            .thealt
            .read()
            .get()
            .is_some_and(IFSelectSelectPointed::is_set)
    }
    /// `Alternate`: the alternate list, created at the first call.
    fn alternate(&self) -> Handle<IFSelectSelectPointed> {
        let mut alt = self.deduct_base().thealt.write();
        if alt.is_null() {
            *alt = Handle::new(IFSelectSelectPointed::new());
        }
        alt.clone()
    }
    /// `InputResult(G)`: the alternate list when set (then cleared), else the unique
    /// result of the input (empty without input).
    fn input_result(&self, g: &InterfaceGraph) -> Result<InterfaceEntityIterator, Failure> {
        let alt = self.deduct_base().thealt.read().clone();
        if let Some(live) = alt.get().filter(|a| a.is_set()) {
            let res = live.unique_result(g)?;
            live.clear();
            return Ok(res);
        }
        match self.input().get() {
            Some(input) => input.unique_result(g),
            None => Ok(InterfaceEntityIterator::new()),
        }
    }
}
impl_transient_interface!(
    dyn IFSelectSelectDeduct,
    "IFSelect_SelectDeduct",
    dyn IFSelectSelection
);

/// `IFSelect_SelectDeduct::FillIterator`: the input.
pub fn deduct_fill_iterator<S: IFSelectSelectDeduct + ?Sized>(
    sel: &S,
    iter: &mut IFSelectSelectionIterator,
) {
    iter.add_item(&sel.input());
}

/// The fields of `IFSelect_SelectExtract`: those of the deduction and the direction.
pub struct IFSelectSelectExtractBase {
    /// The deduction fields.
    pub deduct: IFSelectSelectDeductBase,
    thesort: SharedState<bool>,
}

impl Default for IFSelectSelectExtractBase {
    /// The protected constructor: direct (keeps the entities that satisfy `Sort`).
    fn default() -> Self {
        Self {
            deduct: IFSelectSelectDeductBase::default(),
            thesort: SharedState::new(true),
        }
    }
}

/// `IFSelect_SelectExtract`, abstract: `Sort` and `ExtractLabel` are required; its
/// `RootResult` and `Label` are [`extract_root_result`] and [`extract_label`].
pub trait IFSelectSelectExtract: IFSelectSelectDeduct {
    /// The embedded fields.
    fn extract_base(&self) -> &IFSelectSelectExtractBase;
    /// `IsDirect`.
    fn is_direct(&self) -> bool {
        *self.extract_base().thesort.read()
    }
    /// `SetDirect(direct)`.
    fn set_direct(&self, direct: bool) {
        *self.extract_base().thesort.write() = direct;
    }
    /// `Sort(rank, ent, model)`: the criterion.
    fn sort(
        &self,
        rank: i32,
        ent: &Handle<StandardTransient>,
        model: &Handle<dyn InterfaceInterfaceModel>,
    ) -> Result<bool, Failure>;
    /// `SortInGraph(rank, ent, G)`: `Sort` with the model of the graph by default.
    fn sort_in_graph(
        &self,
        rank: i32,
        ent: &Handle<StandardTransient>,
        g: &InterfaceGraph,
    ) -> Result<bool, Failure> {
        self.sort(rank, ent, g.model())
    }
    /// `ExtractLabel`.
    fn extract_label(&self) -> AsciiString;
}
impl_transient_interface!(
    dyn IFSelectSelectExtract,
    "IFSelect_SelectExtract",
    dyn IFSelectSelectDeduct
);

/// `IFSelect_SelectExtract::RootResult`: the input entities whose `SortInGraph` (by
/// rank from 1) equals the direction.
pub fn extract_root_result<S: IFSelectSelectExtract + ?Sized>(
    sel: &S,
    g: &InterfaceGraph,
) -> Result<InterfaceEntityIterator, Failure> {
    let mut iter = InterfaceEntityIterator::new();
    let direct = sel.is_direct();
    for (rank, ent) in sel.input_result(g)?.content().into_iter().enumerate() {
        if sel.sort_in_graph(rank as i32 + 1, &ent, g)? == direct {
            iter.get_one_item(&ent);
        }
    }
    Ok(iter)
}

/// `IFSelect_SelectExtract::Label`: "Picked: " or "Removed: ", then `ExtractLabel`.
pub fn extract_label<S: IFSelectSelectExtract + ?Sized>(sel: &S) -> AsciiString {
    let mut label = AsciiString::new(if sel.is_direct() {
        "Picked: "
    } else {
        "Removed: "
    });
    label.assign_cat(sel.extract_label());
    label
}

/// Implements `IFSelectSelection` and `IFSelectSelectDeduct` for a selection of
/// extraction whose field `$field` is its `IFSelectSelectExtractBase` (the native
/// inherited `RootResult`, `FillIterator` and `Label`).
macro_rules! impl_extract_selection {
    ($type:ty, $field:ident) => {
        impl $crate::if_select::IFSelectSelection for $type {
            fn root_result(
                &self,
                g: &$crate::interface::InterfaceGraph,
            ) -> Result<
                $crate::interface::InterfaceEntityIterator,
                rustycad_tkernel::standard::Failure,
            > {
                $crate::if_select::extract_root_result(self, g)
            }
            fn fill_iterator(&self, iter: &mut $crate::if_select::IFSelectSelectionIterator) {
                $crate::if_select::deduct_fill_iterator(self, iter)
            }
            fn label(&self) -> rustycad_tkernel::tcollection::AsciiString {
                $crate::if_select::extract_label(self)
            }
        }
        impl $crate::if_select::IFSelectSelectDeduct for $type {
            fn deduct_base(&self) -> &$crate::if_select::IFSelectSelectDeductBase {
                &self.$field.deduct
            }
        }
    };
}
pub(crate) use impl_extract_selection;
