//! `Transfer`: the transfer of starting objects (entities of a model, or finders) into
//! results (binders), driven by actors and recorded by transfer processes;
//! docs/xsbase-design.md section 3.
mod actor;
mod binder;
#[cfg(test)]
mod binder_tests;
mod binders;
mod enums;
mod finder;
mod finder_process;
mod kinds;
mod process;
mod process_actors;
mod process_bindings;
mod process_iterator;
mod process_ops;
mod process_results;
#[cfg(test)]
mod process_tests;
mod process_transfer;
mod result_from_model;
mod result_from_transient;
#[cfg(test)]
mod result_tests;
mod transfer_iterator;
mod transfer_output;
mod transient_mapper;
mod transient_process;

pub use actor::{
    TransferActor, TransferActorOfProcessForFinder, TransferActorOfProcessForTransient,
    TransferActorOps,
};
pub use binder::{TransferBinder, TransferBinderBase};
pub use binders::{
    TransferMultipleBinder, TransferSimpleBinderOfTransient, TransferTransientListBinder,
    TransferVoidBinder,
};
pub use enums::{TransferStatusExec, TransferStatusResult, transfer_dead_loop, transfer_failure};
pub use finder::{TransferFindHasher, TransferFinder, TransferFinderBase};
pub use finder_process::TransferFinderProcess;
pub use kinds::{TransferFinderKind, TransferProcessKind, TransferTransientKind};
pub use process::TransferProcess;
pub use process_actors::{
    TransferActorOfFinderProcess, TransferActorOfFinderProcessOps, TransferActorOfTransientProcess,
    TransferActorOfTransientProcessOps, TransferShapeProcessing, transferring_by_finder_process,
    transferring_by_transient_process,
};
pub use process_iterator::TransferIteratorOfProcess;
pub use process_ops::{
    TransferProcessForFinder, TransferProcessForTransient, TransferProcessOps, bind_new,
};
pub use result_from_model::TransferResultFromModel;
pub use result_from_transient::TransferResultFromTransient;
pub use transfer_iterator::TransferTransferIterator;
pub use transfer_output::TransferOutput;
pub use transient_mapper::{TransferDataInfo, TransferTransientMapper};
pub use transient_process::{GraphGuard, TransferTransientProcess};
