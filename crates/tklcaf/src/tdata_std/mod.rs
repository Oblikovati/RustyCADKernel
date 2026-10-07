//! The TDataStd package: the standard attributes of OCAF (values, strings, arrays, lists,
//! maps, tree nodes, expressions).
mod array_attribute;
mod array_core;
mod array_delta;
mod arrays;
mod ascii_string;
mod boolean_array;
mod child_node_iterator;
mod expression;
mod generic_empty;
mod generic_ext_string;
mod h_data_maps;
mod int_packed_map;
mod integer;
mod list_attribute;
mod list_core;
mod lists;
mod name;
mod named_data;
mod named_data_category;
mod package;
mod real;
mod reference_array;
pub mod support;
mod tree_node;
mod tree_node_attribute;
mod u_attribute;
mod variable;
pub use array_delta::{
    TDataStdDeltaOnModificationOfByteArray, TDataStdDeltaOnModificationOfExtStringArray,
    TDataStdDeltaOnModificationOfIntArray, TDataStdDeltaOnModificationOfRealArray,
};
pub use arrays::{
    TDataStdByteArray, TDataStdExtStringArray, TDataStdIntegerArray, TDataStdRealArray,
};
pub use ascii_string::TDataStdAsciiString;
pub use boolean_array::TDataStdBooleanArray;
pub use child_node_iterator::TDataStdChildNodeIterator;
pub use expression::{TDataStdExpression, TDataStdRelation};
pub use generic_empty::{
    TDataStdDirectory, TDataStdGenericEmpty, TDataStdNoteBook, TDataStdTick, dump_generic_empty,
};
pub use generic_ext_string::{
    TDataStdGenericExtString, TDataStdGenericExtStringBase, TDataStdGenericExtStringMembers,
};
pub use h_data_maps::{
    TDataStdHDataMapOfStringByte, TDataStdHDataMapOfStringHArray1OfInteger,
    TDataStdHDataMapOfStringHArray1OfReal, TDataStdHDataMapOfStringInteger,
    TDataStdHDataMapOfStringReal, TDataStdHDataMapOfStringString, TDataStdNamedMap,
};
pub use int_packed_map::{TDataStdDeltaOnModificationOfIntPackedMap, TDataStdIntPackedMap};
pub use integer::TDataStdInteger;
pub use lists::{
    TDataStdBooleanList, TDataStdExtStringList, TDataStdIntegerList, TDataStdRealList,
    TDataStdReferenceList,
};
pub use name::{TDataStdComment, TDataStdName};
pub use named_data::TDataStdNamedData;
pub use package::TDataStd;
pub use real::{TDataStdReal, TDataStdRealEnum};
pub use reference_array::TDataStdReferenceArray;
pub use tree_node::TDataStdTreeNode;
pub use u_attribute::{TDataStdCurrent, TDataStdUAttribute};
pub use variable::TDataStdVariable;

use rustycad_tkernel::standard::Guid;

/// The IDs `TDataStd::IDList` appends after the scalar attributes: IntegerArray, RealArray
/// and ExtStringArray.
fn array_ids() -> [Guid; 3] {
    [
        TDataStdIntegerArray::get_id(),
        TDataStdRealArray::get_id(),
        TDataStdExtStringArray::get_id(),
    ]
}

/// The `IMPLEMENT_DERIVED_ATTRIBUTE` registrations of TKLCAF, in the native file order
/// (the static initializers run when the library loads; `TdfDerivedAttribute` calls this
/// once before its first query).
pub(crate) fn register_derived_attributes() {
    use crate::tdf::{TdfAttribute, TdfDerivedAttribute};
    use rustycad_tkernel::standard::Handle;
    let creators: [fn() -> Handle<dyn TdfAttribute>; 6] = [
        || Handle::new(TDataStdComment::new()).cast(),
        || Handle::new(TDataStdDirectory::new()).cast(),
        || Handle::new(TDataStdName::new()).cast(),
        || Handle::new(TDataStdNoteBook::new()).cast(),
        || Handle::new(TDataStdRelation::new()).cast(),
        || Handle::new(TDataStdTick::new()).cast(),
    ];
    for create in creators {
        TdfDerivedAttribute::register(create, None, None);
    }
}
