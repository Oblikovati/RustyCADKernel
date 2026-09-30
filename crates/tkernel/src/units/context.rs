//! The `Units` package functions. Their native process-wide statics (dictionary,
//! lexicons, last-unit cache, `FirstQuantity` and `Units_Dimensions::Quantity`
//! caches) live in an explicit context, so independent contexts do not interact.
use super::diagnostics::{ConsoleDiagnostics, ConsoleSink, UnitsDiagnostics};
use super::dictionary::UnitsDictionary;
use super::dimensions::Dimensions;
use super::lexicon::{Lexicon, UnitsLexicon};
use super::quantity::Quantity;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard, PoisonError, RwLock};

mod conversion;
mod queries;

/// The native `lastunit`/`lastvalue`/`lastmove`/`lastdimension` statics of `ToSI`/`FromSI`.
#[derive(Default)]
struct LastUnit {
    unit: AsciiString,
    value: f64,
    movement: f64,
    dimensions: Handle<Dimensions>,
}

/// Every cache of the package, native initial values included (empty strings).
#[derive(Default)]
struct Caches {
    units_file: AsciiString,
    lexicon_file: AsciiString,
    last: LastUnit,
    first_symbol: AsciiString,
    first_quantity: AsciiString,
    dimension_quantities: Handle<HSequence<Handle<Quantity>>>,
}

/// A units context replacing the package statics; `Units::new()`.
pub struct Units {
    diagnostics: Box<dyn UnitsDiagnostics>,
    dictionary: RwLock<Handle<UnitsDictionary>>,
    lexicon_units: RwLock<Handle<Lexicon>>,
    lexicon_formula: RwLock<Handle<Lexicon>>,
    null_dimensions: RwLock<Handle<Dimensions>>,
    caches: Mutex<Caches>,
}

impl Default for Units {
    fn default() -> Self {
        Self::new()
    }
}

impl Units {
    /// A fresh context writing native console text to the process stdout;
    /// `Units::new()`.
    pub fn new() -> Self {
        Self::with_diagnostics(Box::new(ConsoleDiagnostics))
    }
    /// A fresh context reporting through `diagnostics`;
    /// `Units::with_diagnostics(Box::new(ConsoleDiagnostics))`.
    pub fn with_diagnostics(diagnostics: Box<dyn UnitsDiagnostics>) -> Self {
        Self {
            diagnostics,
            dictionary: RwLock::default(),
            lexicon_units: RwLock::default(),
            lexicon_formula: RwLock::default(),
            null_dimensions: RwLock::default(),
            caches: Mutex::default(),
        }
    }
    fn caches(&self) -> MutexGuard<'_, Caches> {
        self.caches.lock().unwrap_or_else(PoisonError::into_inner)
    }
    /// Text the native code writes to `std::cout`.
    pub(crate) fn print(&self, text: &[u8]) {
        self.diagnostics.print(text);
    }
    /// A `Message::SendWarning` payload.
    pub(crate) fn warning(&self, text: &[u8]) {
        self.diagnostics.warning(text);
    }
    /// A `Message::SendInfo` payload.
    pub(crate) fn info(&self, text: &[u8]) {
        self.diagnostics.info(text);
    }
    /// A writer forwarding to the console channel.
    pub(crate) fn console(&self) -> ConsoleSink<'_> {
        ConsoleSink::new(self.diagnostics.as_ref())
    }
    /// Records the units file location, which nothing reads (the dictionary comes
    /// from the compiled resource); `units.set_units_file(b"/path/Units.dat")`.
    pub fn set_units_file(&self, file: &[u8]) {
        self.caches().units_file = AsciiString::from_bytes(file);
    }
    /// Records the lexicon file location, which nothing reads;
    /// `units.set_lexicon_file(b"/path/Lexi")`.
    pub fn set_lexicon_file(&self, file: &[u8]) {
        self.caches().lexicon_file = AsciiString::from_bytes(file);
    }
    /// The context dictionary, created on first use; `mode` recreates an existing
    /// one, which re-adds every symbol to the units lexicon (so repeated meanings
    /// print and accumulate, as natively); `units.dictionary_of_units(false)?`.
    pub fn dictionary_of_units(&self, mode: bool) -> Result<Handle<UnitsDictionary>, Failure> {
        let existing = self
            .dictionary
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(dictionary) = existing.get() {
            if mode {
                dictionary.creates(self)?;
            }
            return Ok(existing);
        }
        let created = Handle::new(UnitsDictionary::new());
        *self
            .dictionary
            .write()
            .unwrap_or_else(PoisonError::into_inner) = created.clone();
        created.get().unwrap().creates(self)?;
        Ok(created)
    }
    /// The units lexicon, created on first use (`mode` also builds the dictionary);
    /// `units.lexicon_units(true)?`.
    pub fn lexicon_units(&self, mode: bool) -> Result<Handle<Lexicon>, Failure> {
        let existing = self
            .lexicon_units
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if existing.is_some() {
            return Ok(existing);
        }
        let created = Handle::new(UnitsLexicon::new());
        let lexicon: Handle<Lexicon> = created.cast();
        *self
            .lexicon_units
            .write()
            .unwrap_or_else(PoisonError::into_inner) = lexicon.clone();
        created.get().unwrap().creates(mode, self)?;
        Ok(lexicon)
    }
    /// The formula lexicon, created on first use; `units.lexicon_formula()?`.
    pub fn lexicon_formula(&self) -> Result<Handle<Lexicon>, Failure> {
        let existing = self
            .lexicon_formula
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if existing.is_some() {
            return Ok(existing);
        }
        let created = Handle::new(Lexicon::new());
        *self
            .lexicon_formula
            .write()
            .unwrap_or_else(PoisonError::into_inner) = created.clone();
        created.get().unwrap().creates()?;
        Ok(created)
    }
    /// The context's shared dimensionless object; `units.null_dimensions()`.
    pub fn null_dimensions(&self) -> Handle<Dimensions> {
        let mut cell = self
            .null_dimensions
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        if cell.is_null() {
            *cell = Dimensions::handle([0.0; 9]);
        }
        cell.clone()
    }
}
