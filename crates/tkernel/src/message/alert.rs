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

//! `Message_Alert`: a situation met by an algorithm, identified by a message key.
use super::json;
use crate::standard::dump::DumpStream;
use crate::standard::{RuntimeType, RuntimeTyped, StandardTransient};
use crate::tcollection::AsciiString;
use std::any::Any;
use std::sync::{Arc, OnceLock};

/// Shared alert handle, the Rust `occ::handle<Message_Alert>`; alerts are never null.
pub type AlertHandle = Arc<dyn Alert>;

/// Alert behavior; the provided methods are the `Message_Alert` base implementations.
///
/// ```
/// use rustycad_tkernel::message::{Alert, BaseAlert};
/// use rustycad_tkernel::standard::RuntimeTyped;
/// let alert = BaseAlert::default();
/// assert_eq!(alert.get_message_key().as_bytes(), b"Message_Alert");
/// ```
pub trait Alert: Any + Send + Sync {
    /// The most-derived runtime descriptor (`DynamicType`); `alert.dynamic_type().name()`.
    fn dynamic_type(&self) -> &'static RuntimeType;
    /// Key of the user message describing the alert; the type name by default.
    fn get_message_key(&self) -> AsciiString {
        AsciiString::new(self.dynamic_type().name())
    }
    /// Whether alerts of this type merge with an earlier one; true by default.
    fn supports_merge(&self) -> bool {
        true
    }
    /// Merges this alert's data into `target`; the base merges trivially and returns true.
    fn merge(&self, _target: &AlertHandle) -> bool {
        true
    }
    /// `DumpJson`; the base writes `"className": "Message_Alert"` (the static class name).
    fn dump_json(&self, output: &mut DumpStream, _depth: i32) {
        json::class_begin(output, BaseAlert::static_type().name());
    }
}

/// A plain `Message_Alert` instance; `Arc::new(BaseAlert::default())`.
#[derive(Debug, Default)]
pub struct BaseAlert;
impl RuntimeTyped for BaseAlert {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimeType::new::<Self>("Message_Alert", Some(StandardTransient::static_type()))
        })
    }
}
impl Alert for BaseAlert {
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
}

/// `DEFINE_SIMPLE_ALERT(Name)`: a data-less alert type deriving `Message_Alert`.
///
/// ```
/// rustycad_tkernel::define_simple_alert!(MyAlert, "MyAlert");
/// use rustycad_tkernel::message::Alert;
/// assert_eq!(MyAlert.get_message_key().as_bytes(), b"MyAlert");
/// ```
#[macro_export]
macro_rules! define_simple_alert {
    ($alert:ident, $name:literal) => {
        #[doc = concat!("Simple alert `", $name, "`.")]
        #[derive(Debug, Default)]
        pub struct $alert;
        impl $crate::standard::RuntimeTyped for $alert {
            fn static_type() -> &'static $crate::standard::RuntimeType {
                static DESCRIPTOR: std::sync::OnceLock<$crate::standard::RuntimeType> =
                    std::sync::OnceLock::new();
                DESCRIPTOR.get_or_init(|| {
                    $crate::standard::RuntimeType::new::<Self>(
                        $name,
                        Some(<$crate::message::BaseAlert as $crate::standard::RuntimeTyped>::static_type()),
                    )
                })
            }
        }
        impl $crate::message::Alert for $alert {
            fn dynamic_type(&self) -> &'static $crate::standard::RuntimeType {
                <Self as $crate::standard::RuntimeTyped>::static_type()
            }
        }
    };
}

/// Downcasts a shared alert to a concrete type (`occ::down_cast`); `downcast_alert::<AlertExtended>(&a)`.
pub fn downcast_alert<T: Alert>(alert: &AlertHandle) -> Option<Arc<T>> {
    let any: Arc<dyn Any + Send + Sync> = alert.clone();
    any.downcast::<T>().ok()
}
