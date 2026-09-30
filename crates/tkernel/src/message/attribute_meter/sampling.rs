//! `Message_AttributeMeter::SetAlertMetrics`: sampling the default report's metrics.
use super::AttributeMeter;
use crate::message::{AlertExtended, Message, MetricSampler, MetricType, downcast_attribute};
use crate::ncollection::IndexedMap;
use crate::osd::MemCounter;
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
    if active.contains(&MetricType::WallClock) {
        let time = sampler.wall_clock_time();
        record(meter, MetricType::WallClock, time, is_start);
    }
    let process = [
        MetricType::ProcessCpuUserTime,
        MetricType::ProcessCpuSystemTime,
    ];
    record_pair(meter, active, process, || sampler.process_cpu(), is_start);
    let thread = [
        MetricType::ThreadCpuUserTime,
        MetricType::ThreadCpuSystemTime,
    ];
    record_pair(meter, active, thread, || sampler.thread_cpu(), is_start);
}

/// A `(user, system)` clock, sampled once when either of its two metrics is active.
fn record_pair(
    meter: &AttributeMeter,
    active: &[MetricType],
    metrics: [MetricType; 2],
    sample: impl FnOnce() -> (f64, f64),
    is_start: bool,
) {
    if !metrics.iter().any(|metric| active.contains(metric)) {
        return;
    }
    let (user, system) = sample();
    for (metric, value) in metrics.into_iter().zip([user, system]) {
        if active.contains(&metric) {
            record(meter, metric, value, is_start);
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
    let mut counters: IndexedMap<MemCounter> = IndexedMap::default();
    for metric in active {
        if let Some(counter) = Message::to_osd_metric(*metric) {
            counters.add(counter);
        }
    }
    if counters.is_empty() {
        return;
    }
    let counters: Vec<MemCounter> = counters.iter().copied().collect();
    let values = sampler.memory_mib(&counters);
    for (counter, value) in counters.iter().zip(values) {
        if let Some(metric) = Message::to_message_metric(*counter) {
            record(meter, metric, value, is_start);
        }
    }
}
