// Copyright (c) 2026 OPEN CASCADE SAS
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

//! The process-wide Message state: default messenger and report, levels and metrics.
//! Every test holds `DEFAULTS_GUARD`, since they share the default messenger and report.
#[path = "message_api_behavior/fakes.rs"]
mod fakes;
use fakes::{FakeMetricSampler, RecordingPrinter};
use rustycad_tkernel::message::{
    AlertExtended, AlertHandle, AttributeMeter, BaseAttribute, Gravity, Level, Message, MetricType,
    PrinterHandle, downcast_alert, downcast_attribute,
};
use rustycad_tkernel::message::{Attribute, Printer};
use rustycad_tkernel::osd::MemCounter;
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Write;
use std::sync::{Arc, Mutex, MutexGuard};

static DEFAULTS_GUARD: Mutex<()> = Mutex::new(());

fn guard() -> MutexGuard<'static, ()> {
    DEFAULTS_GUARD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Replaces the default messenger's printers by one recorder; returns the originals.
fn record_default_messenger() -> (Arc<RecordingPrinter>, Vec<PrinterHandle>) {
    let recorder = Arc::new(RecordingPrinter::default());
    recorder.set_trace_level(Gravity::Trace);
    let messenger = Message::default_messenger();
    let original = std::mem::take(&mut *messenger.printers_mut());
    messenger.add_printer(recorder.clone());
    (recorder, original)
}

fn restore_default_messenger(original: Vec<PrinterHandle>) {
    *Message::default_messenger().printers_mut() = original;
}

#[test]
fn shortcuts_send_through_the_default_messenger() {
    let _guard = guard();
    let (recorder, original) = record_default_messenger();
    Message::send_text(&AsciiString::new("text"), Gravity::Info);
    Message::send_fail_text(&AsciiString::new("f"));
    Message::send_alarm_text(&AsciiString::new("a"));
    Message::send_warning_text(&AsciiString::new("w"));
    Message::send_info_text(&AsciiString::new("i"));
    Message::send_trace_text(&AsciiString::new("t"));
    write!(Message::send(Gravity::Info), "s").unwrap();
    write!(Message::send_fail(), "1").unwrap();
    write!(Message::send_alarm(), "2").unwrap();
    write!(Message::send_warning(), "3").unwrap();
    write!(Message::send_info(), "4").unwrap();
    write!(Message::send_trace(), "5").unwrap();
    restore_default_messenger(original);
    assert_eq!(
        recorder.texts(),
        [
            "text", "f", "a", "w", "i", "t", "s", "1", "2", "3", "4", "5"
        ]
    );
    assert!(Arc::ptr_eq(
        Message::default_messenger(),
        Message::default_messenger()
    ));
}

#[test]
fn time_and_metric_names() {
    assert_eq!(
        Message::fill_time(5, 12, 26.3345).as_bytes(),
        b"05h:12m:26.33s"
    );
    assert_eq!(Message::fill_time(0, 6, 34.496).as_bytes(), b"06m:34.50s");
    assert_eq!(Message::fill_time(0, 0, 4.5).as_bytes(), b"4.50s");
    assert_eq!(
        Message::metric_to_string(MetricType::MemHeapUsage),
        "MemHeapUsage"
    );
    assert_eq!(
        Message::metric_from_string("WallClock"),
        Some(MetricType::WallClock)
    );
    assert_eq!(Message::metric_from_string("wallclock"), None);
    assert_eq!(Message::metric_from_string_or_none("?"), MetricType::None);
    assert_eq!(
        Message::to_osd_metric(MetricType::MemVirtual),
        Some(MemCounter::Virtual)
    );
    assert_eq!(Message::to_osd_metric(MetricType::WallClock), None);
    assert_eq!(
        Message::to_message_metric(MemCounter::HeapUsage),
        Some(MetricType::MemHeapUsage)
    );
}

/// The meter of an extended alert, if it has one.
fn meter_of(alert: &AlertHandle) -> Option<Arc<AttributeMeter>> {
    let extended = downcast_alert::<AlertExtended>(alert)?;
    downcast_attribute::<AttributeMeter>(&extended.attribute()?)
}

#[test]
fn levels_nest_alerts_and_sample_injected_metrics() {
    let _guard = guard();
    let (_, original) = record_default_messenger();
    let report = Message::default_report(true).unwrap();
    report.set_metric_sampler(Arc::new(FakeMetricSampler::default()));
    report.clear();
    report.set_active_metric(MetricType::WallClock, true);
    report.set_active_metric(MetricType::MemSwapUsage, true);
    report.activate_in_messenger(true, None);
    {
        let level = Level::new(&AsciiString::new("outer"));
        let child = AlertExtended::new();
        child.set_attribute(Some(Arc::new(AttributeMeter::new(&AsciiString::new(
            "step",
        )))));
        let child: AlertHandle = Arc::new(child);
        assert!(level.add_alert(Gravity::Info, &child));
        assert!(!level.add_alert(
            Gravity::Info,
            &(Arc::new(rustycad_tkernel::message::BaseAlert) as AlertHandle)
        ));
        assert!(level.root_alert().is_some());
    }
    let top = report.get_alerts(Gravity::Info);
    let root = meter_of(&top[0]).unwrap();
    assert_eq!(root.get_name().as_bytes(), b"outer");
    assert!(root.is_metric_valid(MetricType::WallClock));
    assert!(!root.is_metric_valid(MetricType::MemSwapUsage));
    report.activate_in_messenger(false, None);
    report.clear();
    report.clear_metrics();
    restore_default_messenger(original);
}

#[test]
fn a_level_without_an_active_report_records_nothing() {
    let _guard = guard();
    let report = Message::default_report(true).unwrap();
    report.update_active_in_messenger(None);
    let level = Level::new(&AsciiString::new("idle"));
    assert!(level.root_alert().is_none());
    let alert = AlertExtended::new();
    alert.set_attribute(Some(Arc::new(BaseAttribute::new(&AsciiString::new("x")))));
    // Native code dereferences the null root here; the port refuses the alert.
    assert!(!level.add_alert(Gravity::Info, &(Arc::new(alert) as AlertHandle)));
    AttributeMeter::set_alert_metrics(None, true);
}
