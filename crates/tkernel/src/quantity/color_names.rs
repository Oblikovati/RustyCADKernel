// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Pinned OCCT color names with their sRGB and linear RGB table values, copied without
//! recomputation by `tools/generate_quantity_color_table.py`.
use crate::tcollection::AsciiString;

/// One `Quantity_StandardColor` row: name, sRGB values and linear RGB values.
type ColorEntry = (&'static str, [f32; 3], [f32; 3]);
/// Number of canonical enumerators, `Quantity_NOC_BLACK..=Quantity_NOC_WHITE`.
const COLOR_COUNT: usize = 509;

/// `Quantity_NameOfColor`: a predefined OCCT color, including the enum aliases such as
/// `BLUE1`; `NamedColor::RED.name() == "RED"`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct NamedColor(usize);
impl NamedColor {
    /// Returns the canonical table spelling (`Quantity_Color::StringName`);
    /// `NamedColor::RED.name() == "RED"`.
    pub fn name(self) -> &'static str {
        self.entry().0
    }
    /// Returns the table's linear RGB floats; `NamedColor::RED.rgb() == [1.0,0.0,0.0]`.
    pub fn rgb(self) -> [f32; 3] {
        self.entry().2
    }
    /// Returns the table's sRGB floats, used by the nearest-name search;
    /// `NamedColor::MATRABLUE.srgb()[2] == 0.75`.
    pub fn srgb(self) -> [f32; 3] {
        self.entry().1
    }
    /// Returns the enumerator's integer value; `NamedColor::WHITE.index() == 508`.
    pub const fn index(self) -> i32 {
        self.0 as i32
    }
    /// Converts an integer enumerator value, `None` outside `BLACK..=WHITE` where the native
    /// `valuesOf` raises "Bad name"; `NamedColor::from_index(0) == Some(NamedColor::BLACK)`.
    pub fn from_index(index: i32) -> Option<Self> {
        usize::try_from(index)
            .ok()
            .filter(|value| *value < COLOR_COUNT)
            .map(Self)
    }
    /// Every canonical enumerator in declaration order; `NamedColor::all().count() == 509`.
    pub fn all() -> impl Iterator<Item = Self> {
        (0..COLOR_COUNT).map(Self)
    }
    /// `Quantity_Color::ColorFromName(const char*, Quantity_NameOfColor&)`: case-insensitive,
    /// optional `Quantity_NOC_` prefix, then the enum aliases; `NamedColor::from_name("red")`.
    pub fn from_name(text: &str) -> Option<Self> {
        let key = normalized_name(text);
        let key = key.as_bytes();
        Self::all()
            .find(|color| color.name().as_bytes() == key)
            .or_else(|| {
                ALIASES
                    .iter()
                    .find(|(alias, _)| alias.as_bytes() == key)
                    .map(|(_, color)| *color)
            })
    }
    fn entry(self) -> ColorEntry {
        let table: &[ColorEntry] = match self.0 / 60 {
            0 => &TABLE_0,
            1 => &TABLE_1,
            2 => &TABLE_2,
            3 => &TABLE_3,
            4 => &TABLE_4,
            5 => &TABLE_5,
            6 => &TABLE_6,
            7 => &TABLE_7,
            8 => &TABLE_8,
            _ => unreachable!("NamedColor uses only valid table indices"),
        };
        table[self.0 % 60]
    }
}
/// Native `TCollection_AsciiString(theName)` stops at the first NUL, `UpperCase` is C-locale
/// `toupper`, and a leading `QUANTITY_NOC_` (Search(...) == 1) is removed. For the bare
/// prefix the native `SubString(14, 13)` throws inside the `noexcept` ColorFromName, which
/// terminates OCCT; here the empty remainder simply matches no name.
fn normalized_name(text: &str) -> AsciiString {
    let mut name = AsciiString::new(text.split('\0').next().unwrap_or_default());
    name.upper_case();
    let bytes = name.as_bytes();
    if bytes.starts_with(b"QUANTITY_NOC_") {
        return AsciiString::from_bytes(&bytes[13..]);
    }
    name
}
include!("color_names/aliases.rs");
include!("color_names/constants_0.rs");
include!("color_names/constants_1.rs");
include!("color_names/constants_2.rs");
include!("color_names/constants_3.rs");
include!("color_names/constants_4.rs");
include!("color_names/constants_5.rs");
include!("color_names/table_0.rs");
include!("color_names/table_1.rs");
include!("color_names/table_2.rs");
include!("color_names/table_3.rs");
include!("color_names/table_4.rs");
include!("color_names/table_5.rs");
include!("color_names/table_6.rs");
include!("color_names/table_7.rs");
include!("color_names/table_8.rs");
