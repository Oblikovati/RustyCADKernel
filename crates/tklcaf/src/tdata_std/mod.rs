//! The TDataStd package: the standard attributes of OCAF (values, strings, arrays, lists,
//! maps, tree nodes, expressions).
mod array_attribute;
mod array_core;
mod array_delta;
mod arrays;
mod ascii_string;
mod boolean_array;
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
pub(crate) mod support;
mod u_attribute;
pub use array_delta::{
    TDataStdDeltaOnModificationOfByteArray, TDataStdDeltaOnModificationOfExtStringArray,
    TDataStdDeltaOnModificationOfIntArray, TDataStdDeltaOnModificationOfRealArray,
};
pub use arrays::{
    TDataStdByteArray, TDataStdExtStringArray, TDataStdIntegerArray, TDataStdRealArray,
};
pub use ascii_string::TDataStdAsciiString;
pub use boolean_array::TDataStdBooleanArray;
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
pub use u_attribute::{TDataStdCurrent, TDataStdUAttribute};

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
