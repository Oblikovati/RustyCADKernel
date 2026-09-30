//! `TCollection_HAsciiString`: an [`AsciiString`] inside a reference-counted transient.
//!
//! OCCT edits the string through any handle to it, so the value sits behind a lock and
//! every member takes `&self`. A member that reads another string takes a copy of it
//! before locking this one, which keeps `text.assign_cat_string(&text)` (the same object
//! on both sides, legal in C++) free of deadlock. Handle parameters that OCCT dereferences
//! without a check are `&HAsciiString` (a `&Handle` coerces to it); the three members that
//! raise `Standard_NullObject` for a null handle take `&Handle<HAsciiString>`.
use super::{AsciiString, HExtendedString};
use crate::standard::{Handle, StandardTransient};
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A shared, mutable string object; `Handle::new(HAsciiString::new("text"))`.
#[repr(C)]
#[derive(Default)]
pub struct HAsciiString {
    parent: StandardTransient,
    value: RwLock<AsciiString>,
}
crate::impl_transient!(
    HAsciiString,
    "TCollection_HAsciiString",
    StandardTransient,
    parent
);
impl std::fmt::Debug for HAsciiString {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("HAsciiString")
            .field(&*self.string())
            .finish()
    }
}
impl From<AsciiString> for HAsciiString {
    /// `TCollection_HAsciiString(TCollection_AsciiString&&)`: takes the string's storage.
    fn from(value: AsciiString) -> Self {
        Self {
            parent: StandardTransient::default(),
            value: RwLock::new(value),
        }
    }
}
impl HAsciiString {
    /// `TCollection_HAsciiString(const char*)`: copies through the first NUL;
    /// `HAsciiString::new("abc").length() == 3`.
    pub fn new(text: impl AsRef<[u8]>) -> Self {
        AsciiString::from_bytes(text).into()
    }
    /// `TCollection_HAsciiString(char)`; `HAsciiString::from_char(b'A')`.
    pub fn from_char(byte: u8) -> Self {
        AsciiString::from_char(byte).into()
    }
    /// `TCollection_HAsciiString(int length, char filler)`; `HAsciiString::filled(3, b'*')`.
    pub fn filled(length: usize, filler: u8) -> Self {
        AsciiString::filled(length, filler).into()
    }
    /// `TCollection_HAsciiString(int)`: `%d`; `HAsciiString::from_integer(-7)`.
    pub fn from_integer(value: i32) -> Self {
        AsciiString::from_integer(value).into()
    }
    /// `TCollection_HAsciiString(double)`: `%g`; `HAsciiString::from_real(0.5)`.
    pub fn from_real(value: f64) -> Self {
        AsciiString::from_real(value).into()
    }
    /// `TCollection_HAsciiString(const TCollection_AsciiString&)`; `HAsciiString::from_string(&text)`.
    pub fn from_string(text: &AsciiString) -> Self {
        text.clone().into()
    }
    /// `TCollection_HAsciiString(const occ::handle<TCollection_HAsciiString>&)`: copies the
    /// other object's string; `HAsciiString::copied(&handle)`.
    pub fn copied(other: &HAsciiString) -> Self {
        other.snapshot().into()
    }
    /// `TCollection_HAsciiString(const occ::handle<TCollection_HExtendedString>&, char)`: the
    /// other object's `String()` as UTF-8, or `replacement` for units above 0xFF when nonzero;
    /// `HAsciiString::from_h_extended_string(&wide, b'?')`.
    pub fn from_h_extended_string(source: &HExtendedString, replacement: u8) -> Self {
        AsciiString::from_extended(&source.string(), Some(replacement)).into()
    }
    /// `String`: borrows the string for as long as the guard lives; `object.string().len()`.
    pub fn string(&self) -> RwLockReadGuard<'_, AsciiString> {
        self.value.read().unwrap_or_else(PoisonError::into_inner)
    }
    /// `ToCString`: a copy of the bytes with their terminating NUL; `object.as_bytes_with_nul()`.
    pub fn as_bytes_with_nul(&self) -> Vec<u8> {
        self.string().as_bytes_with_nul().to_vec()
    }
    /// `Length`; `HAsciiString::new("abc").length() == 3`.
    pub fn length(&self) -> i32 {
        self.string().length()
    }
    /// `IsEmpty`; `HAsciiString::default().is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.string().is_empty()
    }
    fn snapshot(&self) -> AsciiString {
        self.string().clone()
    }
    fn edit(&self) -> RwLockWriteGuard<'_, AsciiString> {
        self.value.write().unwrap_or_else(PoisonError::into_inner)
    }
    fn shared(value: AsciiString) -> Handle<Self> {
        Handle::new(Self::from(value))
    }
}
mod edits;
mod queries;
