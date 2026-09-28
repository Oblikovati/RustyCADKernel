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

//! Pinned OCCT color names and linear RGB table, copied without recomputation.
/// A predefined OCCT color, including enum aliases; `NamedColor::RED.name() == "RED"`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedColor(usize);
impl NamedColor {
    /// Returns the canonical table spelling; `NamedColor::RED.name() == "RED"`.
    pub fn name(self) -> &'static str {
        self.entry().0
    }
    /// Returns the table's linear RGB floats; `NamedColor::RED.rgb() == [1.0,0.0,0.0]`.
    pub fn rgb(self) -> [f32; 3] {
        self.entry().1
    }
    fn entry(self) -> (&'static str, [f32; 3]) {
        match self.0 / 60 {
            0 => TABLE_0[self.0 % 60],
            1 => TABLE_1[self.0 % 60],
            2 => TABLE_2[self.0 % 60],
            3 => TABLE_3[self.0 % 60],
            4 => TABLE_4[self.0 % 60],
            5 => TABLE_5[self.0 % 60],
            6 => TABLE_6[self.0 % 60],
            7 => TABLE_7[self.0 % 60],
            8 => TABLE_8[self.0 % 60],
            _ => unreachable!("NamedColor uses only valid table indices"),
        }
    }
}
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
