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

//! `Message_AttributeMeter`: start/stop values of the metrics collected on an alert.
use super::attribute::{AttributeBase, BaseAttribute, dump_attribute};
use super::{Attribute, MetricType, json};
use crate::ncollection::IndexedDataMap;
use crate::precision;
use crate::standard::dump::DumpStream;
use crate::standard::{RuntimeType, RuntimeTyped};
use crate::tcollection::AsciiString;
use std::sync::{Mutex, MutexGuard, OnceLock};
mod sampling;

/// Metric start/stop pairs in insertion order; `AttributeMeter::new(&AsciiString::default())`.
pub struct AttributeMeter {
    base: AttributeBase,
    metrics: Mutex<IndexedDataMap<MetricType, (f64, f64)>>,
}
impl AttributeMeter {
    /// Value of a metric that was never set; `AttributeMeter::undefined_metric_value() == -1.0`.
    pub const fn undefined_metric_value() -> f64 {
        -1.0
    }
    /// Creates a named meter; `AttributeMeter::new(&AsciiString::new("step"))`.
    pub fn new(name: &AsciiString) -> Self {
        Self {
            base: AttributeBase::new(name),
            metrics: Mutex::new(IndexedDataMap::default()),
        }
    }
    fn metrics(&self) -> MutexGuard<'_, IndexedDataMap<MetricType, (f64, f64)>> {
        self.metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// Whether the metric has values; `meter.has_metric(MetricType::WallClock)`.
    pub fn has_metric(&self, metric: MetricType) -> bool {
        self.metrics().contains(&metric)
    }
    /// Both values differ from the undefined value by more than `Precision::Confusion`;
    /// `meter.is_metric_valid(MetricType::WallClock)`.
    pub fn is_metric_valid(&self, metric: MetricType) -> bool {
        let undefined = Self::undefined_metric_value();
        (self.start_value(metric) - undefined).abs() > precision::confusion()
            && (self.stop_value(metric) - undefined).abs() > precision::confusion()
    }
    /// Start value, undefined when absent; `meter.start_value(MetricType::WallClock)`.
    pub fn start_value(&self, metric: MetricType) -> f64 {
        self.metrics()
            .seek(&metric)
            .map_or(Self::undefined_metric_value(), |values| values.0)
    }
    /// Sets the start value, adding the metric with an undefined stop; `meter.set_start_value(m, 1.0)`.
    pub fn set_start_value(&self, metric: MetricType, value: f64) {
        let mut metrics = self.metrics();
        if let Some(values) = metrics.change_seek(&metric) {
            values.0 = value;
            return;
        }
        metrics.add(metric, (value, Self::undefined_metric_value()));
    }
    /// Stop value, undefined when absent; `meter.stop_value(MetricType::WallClock)`.
    pub fn stop_value(&self, metric: MetricType) -> f64 {
        self.metrics()
            .seek(&metric)
            .map_or(Self::undefined_metric_value(), |values| values.1)
    }
    /// Sets the stop value of a present metric only; `meter.set_stop_value(m, 2.0)`.
    pub fn set_stop_value(&self, metric: MetricType, value: f64) {
        if let Some(values) = self.metrics().change_seek(&metric) {
            values.1 = value;
        }
    }
}
impl RuntimeTyped for AttributeMeter {
    fn static_type() -> &'static RuntimeType {
        static DESCRIPTOR: OnceLock<RuntimeType> = OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimeType::new::<Self>("Message_AttributeMeter", Some(BaseAttribute::static_type()))
        })
    }
}
impl Attribute for AttributeMeter {
    fn attribute_base(&self) -> &AttributeBase {
        &self.base
    }
    fn dynamic_type(&self) -> &'static RuntimeType {
        Self::static_type()
    }
    /// Class name, the base dump, then `"<MetricName>": [start, stop]` per metric.
    fn dump_json(&self, output: &mut DumpStream, depth: i32) {
        json::class_begin(output, Self::static_type().name());
        json::base_class(output, depth, |output, depth| {
            dump_attribute(&self.base, output, depth)
        });
        let metrics = self.metrics();
        for (metric, values) in metrics.items() {
            let name = super::Message::metric_to_string(*metric);
            json::field_reals(output, name, &[values.0, values.1]);
        }
    }
}
