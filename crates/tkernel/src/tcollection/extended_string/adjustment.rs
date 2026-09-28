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

use super::ExtendedString;
use super::errors::out_of_range;
use crate::standard::Failure;
impl ExtendedString {
    /// Removes leading space code units; `text.left_adjust()`.
    pub fn left_adjust(&mut self) {
        let first = self
            .as_utf16()
            .iter()
            .position(|&unit| unit != 32)
            .unwrap_or(self.len());
        self.terminated.drain(..first);
    }
    /// Removes trailing space code units; `text.right_adjust()`.
    pub fn right_adjust(&mut self) {
        let length = self
            .as_utf16()
            .iter()
            .rposition(|&unit| unit != 32)
            .map_or(0, |index| index + 1);
        self.terminated.truncate(length + 1);
        self.terminated[length] = 0;
    }
    /// Appends filler up to a minimum width; `text.left_justify(10,45)?`.
    pub fn left_justify(&mut self, width: i32, filler: u16) -> Result<(), Failure> {
        let extra = self.padding("LeftJustify", width)?;
        let end = self.len();
        self.terminated
            .splice(end..end, std::iter::repeat_n(filler, extra));
        Ok(())
    }
    /// Prepends filler up to a minimum width; `text.right_justify(10,45)?`.
    pub fn right_justify(&mut self, width: i32, filler: u16) -> Result<(), Failure> {
        let extra = self.padding("RightJustify", width)?;
        self.terminated
            .splice(0..0, std::iter::repeat_n(filler, extra));
        Ok(())
    }
    /// Centers with any extra odd filler on the right; `text.center(7,45)?`.
    pub fn center(&mut self, width: i32, filler: u16) -> Result<(), Failure> {
        let extra = self.padding("Center", width)?;
        let left = extra / 2;
        let right = extra - left;
        self.terminated
            .splice(0..0, std::iter::repeat_n(filler, left));
        let end = self.len();
        self.terminated
            .splice(end..end, std::iter::repeat_n(filler, right));
        Ok(())
    }
    /// Capitalizes ASCII letters without altering other code units; `text.capitalize()`.
    pub fn capitalize(&mut self) {
        for (index, unit) in self.terminated.iter_mut().enumerate() {
            if index == 0 && (97..=122).contains(unit) {
                *unit -= 32;
            }
            if index > 0 && (65..=90).contains(unit) {
                *unit += 32;
            }
        }
    }
    fn padding(&self, member: &str, width: i32) -> Result<usize, Failure> {
        if width < 0 {
            return Err(out_of_range(
                &format!("TCollection_ExtendedString::{member} : theWidth is negative"),
                format_args!("theWidth {width}; expected >= 0"),
            ));
        }
        Ok((width as usize).saturating_sub(self.len()))
    }
}
