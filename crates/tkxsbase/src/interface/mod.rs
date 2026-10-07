//! `Interface`: the entity model of the exchange formats (models, protocols, generic
//! libraries, checks, graphs, copies, file reading tools and static parameters);
//! docs/xsbase-design.md section 1.
mod bit_map;
mod check;
mod check_edit;
#[cfg(test)]
mod check_tests;
mod enums;
mod float_writer;
mod int_array;
mod int_list;
mod int_val;
mod line_buffer;
mod msg;
mod msg_text;

pub use bit_map::InterfaceBitMap;
pub use check::InterfaceCheck;
pub use enums::{InterfaceCheckStatus, InterfaceDataState, InterfaceParamType};
pub use float_writer::{InterfaceFloatWriter, InterfaceFloatWriterOptions};
pub use int_array::InterfaceIntArray;
pub use int_list::InterfaceIntList;
pub use int_val::InterfaceIntVal;
pub use line_buffer::InterfaceLineBuffer;
pub use msg::InterfaceMsg;
