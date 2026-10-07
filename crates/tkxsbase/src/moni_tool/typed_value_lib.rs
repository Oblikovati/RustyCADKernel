// Rust port of DataExchange/TKXSBase/MoniTool/MoniTool_TypedValue.cxx (the library of
// typed values and the static values) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The process-wide library of typed values (`AddLib`, `Lib`, `FromLib`, `LibList`) and
//! the map of static values (`Stats`, `StaticValue`); xsbase-design.md section 2.
use super::{MoniToolTypedValue, MoniToolValueType};
use rustycad_tkernel::{
    ncollection::DataMap,
    standard::{Failure, Handle, StandardTransient},
    tcollection::AsciiString,
};
use std::sync::{OnceLock, PoisonError, RwLock, RwLockWriteGuard};

/// The native `NCollection_DataMap<TCollection_AsciiString, occ::handle<Standard_Transient>>`
/// of the library and of the static values.
pub type MoniToolTypedValueMap = DataMap<AsciiString, Handle<StandardTransient>>;

fn library_lock() -> &'static RwLock<MoniToolTypedValueMap> {
    static LIBRARY: OnceLock<RwLock<MoniToolTypedValueMap>> = OnceLock::new();
    LIBRARY.get_or_init(|| RwLock::new(MoniToolTypedValueMap::default()))
}

fn stats_lock() -> &'static RwLock<MoniToolTypedValueMap> {
    static STATS: OnceLock<RwLock<MoniToolTypedValueMap>> = OnceLock::new();
    STATS.get_or_init(|| RwLock::new(MoniToolTypedValueMap::default()))
}

/// The six standard types the native `libtv()` binds when the library is empty.
fn standard_types() -> Result<Vec<MoniToolTypedValue>, Failure> {
    let boolean = MoniToolTypedValue::new("Boolean", MoniToolValueType::ValueEnum, "")?;
    for def in ["enum 0", "eval False", "eval True"] {
        boolean.add_def(def)?;
    }
    let logical = MoniToolTypedValue::new("Logical", MoniToolValueType::ValueEnum, "")?;
    for def in ["enum -1", "eval False", "eval Unknown", "eval True"] {
        logical.add_def(def)?;
    }
    Ok(vec![
        MoniToolTypedValue::new("Integer", MoniToolValueType::ValueInteger, "")?,
        MoniToolTypedValue::new("Real", MoniToolValueType::ValueReal, "")?,
        MoniToolTypedValue::new("Text", MoniToolValueType::ValueText, "")?,
        MoniToolTypedValue::new("Transient", MoniToolValueType::ValueIdent, "")?,
        boolean,
        logical,
    ])
}

/// `libtv()`: the library, filled with the standard types when empty.
fn library() -> RwLockWriteGuard<'static, MoniToolTypedValueMap> {
    let mut lib = library_lock()
        .write()
        .unwrap_or_else(PoisonError::into_inner);
    if lib.is_empty() {
        // The standard types are valid specifications; an error cannot occur here.
        for tv in standard_types().unwrap_or_default() {
            lib.bind(tv.name(), Handle::new(tv).cast());
        }
    }
    lib
}

impl MoniToolTypedValue {
    /// `AddLib(tv, def = "")`: records `tv` under its name (replacing a former one),
    /// enforcing `def` as its definition when given; false for a null handle.
    pub fn add_lib(tv: &Handle<MoniToolTypedValue>, def: &str) -> bool {
        let Some(value) = tv.get() else { return false };
        if !def.is_empty() {
            value.set_definition(def);
        }
        library().bind(value.name(), tv.cast());
        true
    }
    /// `Lib(def)`: the recorded typed value (the original), null when none.
    pub fn lib(def: &str) -> Handle<MoniToolTypedValue> {
        let found = library().seek(&AsciiString::new(def)).cloned();
        found.map(|h| h.cast()).unwrap_or_default()
    }
    /// `FromLib(def)`: a copy of the recorded typed value, null when none.
    pub fn from_lib(def: &str) -> Handle<MoniToolTypedValue> {
        let original = Self::lib(def);
        match original.get() {
            Some(value) => Handle::new(Self::copied(value)),
            None => Handle::default(),
        }
    }
    /// `LibList`: the names of the library, in the native map order.
    pub fn lib_list() -> Vec<AsciiString> {
        library().items().map(|(name, _)| name.clone()).collect()
    }
    /// `Stats` (protected): the map of static values (a guard to drop before any other
    /// call that reads it).
    pub fn stats() -> RwLockWriteGuard<'static, MoniToolTypedValueMap> {
        stats_lock().write().unwrap_or_else(PoisonError::into_inner)
    }
    /// `StaticValue(name)`: the static value of a name, null when unknown.
    pub fn static_value(name: &str) -> Handle<MoniToolTypedValue> {
        let found = Self::stats().seek(&AsciiString::new(name)).cloned();
        found.map(|h| h.cast()).unwrap_or_default()
    }
}
