//! `IFSelect`: selections of entities, dispatches, modifiers, signatures, counters, the
//! work session and its commands; xsbase-design.md section 4. Abstract classes are
//! traits over embedded bases; the native overrides inherited by many classes are
//! functions (`extract_root_result`, `deduct_fill_iterator`...) or macros
//! (`impl_extract_selection!`) the implementations call.
mod context_modif;
mod context_write;
mod dispatch;
#[cfg(test)]
mod dispatch_tests;
mod enums;
mod int_param;
mod model_copier;
mod model_copier_send;
#[cfg(test)]
mod model_copier_tests;
mod model_copier_whole;
#[cfg(test)]
mod modifier_tests;
mod modifiers;
mod packet_list;
mod select_bases;
mod select_chains;
mod select_combine;
mod select_deduct;
mod select_extracts;
mod select_pointed;
mod select_signature;
mod select_types;
mod selection;
#[cfg(test)]
mod selection_more_tests;
#[cfg(test)]
mod selection_tests;
mod share_out;
mod share_out_result;
#[cfg(test)]
mod share_out_tests;
mod sign_counter;
mod signature;
mod signature_list;
#[cfg(test)]
mod signature_tests;
mod signatures;
#[cfg(test)]
mod test_work_library;
mod transform_standard;
#[cfg(test)]
mod transform_tests;
mod work_library;

pub use context_modif::IFSelectContextModif;
pub use context_write::IFSelectContextWrite;
pub use dispatch::{
    IFSelectDispGlobal, IFSelectDispPerCount, IFSelectDispPerFiles, IFSelectDispPerOne,
    IFSelectDispPerSignature, IFSelectDispatch, IFSelectDispatchBase,
};
pub use enums::{
    IFSelectEditValue, IFSelectPrintCount, IFSelectPrintFail, IFSelectRemainMode,
    IFSelectReturnStatus,
};
pub use int_param::IFSelectIntParam;
pub use model_copier::IFSelectModelCopier;
pub use model_copier_send::IFSelectCopiedModelArgs;
pub use modifiers::{
    IFSelectAppliedItem, IFSelectAppliedModifiers, IFSelectGeneralModifier,
    IFSelectGeneralModifierBase, IFSelectModifReorder, IFSelectModifier,
};
pub use packet_list::IFSelectPacketList;
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
pub use select_signature::IFSelectSelectSignature;
pub use select_types::{
    IFSelectSelectAnyType, IFSelectSelectErrorEntities, IFSelectSelectType,
    IFSelectSelectUnknownEntities, any_type_sort,
};
pub use selection::{
    IFSelectSelectBase, IFSelectSelection, IFSelectSelectionIterator, IFSelectTransformer,
    select_base_fill_iterator, selection_complete_result,
};
pub use share_out::IFSelectShareOut;
pub use share_out_result::IFSelectShareOutResult;
pub use sign_counter::{
    IFSelectCheckCounter, IFSelectGraphCounter, IFSelectSignCounter, IFSelectSignCounterOps,
};
pub(crate) use signature::impl_signature_text;
pub use signature::{IFSelectSignature, IFSelectSignatureBase, signature_name};
pub use signature_list::{
    IFSelectSignatureCounts, IFSelectSignatureEntities, IFSelectSignatureList,
    IFSelectSignatureListOps,
};
pub use signatures::{
    IFSelectSignAncestor, IFSelectSignCategory, IFSelectSignMultiple, IFSelectSignType,
    IFSelectSignValidity,
};
pub use transform_standard::IFSelectTransformStandard;
pub use work_library::{IFSelectWorkLibrary, IFSelectWorkLibraryBase};
