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

//! `Message_AttributeMeter::SetAlertMetrics`: sampling the default report's metrics.
use super::AttributeMeter;
use crate::message::{AlertExtended, Message, MetricSampler, MetricType, downcast_attribute};
use crate::ncollection::IndexedMap;
use crate::osd::MemInfoCounter;
use std::sync::Arc;

/// Stores `value` as the start or stop value of `metric`.
fn record(meter: &AttributeMeter, metric: MetricType, value: f64, is_start: bool) {
    if is_start {
        meter.set_start_value(metric, value);
    } else {
        meter.set_stop_value(metric, value);
    }
}

impl AttributeMeter {
    /// Starts the meter of an alert; `AttributeMeter::start_alert(Some(&alert))`.
    pub fn start_alert(alert: Option<&Arc<AlertExtended>>) {
        Self::set_alert_metrics(alert, true);
    }
    /// Stops the meter of an alert; `AttributeMeter::stop_alert(Some(&alert))`.
    pub fn stop_alert(alert: Option<&Arc<AlertExtended>>) {
        Self::set_alert_metrics(alert, false);
    }
    /// Samples the active metrics of the default report (creating it) into the alert's
    /// meter attribute; nothing without such an attribute;
    /// `AttributeMeter::set_alert_metrics(Some(&alert), true)`.
    pub fn set_alert_metrics(alert: Option<&Arc<AlertExtended>>, is_start: bool) {
        let Some(meter) = alert
            .and_then(|alert| alert.attribute())
            .and_then(|attribute| downcast_attribute::<AttributeMeter>(&attribute))
        else {
            return;
        };
        let report = Message::default_report(true).expect("created on request");
        let active = report.active_metrics();
        let sampler = report.metric_sampler();
        record_times(&meter, &active, sampler.as_ref(), is_start);
        record_memory(&meter, &active, sampler.as_ref(), is_start);
    }
}

fn record_times(
    meter: &AttributeMeter,
    active: &[MetricType],
    sampler: &dyn MetricSampler,
    is_start: bool,
) {
    let has = |metric: MetricType| active.contains(&metric);
    if has(MetricType::WallClock) {
        record(
            meter,
            MetricType::WallClock,
            sampler.wall_clock_time(),
            is_start,
        );
    }
    if has(MetricType::ProcessCpuUserTime) || has(MetricType::ProcessCpuSystemTime) {
        let (user, system) = sampler.process_cpu();
        if has(MetricType::ProcessCpuUserTime) {
            record(meter, MetricType::ProcessCpuUserTime, user, is_start);
        }
        if has(MetricType::ProcessCpuSystemTime) {
            record(meter, MetricType::ProcessCpuSystemTime, system, is_start);
        }
    }
    if has(MetricType::ThreadCpuUserTime) || has(MetricType::ThreadCpuSystemTime) {
        let (user, system) = sampler.thread_cpu();
        if has(MetricType::ThreadCpuUserTime) {
            record(meter, MetricType::ThreadCpuUserTime, user, is_start);
        }
        if has(MetricType::ThreadCpuSystemTime) {
            record(meter, MetricType::ThreadCpuSystemTime, system, is_start);
        }
    }
}

/// Memory counters of the active metrics, deduplicated in activation order.
fn record_memory(
    meter: &AttributeMeter,
    active: &[MetricType],
    sampler: &dyn MetricSampler,
    is_start: bool,
) {
    let mut counters: IndexedMap<MemInfoCounter> = IndexedMap::default();
    for metric in active {
        if let Some(counter) = Message::to_osd_metric(*metric) {
            counters.add(counter);
        }
    }
    if counters.is_empty() {
        return;
    }
    let counters: Vec<MemInfoCounter> = counters.iter().copied().collect();
    let values = sampler.memory_mib(&counters);
    for (counter, value) in counters.iter().zip(values) {
        if let Some(metric) = Message::to_message_metric(*counter) {
            record(meter, metric, value, is_start);
        }
    }
}
