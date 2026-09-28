// Copyright (c) 2020 OPEN CASCADE SAS
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

//! `Message_AlertExtended`: an alert with an attribute and optional child alerts.
use super::{
    Alert, AlertHandle, AttributeHandle, BaseAlert, CompositeAlerts, Gravity, Report, json,
};
use crate::standard::dump::DumpStream;
use crate::standard::{RuntimeType, RuntimeTyped};
use crate::tcollection::AsciiString;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

/// Alert carrying an attribute and a lazily created child container; `AlertExtended::default()`.
#[derive(Default)]
pub struct AlertExtended {
    composite: Mutex<Option<Arc<CompositeAlerts>>>,
    attribute: Mutex<Option<AttributeHandle>>,
}
impl RuntimeTyped for AlertExtended {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimeType::new::<Self>("Message_AlertExtended", Some(BaseAlert::static_type()))
        })
    }
}
fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
impl AlertExtended {
    /// Creates an alert without attribute or children; `AlertExtended::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// Creates an alert holding `attribute` and adds it to `report`; returns it;
    /// `AlertExtended::add_alert(&report, Some(attribute), Gravity::Info)`.
    pub fn add_alert(
        report: &Arc<Report>,
        attribute: Option<AttributeHandle>,
        gravity: Gravity,
    ) -> AlertHandle {
        let alert = Arc::new(Self::new());
        alert.set_attribute(attribute);
        let alert: AlertHandle = alert;
        report.add_alert(gravity, &alert);
        alert
    }
    /// The attribute, if any; `alert.attribute()`.
    pub fn attribute(&self) -> Option<AttributeHandle> {
        locked(&self.attribute).clone()
    }
    /// Replaces the attribute; `alert.set_attribute(Some(attribute))`.
    pub fn set_attribute(&self, attribute: Option<AttributeHandle>) {
        *locked(&self.attribute) = attribute;
    }
    /// The child container, created on request; `alert.composite_alerts(true)`.
    pub fn composite_alerts(&self, to_create: bool) -> Option<Arc<CompositeAlerts>> {
        let mut composite = locked(&self.composite);
        if composite.is_none() && to_create {
            *composite = Some(Arc::new(CompositeAlerts::new()));
        }
        composite.clone()
    }
}
impl Alert for AlertExtended {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// The attribute's key, else the type name.
    fn get_message_key(&self) -> AsciiString {
        match self.attribute() {
            Some(attribute) => attribute.get_message_key(),
            None => AsciiString::new(self.dynamic_type().name()),
        }
    }
    /// Hierarchical alerts (with any child) never merge.
    fn supports_merge(&self) -> bool {
        let Some(composite) = self.composite_alerts(false) else {
            return true;
        };
        Gravity::ALL
            .iter()
            .all(|gravity| composite.alerts(*gravity).is_empty())
    }
    /// Extended alerts do not merge: always false.
    fn merge(&self, _target: &AlertHandle) -> bool {
        false
    }
    /// Class name, then `CompositAlerts` and `Attribute` child dumps when present.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        if let Some(composite) = self.composite_alerts(false) {
            json::field_dumped(output, depth, "CompositAlerts", |field, depth| {
                composite.dump_json(field, depth)
            });
        }
        if let Some(attribute) = self.attribute() {
            json::field_dumped(output, depth, "Attribute", |field, depth| {
                attribute.dump_json(field, depth)
            });
        }
    }
}
