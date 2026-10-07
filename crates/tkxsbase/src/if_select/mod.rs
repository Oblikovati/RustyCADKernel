//! `IFSelect`: selections of entities, dispatches, modifiers, signatures, counters, the
//! work session and its commands; xsbase-design.md section 4. Abstract classes are
//! traits over embedded bases; the native overrides inherited by many classes are
//! functions (`extract_root_result`, `deduct_fill_iterator`...) or macros
//! (`impl_extract_selection!`) the implementations call.
mod enums;
mod int_param;
mod select_bases;
mod select_chains;
mod select_combine;
mod select_deduct;
mod select_extracts;
mod select_pointed;
mod select_types;
mod selection;
#[cfg(test)]
mod selection_more_tests;
#[cfg(test)]
mod selection_tests;

pub use enums::{
    IFSelectEditValue, IFSelectPrintCount, IFSelectPrintFail, IFSelectRemainMode,
    IFSelectReturnStatus,
};
pub use int_param::IFSelectIntParam;
pub use select_bases::{
    IFSelectSelectEntityNumber, IFSelectSelectModelEntities, IFSelectSelectModelRoots,
    IFSelectSelectShared, IFSelectSelectSharing,
};
pub use select_chains::{
    IFSelectSelectAnyList, IFSelectSelectAnyListBase, IFSelectSelectExplore,
    IFSelectSelectExploreBase, IFSelectSelectSuite, any_list_label, any_list_root_result,
    explore_label, explore_root_result,
};
pub use select_combine::{
    IFSelectSelectCombine, IFSelectSelectCombineBase, IFSelectSelectControl,
    IFSelectSelectControlBase, IFSelectSelectDiff, IFSelectSelectIntersection, IFSelectSelectUnion,
    combine_fill_iterator, control_fill_iterator,
};
pub(crate) use select_deduct::impl_extract_selection;
pub use select_deduct::{
    IFSelectSelectDeduct, IFSelectSelectDeductBase, IFSelectSelectExtract,
    IFSelectSelectExtractBase, deduct_fill_iterator, extract_label, extract_root_result,
};
pub use select_extracts::{
    IFSelectRankRange, IFSelectSelectFlag, IFSelectSelectIncorrectEntities, IFSelectSelectRange,
    IFSelectSelectRootComps, IFSelectSelectRoots,
};
pub use select_pointed::IFSelectSelectPointed;
pub use select_types::{
    IFSelectSelectAnyType, IFSelectSelectErrorEntities, IFSelectSelectType,
    IFSelectSelectUnknownEntities, any_type_sort,
};
pub use selection::{
    IFSelectSelectBase, IFSelectSelection, IFSelectSelectionIterator, IFSelectTransformer,
    select_base_fill_iterator, selection_complete_result,
};
