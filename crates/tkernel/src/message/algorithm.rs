// Created on: 2007-07-06
// Created by: Pavel TELKOV
// Copyright (c) 2007-2014 OPEN CASCADE SAS
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

//! `Message_Algorithm`: execution status with per-status data and message output.
use super::{ExecStatus, Message, Messenger, Msg, Status};
use crate::ncollection::PackedMap;
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient};
use crate::tcollection::{AsciiString, ExtendedString, HAsciiString};
use std::sync::{Arc, Mutex, OnceLock};
mod reporting;

/// One optional entry per status index `FirstStatus..=LastStatus` (index 0 unused).
type StatusSlots<T> = Vec<Option<T>>;

fn status_slots<T>() -> StatusSlots<T> {
    (0..=ExecStatus::LAST_STATUS).map(|_| None).collect()
}

/// Status flags plus integers, strings and custom messages recorded per status;
/// `Algorithm::new()`.
///
/// Derived OCCT algorithms name their messages after their type chain; give the
/// descriptor with [`Algorithm::with_runtime_type`].
pub struct Algorithm {
    runtime_type: &'static RuntimeType,
    status: ExecStatus,
    messenger: Arc<Messenger>,
    report_integers: Option<StatusSlots<PackedMap<i32>>>,
    report_strings: Option<StatusSlots<Vec<ExtendedString>>>,
    // `Get()` rewrites a message while sending it from a const method.
    report_messages: Option<StatusSlots<Mutex<Msg>>>,
}
impl RuntimeTyped for Algorithm {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimeType::new::<Self>("Message_Algorithm", Some(StandardTransient::static_type()))
        })
    }
}
impl Default for Algorithm {
    fn default() -> Self {
        Self::with_runtime_type(Self::static_type())
    }
}
impl Algorithm {
    /// An algorithm sending to the default messenger; `Algorithm::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// An algorithm whose messages are named after `runtime_type` and its ancestors;
    /// `Algorithm::with_runtime_type(Algorithm::static_type())`.
    pub fn with_runtime_type(runtime_type: &'static RuntimeType) -> Self {
        Self {
            runtime_type,
            status: ExecStatus::new(),
            messenger: Message::default_messenger().clone(),
            report_integers: None,
            report_strings: None,
            report_messages: None,
        }
    }
    /// The descriptor naming the messages (`DynamicType`); `algorithm.dynamic_type()`.
    pub fn dynamic_type(&self) -> &'static RuntimeType {
        self.runtime_type
    }
    /// Raises a status flag; `algorithm.set_status(Status::WARN1)`.
    pub fn set_status(&mut self, status: Status) {
        self.status.set(status);
    }
    /// Raises a flag and records an integer for it; `algorithm.set_status_with_integer(Status::FAIL1, 7)`.
    pub fn set_status_with_integer(&mut self, status: Status, value: i32) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if index == 0 {
            return;
        }
        let slots = self.report_integers.get_or_insert_with(status_slots);
        slots[index]
            .get_or_insert_with(PackedMap::default)
            .add(value);
    }
    /// `SetStatus(stat, const char*)`: bytes widened one per unit; `algorithm.set_status_with_c_str(s, b"x", true)`.
    pub fn set_status_with_c_str(&mut self, status: Status, text: &[u8], no_repetitions: bool) {
        let units: Vec<u16> = text.iter().map(|&byte| u16::from(byte)).collect();
        self.set_status_with_h_extended(
            status,
            Some(&ExtendedString::from_utf16(&units)),
            no_repetitions,
        );
    }
    /// `SetStatus(stat, const TCollection_AsciiString&)`: decoded as UTF-8 when valid;
    /// `algorithm.set_status_with_ascii(s, &text, true)`.
    pub fn set_status_with_ascii(
        &mut self,
        status: Status,
        text: &AsciiString,
        no_repetitions: bool,
    ) {
        self.set_status_with_h_extended(
            status,
            Some(&ExtendedString::from_ascii(text)),
            no_repetitions,
        );
    }
    /// `SetStatus(stat, const occ::handle<TCollection_HAsciiString>&)`; decoded as UTF-8 when valid.
    pub fn set_status_with_h_ascii(
        &mut self,
        status: Status,
        text: &HAsciiString,
        no_repetitions: bool,
    ) {
        let text = AsciiString::from_bytes(text.as_bytes_with_nul());
        self.set_status_with_ascii(status, &text, no_repetitions);
    }
    /// `SetStatus(stat, const TCollection_ExtendedString&)`; `algorithm.set_status_with_extended(s, &text, true)`.
    pub fn set_status_with_extended(
        &mut self,
        status: Status,
        text: &ExtendedString,
        no_repetitions: bool,
    ) {
        self.set_status_with_h_extended(status, Some(text), no_repetitions);
    }
    /// Raises a flag and records a string (a null handle records nothing), optionally
    /// skipping an equal string; `algorithm.set_status_with_h_extended(s, Some(&text), true)`.
    pub fn set_status_with_h_extended(
        &mut self,
        status: Status,
        text: Option<&ExtendedString>,
        no_repetitions: bool,
    ) {
        self.set_status(status);
        let (Some(text), index) = (text, ExecStatus::status_index(status) as usize) else {
            return;
        };
        if index == 0 {
            return;
        }
        let slots = self.report_strings.get_or_insert_with(status_slots);
        let strings = slots[index].get_or_insert_with(Vec::new);
        if no_repetitions && strings.iter().any(|existing| existing.is_equal(text)) {
            return;
        }
        strings.push(text.clone());
    }
    /// Raises a flag and stores a custom message for it; `algorithm.set_status_with_msg(s, &msg)`.
    pub fn set_status_with_msg(&mut self, status: Status, message: &Msg) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if index == 0 {
            return;
        }
        let slots = self.report_messages.get_or_insert_with(status_slots);
        slots[index] = Some(Mutex::new(message.clone()));
    }
    /// The status flags; `algorithm.get_status().is_fail()`.
    pub fn get_status(&self) -> &ExecStatus {
        &self.status
    }
    /// The status flags for editing; `algorithm.change_status().clear()`.
    pub fn change_status(&mut self) -> &mut ExecStatus {
        &mut self.status
    }
    /// Clears the flags and every recorded datum; `algorithm.clear_status()`.
    pub fn clear_status(&mut self) {
        self.status.clear();
        self.report_integers = None;
        self.report_strings = None;
        self.report_messages = None;
    }
    /// Sets the messenger; `None` selects the default one; `algorithm.set_messenger(None)`.
    pub fn set_messenger(&mut self, messenger: Option<Arc<Messenger>>) {
        self.messenger = messenger.unwrap_or_else(|| Message::default_messenger().clone());
    }
    /// The messenger; `algorithm.get_messenger()`.
    pub fn get_messenger(&self) -> Arc<Messenger> {
        self.messenger.clone()
    }
    /// Integers recorded for a status; `algorithm.get_message_numbers(Status::FAIL1)`.
    pub fn get_message_numbers(&self, status: Status) -> Option<&PackedMap<i32>> {
        let index = ExecStatus::status_index(status) as usize;
        self.report_integers
            .as_ref()?
            .get(index)?
            .as_ref()
            .filter(|_| index != 0)
    }
    /// Strings recorded for a status; `algorithm.get_message_strings(Status::FAIL1)`.
    pub fn get_message_strings(&self, status: Status) -> Option<&[ExtendedString]> {
        let index = ExecStatus::status_index(status) as usize;
        self.report_strings
            .as_ref()?
            .get(index)?
            .as_deref()
            .filter(|_| index != 0)
    }
}
