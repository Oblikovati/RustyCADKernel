// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Members that change the shared string, or build a new shared string from it.
//!
//! `const char*` parameters are C strings: the bytes through the first NUL.
use super::HAsciiString;
use crate::standard::{Failure, Handle};
use crate::tcollection::ascii_string::{c_string_prefix, negative_width};

impl HAsciiString {
    /// `AssignCat(const char*)`; `object.assign_cat("tail")`.
    pub fn assign_cat(&self, other: impl AsRef<[u8]>) {
        self.edit().assign_cat(c_string_prefix(other.as_ref()));
    }
    /// `AssignCat(const occ::handle<TCollection_HAsciiString>&)`; `object.assign_cat_string(&other)`.
    pub fn assign_cat_string(&self, other: &HAsciiString) {
        let other = other.snapshot();
        self.edit().append(&other);
    }
    /// `Cat(const char*)`: a new object; `object.cat("tail").length()`.
    pub fn cat(&self, other: impl AsRef<[u8]>) -> Handle<Self> {
        Self::shared(self.string().cat(c_string_prefix(other.as_ref())))
    }
    /// `Cat(const occ::handle<TCollection_HAsciiString>&)`; `object.cat_string(&other)`.
    pub fn cat_string(&self, other: &HAsciiString) -> Handle<Self> {
        let other = other.snapshot();
        Self::shared(self.string().cat(&other))
    }
    /// `Capitalize`; `object.capitalize()`.
    pub fn capitalize(&self) {
        self.edit().capitalize();
    }
    /// `UpperCase`; `object.upper_case()`.
    pub fn upper_case(&self) {
        self.edit().upper_case();
    }
    /// `LowerCase`; `object.lower_case()`.
    pub fn lower_case(&self) {
        self.edit().lower_case();
    }
    /// `Center`: `Standard_NegativeValue` for a negative width; `object.center(9, b'*')?`.
    pub fn center(&self, width: i32, filler: u8) -> Result<(), Failure> {
        check_width("Center", width)?;
        self.edit().center(width, filler)
    }
    /// `LeftJustify`; `object.left_justify(9, b' ')?`.
    pub fn left_justify(&self, width: i32, filler: u8) -> Result<(), Failure> {
        check_width("LeftJustify", width)?;
        self.edit().left_justify(width, filler)
    }
    /// `RightJustify`; `object.right_justify(9, b' ')?`.
    pub fn right_justify(&self, width: i32, filler: u8) -> Result<(), Failure> {
        check_width("RightJustify", width)?;
        self.edit().right_justify(width, filler)
    }
    /// `ChangeAll`; `object.change_all(b'a', b'o', true)`.
    pub fn change_all(&self, from: u8, to: u8, case_sensitive: bool) {
        self.edit().change_all(from, to, case_sensitive);
    }
    /// `Clear`; `object.clear()`.
    pub fn clear(&self) {
        self.edit().clear();
    }
    /// `LeftAdjust`; `object.left_adjust()`.
    pub fn left_adjust(&self) {
        self.edit().left_adjust();
    }
    /// `RightAdjust`; `object.right_adjust()`.
    pub fn right_adjust(&self) {
        self.edit().right_adjust();
    }
}

impl HAsciiString {
    /// `Insert(int, char)`; `object.insert_char(1, b'>')?`.
    pub fn insert_char(&self, position: i32, byte: u8) -> Result<(), Failure> {
        self.edit().insert_char(position, byte)
    }
    /// `Insert(int, const char*)`; `object.insert(1, "head")?`.
    pub fn insert(&self, position: i32, what: impl AsRef<[u8]>) -> Result<(), Failure> {
        self.edit().insert(position, c_string_prefix(what.as_ref()))
    }
    /// `Insert(int, const occ::handle<TCollection_HAsciiString>&)`; `object.insert_string(1, &other)?`.
    pub fn insert_string(&self, position: i32, what: &HAsciiString) -> Result<(), Failure> {
        let what = what.snapshot();
        self.edit().insert(position, &what)
    }
    /// `InsertAfter`: index in `0..=Length`; `object.insert_after(0, &other)?`.
    pub fn insert_after(&self, index: i32, other: &HAsciiString) -> Result<(), Failure> {
        let other = other.snapshot();
        self.edit().insert_after(index, &other)
    }
    /// `InsertBefore`: index in `1..=Length`; `object.insert_before(1, &other)?`.
    pub fn insert_before(&self, index: i32, other: &HAsciiString) -> Result<(), Failure> {
        let other = other.snapshot();
        self.edit().insert_before(index, &other)
    }
    /// `Prepend`; `object.prepend(&other)`.
    pub fn prepend(&self, other: &HAsciiString) {
        let other = other.snapshot();
        self.edit().prepend(&other);
    }
    /// `RemoveAll(char, bool)`; `object.remove_all_with_case(b'A', false)`.
    pub fn remove_all_with_case(&self, byte: u8, case_sensitive: bool) {
        self.edit().remove_all_with_case(byte, case_sensitive);
    }
    /// `RemoveAll(char)`; `object.remove_all(b' ')`.
    pub fn remove_all(&self, byte: u8) {
        self.edit().remove_all(byte);
    }
    /// `Remove`; `object.remove(2, 1)?`.
    pub fn remove(&self, position: i32, count: i32) -> Result<(), Failure> {
        self.edit().remove(position, count)
    }
    /// `SetValue(int, char)`; `object.set_value_char(1, b'x')?`.
    pub fn set_value_char(&self, position: i32, byte: u8) -> Result<(), Failure> {
        self.edit().set_value_char(position, byte)
    }
    /// `SetValue(int, const char*)`; `object.set_value(2, "yz")?`.
    pub fn set_value(&self, position: i32, what: impl AsRef<[u8]>) -> Result<(), Failure> {
        self.edit()
            .set_value(position, c_string_prefix(what.as_ref()))
    }
    /// `SetValue(int, const occ::handle<TCollection_HAsciiString>&)`; `object.set_value_string(2, &other)?`.
    pub fn set_value_string(&self, position: i32, what: &HAsciiString) -> Result<(), Failure> {
        let what = what.snapshot();
        self.edit().set_value(position, &what)
    }
    /// `Split`: keeps the head and returns the tail as a new object; `object.split(3)?`.
    pub fn split(&self, length: i32) -> Result<Handle<Self>, Failure> {
        let tail = self.edit().split(length)?;
        Ok(Self::shared(tail))
    }
    /// `SubString`: a new object; `object.sub_string(2, 4)?`.
    pub fn sub_string(&self, from: i32, to: i32) -> Result<Handle<Self>, Failure> {
        Ok(Self::shared(self.string().sub_string(from, to)?))
    }
    /// `Token`: a new object; `object.token(" \t", 2)`.
    pub fn token(&self, separators: impl AsRef<[u8]>, which: i32) -> Handle<Self> {
        Self::shared(self.string().token(separators, which))
    }
    /// `Trunc`; `object.truncate(3)?`.
    pub fn truncate(&self, length: i32) -> Result<(), Failure> {
        self.edit().truncate(length)
    }
}

/// The width check `TCollection_HAsciiString` makes before delegating.
fn check_width(member: &str, width: i32) -> Result<(), Failure> {
    if width < 0 {
        return Err(negative_width(member, width));
    }
    Ok(())
}
