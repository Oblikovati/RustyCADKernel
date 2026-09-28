// Copyright (c) 2019 OPEN CASCADE SAS
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

//! `Message_Level`: a scope sentry opening a level in the default report.
use super::{AlertExtended, AlertHandle, AttributeMeter, Gravity, Message, downcast_alert};
use crate::tcollection::AsciiString;
use std::sync::{Arc, Mutex, MutexGuard};

/// The level state shared by the sentry and the report's level stack.
#[derive(Default)]
pub(crate) struct LevelCore {
    root: Mutex<Option<Arc<AlertExtended>>>,
    last: Mutex<Option<Arc<AlertExtended>>>,
}
fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
impl LevelCore {
    pub(crate) fn root_alert(&self) -> Option<Arc<AlertExtended>> {
        locked(&self.root).clone()
    }
    pub(crate) fn set_root_alert(
        &self,
        alert: Option<Arc<AlertExtended>>,
        is_required_to_start: bool,
    ) {
        *locked(&self.root) = alert.clone();
        if is_required_to_start {
            AttributeMeter::start_alert(alert.as_ref());
        }
    }
    /// Stops the previous alert's meters, starts the new one's and adds it below the root.
    ///
    /// Deviation: with no root alert the native code dereferences a null handle; here the
    /// call does nothing and returns false.
    pub(crate) fn add_alert(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        let Some(extended) = downcast_alert::<AlertExtended>(alert) else {
            return false;
        };
        let Some(root) = self.root_alert() else {
            return false;
        };
        let composite = root.composite_alerts(true).expect("created on request");
        let previous = locked(&self.last).replace(extended.clone());
        AttributeMeter::stop_alert(previous.as_ref());
        AttributeMeter::start_alert(Some(&extended));
        composite.add_alert(gravity, alert);
        true
    }
    fn last_alert(&self) -> Option<Arc<AlertExtended>> {
        locked(&self.last).clone()
    }
}

/// Scope sentry: while alive, alerts of the default report go below its root alert;
/// `let _level = Level::new(&AsciiString::new("step"));` (`OCCT_ADD_MESSAGE_LEVEL_SENTRY`).
pub struct Level {
    core: Arc<LevelCore>,
}
impl Level {
    /// Opens a level in the default report when it exists and is active in its messenger;
    /// `Level::new(&AsciiString::new("step"))`.
    pub fn new(name: &AsciiString) -> Self {
        let level = Self {
            core: Arc::new(LevelCore::default()),
        };
        if let Some(report) =
            Message::default_report(false).filter(|report| report.is_active_in_messenger(None))
        {
            report.add_level(&level, name);
        }
        level
    }
    pub(crate) fn core(&self) -> &Arc<LevelCore> {
        &self.core
    }
    /// The root alert, set by the report; `level.root_alert()`.
    pub fn root_alert(&self) -> Option<Arc<AlertExtended>> {
        self.core.root_alert()
    }
    /// Sets the root alert, starting its meters on request; `level.set_root_alert(Some(alert), true)`.
    pub fn set_root_alert(&self, alert: Option<Arc<AlertExtended>>, is_required_to_start: bool) {
        self.core.set_root_alert(alert, is_required_to_start);
    }
    /// Adds an extended alert below the root; false for other alerts; `level.add_alert(g, &alert)`.
    pub fn add_alert(&self, gravity: Gravity, alert: &AlertHandle) -> bool {
        self.core.add_alert(gravity, alert)
    }
    /// `remove()`: stops the last alert's meters and closes the level in the default report.
    fn remove(&self) {
        let Some(report) =
            Message::default_report(false).filter(|report| report.is_active_in_messenger(None))
        else {
            return;
        };
        AttributeMeter::stop_alert(self.core.last_alert().as_ref());
        report.remove_level(self);
    }
}
impl Drop for Level {
    fn drop(&mut self) {
        self.remove();
    }
}
