//! `Message_Report`: a thread-safe container of alerts, optionally grouped in levels.
use super::level::LevelCore;
use super::{
    AlertHandle, CompositeAlerts, Gravity, MetricSampler, MetricType, NativeMetricSampler,
};
use crate::ncollection::IndexedMap;
use crate::standard::{RuntimeType, StandardTransient};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
mod messenger_link;
mod output;

/// Alerts and the stack of open levels, guarded by the report mutex (`myMutex`).
#[derive(Default)]
pub(crate) struct ReportState {
    pub(crate) composite: Option<Arc<CompositeAlerts>>,
    pub(crate) levels: Vec<Arc<LevelCore>>,
}

/// Collected alerts per gravity with active metrics and a limit; `Arc::new(Report::new())`.
pub struct Report {
    state: Mutex<ReportState>,
    active_metrics: Mutex<IndexedMap<MetricType>>,
    limit: AtomicI32,
    is_active_in_messenger: AtomicBool,
    // Replaceable so the process-wide default report can sample through an injected fake.
    sampler: RwLock<Arc<dyn MetricSampler>>,
}
message_runtime_type!(Report, "Message_Report", StandardTransient);
impl Default for Report {
    fn default() -> Self {
        Self::with_metric_sampler(Arc::new(NativeMetricSampler))
    }
}
impl Report {
    /// An empty, inactive report with limit -1 and native metric sampling; `Report::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// An empty report whose alert meters sample through `sampler`;
    /// `Report::with_metric_sampler(Arc::new(NativeMetricSampler))`.
    pub fn with_metric_sampler(sampler: Arc<dyn MetricSampler>) -> Self {
        Self {
            state: Mutex::new(ReportState::default()),
            active_metrics: Mutex::new(IndexedMap::default()),
            limit: AtomicI32::new(-1),
            is_active_in_messenger: AtomicBool::new(false),
            sampler: RwLock::new(sampler),
        }
    }
    pub(crate) fn state(&self) -> MutexGuard<'_, ReportState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    /// The sampler used by alert meters of this report; `report.metric_sampler()`.
    pub fn metric_sampler(&self) -> Arc<dyn MetricSampler> {
        self.sampler
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
    /// Replaces the sampler, e.g. of `Message::default_report`; `report.set_metric_sampler(sampler)`.
    pub fn set_metric_sampler(&self, sampler: Arc<dyn MetricSampler>) {
        *self
            .sampler
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = sampler;
    }
    /// `compositeAlerts(isCreate)`: the top-level container, created on request.
    fn composite(&self, to_create: bool) -> Option<Arc<CompositeAlerts>> {
        let mut state = self.state();
        if state.composite.is_none() && to_create {
            state.composite = Some(Arc::new(CompositeAlerts::new()));
        }
        state.composite.clone()
    }
    /// Adds an alert below the innermost open level, or at the top level; `report.add_alert(g, &alert)`.
    ///
    /// Native quirk kept: the limit is enforced only when the alert was merged, and then the
    /// first alert of the gravity is removed whenever the list is longer than the limit;
    /// with the default limit -1 every merge removes the oldest alert.
    pub fn add_alert(&self, gravity: Gravity, alert: &AlertHandle) {
        let mut state = self.state();
        if let Some(level) = state.levels.last().cloned() {
            level.add_alert(gravity, alert);
            return;
        }
        let composite = state
            .composite
            .get_or_insert_with(|| Arc::new(CompositeAlerts::new()))
            .clone();
        if composite.add_alert(gravity, alert) {
            return;
        }
        let alerts = composite.alerts(gravity);
        if alerts.len() as i64 > i64::from(self.limit()) {
            composite.remove_alert(gravity, &alerts[0]);
        }
    }
    /// Top-level alerts of a gravity; `report.get_alerts(Gravity::Warning)`.
    pub fn get_alerts(&self, gravity: Gravity) -> Vec<AlertHandle> {
        self.composite(false)
            .map_or_else(Vec::new, |composite| composite.alerts(gravity))
    }
    /// Whether a top-level alert of exactly `kind` exists at any gravity; `report.has_alert(t)`.
    pub fn has_alert(&self, kind: &RuntimeType) -> bool {
        Gravity::ALL
            .iter()
            .any(|gravity| self.has_alert_with_gravity(kind, *gravity))
    }
    /// Whether a top-level alert of exactly `kind` has this gravity; `report.has_alert_with_gravity(t, g)`.
    pub fn has_alert_with_gravity(&self, kind: &RuntimeType, gravity: Gravity) -> bool {
        self.composite(false)
            .is_some_and(|composite| composite.has_alert_of_type(kind, gravity))
    }
    /// Removes every alert and forgets open levels; `report.clear()`.
    pub fn clear(&self) {
        self.clear_with(CompositeAlerts::clear);
    }
    /// Removes one gravity's alerts and forgets open levels; `report.clear_gravity(Gravity::Info)`.
    pub fn clear_gravity(&self, gravity: Gravity) {
        self.clear_with(|composite| composite.clear_gravity(gravity));
    }
    /// Removes alerts of exactly `kind` and forgets open levels; `report.clear_type(t)`.
    pub fn clear_type(&self, kind: &RuntimeType) {
        self.clear_with(|composite| composite.clear_type(kind));
    }
    fn clear_with(&self, action: impl FnOnce(&CompositeAlerts)) {
        let mut state = self.state();
        let Some(composite) = state.composite.clone() else {
            return;
        };
        action(&composite);
        state.levels.clear();
    }
}

impl Report {
    /// The metrics computed on alerts, in activation order; `report.active_metrics()`.
    pub fn active_metrics(&self) -> Vec<MetricType> {
        let metrics = self
            .active_metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        metrics.iter().copied().collect()
    }
    /// Activates or deactivates a metric (removal moves the last metric into its slot);
    /// `report.set_active_metric(MetricType::WallClock, true)`.
    pub fn set_active_metric(&self, metric: MetricType, activate: bool) {
        let mut metrics = self
            .active_metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if activate == metrics.contains(&metric) {
            return;
        }
        if activate {
            metrics.add(metric);
        } else {
            metrics.remove_key(&metric);
        }
    }
    /// Deactivates every metric; `report.clear_metrics()`.
    pub fn clear_metrics(&self) {
        let mut metrics = self
            .active_metrics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        metrics.clear(true);
    }
    /// The alert limit (-1 by default); `report.limit()`.
    pub fn limit(&self) -> i32 {
        self.limit.load(Ordering::Relaxed)
    }
    /// Sets the alert limit; `report.set_limit(10)`.
    pub fn set_limit(&self, limit: i32) {
        self.limit.store(limit, Ordering::Relaxed);
    }
    /// Adds every top-level alert of `other` (all gravities); `report.merge(&other)`.
    pub fn merge(&self, other: &Report) {
        for gravity in Gravity::ALL {
            self.merge_gravity(other, gravity);
        }
    }
    /// Adds the top-level alerts of `other` with one gravity; `report.merge_gravity(&other, g)`.
    pub fn merge_gravity(&self, other: &Report, gravity: Gravity) {
        for alert in other.get_alerts(gravity) {
            self.add_alert(gravity, &alert);
        }
    }
}
