// Created on: 2017-06-26
// Created by: Andrey Betenev
// Copyright (c) 2017 OPEN CASCADE SAS
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

//! Report activation in a messenger and the level stack.
use super::Report;
use crate::message::level::Level;
use crate::message::{
    AlertExtended, AlertHandle, AttributeHandle, AttributeMeter, BaseAttribute, Gravity, Message,
    Messenger, PrinterHandle, PrinterToReport,
};
use crate::tcollection::AsciiString;
use std::any::Any;
use std::sync::Arc;

/// The report a printer forwards to, when it is a `PrinterToReport`.
fn forwarded_report(printer: &PrinterHandle) -> Option<Arc<Report>> {
    let any: Arc<dyn Any + Send + Sync> = printer.clone();
    any.downcast::<PrinterToReport>()
        .ok()
        .map(|printer| printer.report())
}

/// `theMessenger.IsNull() ? Message::DefaultMessenger() : theMessenger`.
fn messenger_or_default(messenger: Option<&Arc<Messenger>>) -> Arc<Messenger> {
    messenger
        .cloned()
        .unwrap_or_else(|| Message::default_messenger().clone())
}

impl Report {
    /// The activation flag; the messenger argument is ignored natively;
    /// `report.is_active_in_messenger(None)`.
    pub fn is_active_in_messenger(&self, _messenger: Option<&Arc<Messenger>>) -> bool {
        self.is_active_in_messenger
            .load(std::sync::atomic::Ordering::Relaxed)
    }
    /// Adds (or removes) a `PrinterToReport` to this report in the messenger (default one
    /// when `None`); `report.activate_in_messenger(true, None)`.
    pub fn activate_in_messenger(
        self: &Arc<Self>,
        to_activate: bool,
        messenger: Option<&Arc<Messenger>>,
    ) {
        if to_activate == self.is_active_in_messenger(None) {
            return;
        }
        self.is_active_in_messenger
            .store(to_activate, std::sync::atomic::Ordering::Relaxed);
        let messenger = messenger_or_default(messenger);
        if to_activate {
            let printer = PrinterToReport::new();
            printer.set_report(Some(self.clone()));
            messenger.add_printer(Arc::new(printer));
            return;
        }
        let printers = messenger.printers().clone();
        let to_remove = printers.iter().filter(|printer| {
            forwarded_report(printer).is_some_and(|report| Arc::ptr_eq(&report, self))
        });
        for printer in to_remove {
            messenger.remove_printer(printer);
        }
    }
    /// Recomputes the flag from the messenger's printers; `report.update_active_in_messenger(None)`.
    pub fn update_active_in_messenger(self: &Arc<Self>, messenger: Option<&Arc<Messenger>>) {
        let messenger = messenger_or_default(messenger);
        let printers = messenger.printers().clone();
        let active = printers.iter().any(|printer| {
            forwarded_report(printer).is_some_and(|report| Arc::ptr_eq(&report, self))
        });
        self.is_active_in_messenger
            .store(active, std::sync::atomic::Ordering::Relaxed);
    }
    /// Opens a level: new alerts go below its root alert, a meter when metrics are active;
    /// `report.add_level(&level, &AsciiString::new("step"))`.
    pub fn add_level(&self, level: &Level, name: &AsciiString) {
        let mut state = self.state();
        state.levels.push(level.core().clone());
        let attribute: AttributeHandle = if self.active_metrics().is_empty() {
            Arc::new(BaseAttribute::new(name))
        } else {
            Arc::new(AttributeMeter::new(name))
        };
        let root = Arc::new(AlertExtended::new());
        root.set_attribute(Some(attribute));
        let is_first = state.levels.len() == 1;
        level.core().set_root_alert(Some(root.clone()), is_first);
        if is_first {
            let composite = state.composite.get_or_insert_with(Default::default).clone();
            composite.add_alert(Gravity::Info, &(root as AlertHandle));
            return;
        }
        let previous = state.levels[state.levels.len() - 2].clone();
        previous.add_alert(Gravity::Info, &(root as AlertHandle));
    }
    /// Closes `level` and every level opened after it, stopping their root meters;
    /// `report.remove_level(&level)`.
    pub fn remove_level(&self, level: &Level) {
        let mut state = self.state();
        while let Some(last) = state.levels.pop() {
            AttributeMeter::stop_alert(last.root_alert().as_ref());
            if Arc::ptr_eq(&last, level.core()) {
                return;
            }
        }
    }
}
