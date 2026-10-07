//! `MoniTool`: typed values with their checks, attribute lists, elements that map any
//! object as a key, signatures as text, and named timers.
//!
//! `MoniTool_Macros.hxx` (`GetCasted`, `DeclareAndCast`, `FastCast`) has no Rust
//! counterpart: a down cast is `handle.cast::<T>()`.
mod attr_list;
mod element;
mod int_val;
mod real_val;
mod sign_text;
mod timer;
mod timer_sentry;
mod typed_value;
mod typed_value_def;
mod typed_value_lib;
mod typed_value_print;
mod typed_value_set;
#[cfg(test)]
mod typed_value_tests;
mod value_type;

pub use attr_list::{MoniToolAttrList, MoniToolAttrMap};
pub use element::{MoniToolElement, MoniToolElementBase};
pub use int_val::MoniToolIntVal;
pub use real_val::MoniToolRealVal;
pub use sign_text::{MoniToolSignText, sign_text_alone};
pub use timer::{MoniToolTimer, MoniToolTimerAmendments};
pub use timer_sentry::MoniToolTimerSentry;
pub use typed_value::{MoniToolTypedValue, MoniToolValueInterpret, MoniToolValueSatisfies};
pub use typed_value_lib::MoniToolTypedValueMap;
pub use value_type::MoniToolValueType;
