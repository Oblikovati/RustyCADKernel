//! The TDataStd package: the standard attributes of OCAF (values, strings, arrays, lists,
//! maps, tree nodes, expressions).
mod ascii_string;
mod generic_empty;
mod generic_ext_string;
mod integer;
mod name;
mod package;
mod real;
mod support;
mod u_attribute;
pub use ascii_string::TDataStdAsciiString;
pub use generic_empty::{
    TDataStdDirectory, TDataStdGenericEmpty, TDataStdNoteBook, TDataStdTick, dump_generic_empty,
};
pub use generic_ext_string::{
    TDataStdGenericExtString, TDataStdGenericExtStringBase, TDataStdGenericExtStringMembers,
};
pub use integer::TDataStdInteger;
pub use name::{TDataStdComment, TDataStdName};
pub use package::TDataStd;
pub use real::{TDataStdReal, TDataStdRealEnum};
pub use u_attribute::{TDataStdCurrent, TDataStdUAttribute};

use crate::guid_from_text;
use rustycad_tkernel::standard::Guid;

/// The IDs `TDataStd::IDList` appends after the scalar attributes: IntegerArray, RealArray
/// and ExtStringArray.
fn array_ids() -> [Guid; 3] {
    [
        guid_from_text("2a96b61d-ec8b-11d0-bee7-080009dc3333"),
        guid_from_text("2a96b61e-ec8b-11d0-bee7-080009dc3333"),
        guid_from_text("2a96b624-ec8b-11d0-bee7-080009dc3333"),
    ]
}
