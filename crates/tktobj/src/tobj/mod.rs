//! The TObj package: OCAF-based models of typed objects (tobj-design.md).
mod application;
mod application_errors;
mod assistant;
mod check_model;
mod hidden_partition;
mod label_iterator;
mod messages;
mod model;
mod model_init;
mod model_io;
mod model_iterator;
mod model_state;
mod object;
mod object_arrays;
mod object_children;
mod object_clone;
mod object_data;
mod object_detach;
mod object_iterator;
mod object_names;
mod object_references;
mod object_state;
mod ocaf_object_iterator;
mod partition;
mod persistence;
mod reference_iterator;
mod sequence_iterator;
mod t_int_sparse_array;
mod t_model;
mod t_name_container;
mod t_object;
mod t_reference;
mod t_xyz;
pub use application::{TObjApplication, TObjApplicationBase, TObjStandardApplication};
pub use assistant::TObjAssistant;
pub use check_model::TObjCheckModel;
pub use hidden_partition::TObjHiddenPartition;
pub use label_iterator::{
    TObjLabelIterator, TObjLabelIteratorBase, label_iterator_more, label_iterator_next,
};
pub use model::{TObjModel, TObjModelBase, TObjModelDataTag};
pub use model_iterator::TObjModelIterator;
pub use model_state::TObjModelState;
pub use object::{
    TObjBackReferences, TObjObject, TObjObjectBase, TObjObjectChildTag, TObjObjectDataTag,
    TObjObjectRefTag, TObjObjectStateFlags, TObjTypeFlags,
};
pub use object_arrays::{TObjArrayAttribute, TObjObjectArrays};
pub use object_clone::copy_label_references;
pub use object_data::TObjObjectData;
pub use object_iterator::{TObjObjectIterator, TObjStandardObjectIterator};
pub use object_state::TObjObjectState;
pub use ocaf_object_iterator::TObjOcafObjectIterator;
pub use partition::{
    TObjPartition, TObjPartitionBase, TObjPartitionDataTag, TObjStandardPartition,
    partition_copy_data, partition_set_name,
};
pub use persistence::{TObjPersistence, TObjPersistenceFactory, TObjPersistenceMap};
pub use reference_iterator::TObjReferenceIterator;
pub use sequence_iterator::TObjSequenceIterator;
pub use t_int_sparse_array::TObjTIntSparseArray;
pub use t_model::TObjTModel;
pub use t_name_container::{TObjNameMap, TObjTNameContainer};
pub use t_object::TObjTObject;
pub use t_reference::TObjTReference;
pub use t_xyz::TObjTXyz;

/// `TObj_DeletingMode`: how `Detach` treats the objects that depend on the deleted one.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TObjDeletingMode {
    /// `TObj_FreeOnly`: delete objects only without dependence.
    #[default]
    FreeOnly = 0,
    /// `TObj_KeepDepending`: remove the object if the depending ones can drop the
    /// reference.
    KeepDepending,
    /// `TObj_Forced`: delete this object and all depending objects.
    Forced,
}
