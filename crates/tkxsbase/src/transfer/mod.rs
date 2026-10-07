//! `Transfer`: the transfer of starting objects (entities of a model, or finders) into
//! results (binders), driven by actors and recorded by transfer processes;
//! docs/xsbase-design.md section 3.
mod binder;
#[cfg(test)]
mod binder_tests;
mod binders;
mod enums;
mod finder;
mod transient_mapper;

pub use binder::{TransferBinder, TransferBinderBase};
pub use binders::{
    TransferMultipleBinder, TransferSimpleBinderOfTransient, TransferTransientListBinder,
    TransferVoidBinder,
};
pub use enums::{TransferStatusExec, TransferStatusResult, transfer_dead_loop, transfer_failure};
pub use finder::{TransferFindHasher, TransferFinder, TransferFinderBase};
pub use transient_mapper::{TransferDataInfo, TransferTransientMapper};
