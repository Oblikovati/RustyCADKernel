//! OCCT `Units` package: dimensions, tokens, units, quantities, lexicons, the
//! dictionary of units, unit sentences, measurements and units systems.
//!
//! The package statics become a [`Units`] context; every class that needs them
//! takes the context explicitly. Exceptions are `Failure`s carrying the native type
//! names (`Units_NoSuchType`, `Units_NoSuchUnit`, `Standard_OutOfRange`, ...).
mod context;
mod diagnostics;
mod dictionary;
mod dimensions;
mod explorer;
mod lexicon;
mod measurement;
pub mod operators;
mod quantity;
mod sentence;
mod support;
mod token;
mod unit;
mod unit_sentence;
mod units_system;

pub use context::Units;
pub use diagnostics::{ConsoleDiagnostics, ConsoleSink, UnitsDiagnostics};
pub use dictionary::UnitsDictionary;
pub use dimensions::{Dimensions, Exponents};
pub use explorer::Explorer;
pub use lexicon::{Lexicon, LexiconHandle, UnitsLexicon};
pub use measurement::Measurement;
pub use quantity::Quantity;
pub use sentence::Sentence;
pub use token::{ShiftedToken, Token, TokenHandle};
pub use unit::{ShiftedUnit, Unit, UnitHandle};
pub use unit_sentence::{MathSentence, UnitSentence};
pub use units_system::UnitsSystem;

crate::standard::exceptions::define_standard_exceptions! {
    UNITS_HIERARCHY;
    no_such_type => "Units_NoSuchType" : "Standard_NoSuchObject";
    no_such_unit => "Units_NoSuchUnit" : "Standard_NoSuchObject";
}
