// Created on: 1999-07-29
// Created by: Roman LYGIN
// Copyright (c) 1999 Matra Datavision
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

//! OCCT's Message package: messengers and printers, alerts and reports, the message
//! catalog, algorithm statuses and progress indication.

/// Registers a Message class under its OCCT name with its parent descriptor, as
/// `IMPLEMENT_STANDARD_RTTIEXT`; `message_runtime_type!(Report, "Message_Report", StandardTransient)`.
#[macro_export]
macro_rules! message_runtime_type {
    ($class:ty, $name:literal, $parent:ty) => {
        impl $crate::standard::RuntimeTyped for $class {
            const TYPE_NAME: &'static str = $name;
            fn static_type() -> &'static $crate::standard::RuntimeType {
                // One cached registry lookup per class, as the C++ function-local static handle.
                static DESCRIPTOR: std::sync::OnceLock<&'static $crate::standard::RuntimeType> =
                    std::sync::OnceLock::new();
                DESCRIPTOR.get_or_init(|| {
                    $crate::standard::RuntimeType::register::<Self>(
                        $name,
                        std::mem::size_of::<Self>(),
                        Some(<$parent as $crate::standard::RuntimeTyped>::static_type()),
                    )
                })
            }
            fn project_from(object: &dyn $crate::standard::Transient) -> Option<&Self> {
                $crate::standard::project_concrete::<Self>(object)
            }
        }
    };
}

mod json;

mod gravity;
pub use gravity::{ConsoleColor, Gravity, MetricType};
mod status;
pub use status::{Status, StatusType};
mod exec_status;
pub use exec_status::ExecStatus;

mod printer;
pub use printer::{Printer, PrinterBase, PrinterHandle, PrinterType};
mod printer_ostream;
pub use printer_ostream::PrinterOStream;
mod printer_system_log;
pub use printer_system_log::PrinterSystemLog;
mod printer_to_report;
pub use printer_to_report::PrinterToReport;
mod messenger;
pub use messenger::Messenger;
mod stream;
pub use stream::MessageStream;

mod alert;
pub use alert::{Alert, AlertHandle, BaseAlert, downcast_alert};
mod alert_extended;
pub use alert_extended::AlertExtended;
mod composite_alerts;
pub use composite_alerts::CompositeAlerts;
mod attribute;
pub use attribute::{Attribute, AttributeBase, AttributeHandle, BaseAttribute, downcast_attribute};
mod attribute_meter;
pub use attribute_meter::AttributeMeter;
mod attribute_object;
pub use attribute_object::AttributeObject;
mod attribute_stream;
pub use attribute_stream::AttributeStream;
mod metric_sampler;
pub use metric_sampler::{MetricSampler, NativeMetricSampler};
mod report;
pub use report::Report;
mod level;
pub use level::Level;

mod msg;
pub use msg::Msg;
mod msg_file;
pub use msg_file::{MsgFile, MsgFileSource, NativeMsgFileSource};
mod algorithm;
pub use algorithm::Algorithm;
mod defaults;
pub use defaults::Message;
mod lazy_progress_scope;
mod progress_indicator;
mod progress_range;
mod progress_scope;
mod progress_sentry;
pub use lazy_progress_scope::LazyProgressScope;
pub use progress_indicator::{ProgressIndicator, ProgressIndicatorHooks};
pub use progress_range::ProgressRange;
pub use progress_scope::ProgressScope;
pub use progress_sentry::ProgressSentry;
