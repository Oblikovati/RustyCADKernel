// Created on: 2003-03-04
// Created by: Pavel TELKOV
// Copyright (c) 2003-2014 OPEN CASCADE SAS
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

//! `Message_ExecStatus`: four 32-bit flag sets, one per status group.
use super::{Status, StatusType};

const MASK_TYPE: u32 = 0x0000_ff00;
const MASK_INDEX: u32 = 0x0000_00ff;

/// Bit of a status inside its group word.
///
/// Native `0x1 << (theStatus & MIndex)` is undefined for indices above 31; x86 masks the
/// shift count to five bits, which `wrapping_shl` reproduces for such out-of-enum values.
fn bit_flag(status: Status) -> i32 {
    1_i32.wrapping_shl((status.0 as u32) & MASK_INDEX)
}

/// Done/warn/alarm/fail flag sets of an algorithm; `ExecStatus::from_status(Status::FAIL2)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExecStatus {
    done: i32,
    warn: i32,
    alarm: i32,
    fail: i32,
}
impl ExecStatus {
    /// `FirstStatus`: the first one-based status index.
    pub const FIRST_STATUS: i32 = 1;
    /// `StatusesPerType`: flags per status group.
    pub const STATUSES_PER_TYPE: i32 = 32;
    /// `NbStatuses`: the number of status flags.
    pub const NB_STATUSES: i32 = 128;
    /// `LastStatus`: native value 129, one past the last valid index 128.
    pub const LAST_STATUS: i32 = 129;

    /// Creates an empty set; `ExecStatus::new().is_done() == false`.
    pub const fn new() -> Self {
        Self {
            done: 0,
            warn: 0,
            alarm: 0,
            fail: 0,
        }
    }
    /// Creates a set holding one flag; `ExecStatus::from_status(Status::DONE1).is_done()`.
    pub fn from_status(status: Status) -> Self {
        let mut result = Self::new();
        result.set(status);
        result
    }
    fn word_mut(&mut self, status: Status) -> Option<&mut i32> {
        match Self::type_of_status(status) {
            StatusType::DONE => Some(&mut self.done),
            StatusType::WARN => Some(&mut self.warn),
            StatusType::ALARM => Some(&mut self.alarm),
            StatusType::FAIL => Some(&mut self.fail),
            _ => None,
        }
    }
    fn word(&self, status: Status) -> Option<i32> {
        match Self::type_of_status(status) {
            StatusType::DONE => Some(self.done),
            StatusType::WARN => Some(self.warn),
            StatusType::ALARM => Some(self.alarm),
            StatusType::FAIL => Some(self.fail),
            _ => None,
        }
    }
    /// Raises one flag; statuses outside the four groups are ignored; `status.set(Status::WARN1)`.
    pub fn set(&mut self, status: Status) {
        if let Some(word) = self.word_mut(status) {
            *word |= bit_flag(status);
        }
    }
    /// Tests one flag; `status.is_set(Status::WARN1)`.
    pub fn is_set(&self, status: Status) -> bool {
        self.word(status)
            .is_some_and(|word| word & bit_flag(status) != 0)
    }
    /// Lowers one flag; `status.clear_status(Status::WARN1)`.
    pub fn clear_status(&mut self, status: Status) {
        if let Some(word) = self.word_mut(status) {
            *word &= !bit_flag(status);
        }
    }
    /// Any done flag; `status.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done != 0
    }
    /// Any fail flag; `status.is_fail()`.
    pub fn is_fail(&self) -> bool {
        self.fail != 0
    }
    /// Any warn flag; `status.is_warn()`.
    pub fn is_warn(&self) -> bool {
        self.warn != 0
    }
    /// Any alarm flag; `status.is_alarm()`.
    pub fn is_alarm(&self) -> bool {
        self.alarm != 0
    }
    /// Raises every done flag; `status.set_all_done()`.
    pub fn set_all_done(&mut self) {
        self.done = !0;
    }
    /// Raises every warn flag; `status.set_all_warn()`.
    pub fn set_all_warn(&mut self) {
        self.warn = !0;
    }
    /// Raises every alarm flag; `status.set_all_alarm()`.
    pub fn set_all_alarm(&mut self) {
        self.alarm = !0;
    }
    /// Raises every fail flag; `status.set_all_fail()`.
    pub fn set_all_fail(&mut self) {
        self.fail = !0;
    }
    /// Lowers every done flag; `status.clear_all_done()`.
    pub fn clear_all_done(&mut self) {
        self.done = 0;
    }
    /// Lowers every warn flag; `status.clear_all_warn()`.
    pub fn clear_all_warn(&mut self) {
        self.warn = 0;
    }
    /// Lowers every alarm flag; `status.clear_all_alarm()`.
    pub fn clear_all_alarm(&mut self) {
        self.alarm = 0;
    }
    /// Lowers every fail flag; `status.clear_all_fail()`.
    pub fn clear_all_fail(&mut self) {
        self.fail = 0;
    }
    /// Lowers all flags (`Clear()`); `status.clear()`.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
    /// Unites another set into this one; `status.add(&other)`.
    pub fn add(&mut self, other: &Self) {
        self.done |= other.done;
        self.warn |= other.warn;
        self.alarm |= other.alarm;
        self.fail |= other.fail;
    }
    /// Intersects this set with another; `status.and(&other)`.
    pub fn and(&mut self, other: &Self) {
        self.done &= other.done;
        self.warn &= other.warn;
        self.alarm &= other.alarm;
        self.fail &= other.fail;
    }
}
impl std::ops::BitOrAssign<&ExecStatus> for ExecStatus {
    fn bitor_assign(&mut self, other: &ExecStatus) {
        self.add(other);
    }
}
impl std::ops::BitAndAssign<&ExecStatus> for ExecStatus {
    fn bitand_assign(&mut self, other: &ExecStatus) {
        self.and(other);
    }
}
impl ExecStatus {
    /// One-based index across all groups, 0 outside them; `ExecStatus::status_index(Status::WARN1) == 33`.
    pub fn status_index(status: Status) -> i32 {
        let group = match Self::type_of_status(status) {
            StatusType::DONE => 0,
            StatusType::WARN => 1,
            StatusType::ALARM => 2,
            StatusType::FAIL => 3,
            _ => return 0,
        };
        group * Self::STATUSES_PER_TYPE + Self::local_status_index(status)
    }
    /// One-based index inside the group; `ExecStatus::local_status_index(Status::WARN3) == 3`.
    pub fn local_status_index(status: Status) -> i32 {
        ((status.0 as u32 & MASK_INDEX) + 1) as i32
    }
    /// Group bits of a status; `ExecStatus::type_of_status(Status::FAIL1) == StatusType::FAIL`.
    pub fn type_of_status(status: Status) -> StatusType {
        StatusType(status.0 as u32 & MASK_TYPE)
    }
    /// Status of a one-based index; `ExecStatus::status_by_index(33) == Status::WARN1`.
    ///
    /// Native quirk kept: index 0 (and below) yields `Message_DONE + (index - 1)`, a value
    /// outside the enumerators, instead of `Message_None`.
    pub fn status_by_index(index: i32) -> Status {
        let offset = index.wrapping_sub(1);
        match offset {
            ..32 => Status(Status::DONE1.0.wrapping_add(offset)),
            32..64 => Status(Status::WARN1.0 + (offset - 32)),
            64..96 => Status(Status::ALARM1.0 + (offset - 64)),
            96..128 => Status(Status::FAIL1.0 + (offset - 96)),
            _ => Status::NONE,
        }
    }
}
