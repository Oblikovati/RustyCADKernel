//! The HExtendedString members, each forwarding to the wrapped ExtendedString.
//!
//! OCCT dereferences the argument handle without a null check (undefined behavior for a
//! null handle), so the port takes the object itself: a `&Handle` argument derefs to it.
use super::HExtendedString;
use crate::standard::{Failure, Handle};
use std::io;
impl HExtendedString {
    /// Appends another object's string (itself included); `text.assign_cat(&other)`.
    pub fn assign_cat(&self, other: &Self) {
        let suffix = other.string().clone();
        self.string_mut().assign_cat(&suffix);
    }
    /// Returns a new object holding this string followed by `other`'s; `text.cat(&other)`.
    pub fn cat(&self, other: &Self) -> Handle<Self> {
        let suffix = other.string().clone();
        Self::share(self.string().cat(&suffix))
    }
    /// Replaces every occurrence of a code unit; `text.change_all(0x61, 0x62)`.
    pub fn change_all(&self, from: u16, to: u16) {
        self.string_mut().change_all(from, to);
    }
    /// Empties the string; `text.clear()`.
    pub fn clear(&self) {
        self.string_mut().clear();
    }
    /// Checks for no code units; `text.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.string().is_empty()
    }
    /// Inserts one code unit (zero included) at a one-based position; `text.insert_char(1, 87)?`.
    pub fn insert_char(&self, index: i32, unit: u16) -> Result<(), Failure> {
        self.string_mut().insert_char(index, unit)
    }
    /// Inserts another object's string at a one-based position; `text.insert(1, &other)?`.
    pub fn insert(&self, index: i32, what: &Self) -> Result<(), Failure> {
        let inserted = what.string().clone();
        self.string_mut().insert(index, &inserted)
    }
    /// Orders by unsigned code units; `text.is_less(&other)`.
    pub fn is_less(&self, other: &Self) -> bool {
        let other = other.string().clone();
        self.string().is_less(&other)
    }
    /// Orders by unsigned code units; `text.is_greater(&other)`.
    pub fn is_greater(&self, other: &Self) -> bool {
        let other = other.string().clone();
        self.string().is_greater(&other)
    }
    /// Whether every unit is in the eight-bit range; `text.is_ascii()`.
    pub fn is_ascii(&self) -> bool {
        self.string().is_ascii()
    }
    /// Returns the length in code units; `text.length()`.
    pub fn length(&self) -> i32 {
        self.string().length()
    }
    /// Erases `count` units from a one-based position; `text.remove(2, 2)?`.
    pub fn remove(&self, index: i32, count: i32) -> Result<(), Failure> {
        self.string_mut().remove(index, count)
    }
    /// Removes every occurrence of a code unit; `text.remove_all(32)`.
    pub fn remove_all(&self, unit: u16) {
        self.string_mut().remove_all(unit);
    }
    /// Replaces one code unit at a one-based position; `text.set_value(1, 65)?`.
    pub fn set_value(&self, index: i32, unit: u16) -> Result<(), Failure> {
        self.string_mut().set_value(index, unit)
    }
    /// Overwrites from a one-based position with another object's string, growing as
    /// needed; `text.set_value_string(2, &other)?`.
    pub fn set_value_string(&self, index: i32, what: &Self) -> Result<(), Failure> {
        let units = what.string().clone();
        self.string_mut().set_value_units(index, &units)
    }
    /// Keeps the first `index` units and returns the rest as a new object; `text.split(3)?`.
    pub fn split(&self, index: i32) -> Result<Handle<Self>, Failure> {
        self.string_mut().split(index).map(Self::share)
    }
    /// One-based start of the first occurrence of `what`'s string, or -1; `text.search(&what)`.
    pub fn search(&self, what: &Self) -> i32 {
        let needle = what.string().clone();
        self.string().search(&needle)
    }
    /// One-based start of the last occurrence of `what`'s string, or -1;
    /// `text.search_from_end(&what)`.
    pub fn search_from_end(&self, what: &Self) -> i32 {
        let needle = what.string().clone();
        self.string().search_from_end(&needle)
    }
    /// Copies the units with their terminating zero (`ToExtString`); the lock cannot lend
    /// a pointer past the call, so the port returns a copy; `text.to_ext_string()`.
    pub fn to_ext_string(&self) -> Vec<u16> {
        self.string().as_utf16_with_nul().to_vec()
    }
    /// Returns a new object holding the `which_one`-th token; `text.token(&[32], 2)`.
    pub fn token(&self, separators: &[u16], which_one: i32) -> Handle<Self> {
        Self::share(self.string().token(separators, which_one))
    }
    /// Keeps the first `length` units; `text.trunc(3)?`.
    pub fn trunc(&self, length: i32) -> Result<(), Failure> {
        self.string_mut().trunc(length)
    }
    /// Reads the code unit at a one-based position; `text.value(1)?`.
    pub fn value(&self, index: i32) -> Result<u16, Failure> {
        self.string().value(index)
    }
    /// Writes OCCT's header line, then the UTF-8 string; `text.print(&mut std::io::stdout())?`.
    pub fn print(&self, output: &mut impl io::Write) -> io::Result<()> {
        output.write_all(b"begin class HExtendedString \n")?;
        self.string().print(output)
    }
    /// Whether both objects hold equal strings (`IsSameState`); `text.is_same_state(&other)`.
    pub fn is_same_state(&self, other: &Self) -> bool {
        let other = other.string().clone();
        self.string().is_equal(&other)
    }
}
