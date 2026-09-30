//! The `Message` namespace: default messenger and report, time and metric helpers.
use super::{Gravity, MessageStream, Messenger, MetricType, Report};
use crate::c_format::{CArg, c_sprintf};
use crate::osd::MemCounter;
use crate::tcollection::AsciiString;
use std::sync::{Arc, Mutex, OnceLock};

/// Static tools of the OCCT `Message` class; `Message::default_messenger()`.
pub struct Message;

/// `Message_Table_PrintMetricTypeEnum`, indexed by `MetricType`.
const METRIC_NAMES: [&str; 13] = [
    "NONE",
    "ThreadCPUUserTime",
    "ThreadCPUSystemTime",
    "ProcessCPUUserTime",
    "ProcessCPUSystemTime",
    "WallClock",
    "MemPrivate",
    "MemVirtual",
    "MemWorkingSet",
    "MemWorkingSetPeak",
    "MemSwapUsage",
    "MemSwapUsagePeak",
    "MemHeapUsage",
];

/// Memory metrics and their `OSD_MemInfo` counters, in the native switch order.
const MEMORY_METRICS: [(MetricType, MemCounter); 7] = [
    (MetricType::MemPrivate, MemCounter::Private),
    (MetricType::MemVirtual, MemCounter::Virtual),
    (MetricType::MemWorkingSet, MemCounter::WorkingSet),
    (MetricType::MemWorkingSetPeak, MemCounter::WorkingSetPeak),
    (MetricType::MemSwapUsage, MemCounter::SwapUsage),
    (MetricType::MemSwapUsagePeak, MemCounter::SwapUsagePeak),
    (MetricType::MemHeapUsage, MemCounter::HeapUsage),
];

static DEFAULT_REPORT: Mutex<Option<Arc<Report>>> = Mutex::new(None);

impl Message {
    /// The process-wide messenger, initially printing to standard output;
    /// `Message::default_messenger().send_info_text(&text)`.
    pub fn default_messenger() -> &'static Arc<Messenger> {
        static MESSENGER: OnceLock<Arc<Messenger>> = OnceLock::new();
        MESSENGER.get_or_init(|| Arc::new(Messenger::new()))
    }
    /// `Send(Message_Gravity)` on the default messenger; `write!(Message::send(g), "text")`.
    pub fn send(gravity: Gravity) -> MessageStream<'static> {
        Self::default_messenger().stream(gravity)
    }
    /// `Send(const TCollection_AsciiString&, Message_Gravity)`; `Message::send_text(&text, g)`.
    pub fn send_text(text: &AsciiString, gravity: Gravity) {
        Self::default_messenger().send_ascii(text, gravity);
    }
    /// `SendFail()`; `write!(Message::send_fail(), "text")`.
    pub fn send_fail() -> MessageStream<'static> {
        Self::send(Gravity::Fail)
    }
    /// `SendAlarm()`; `write!(Message::send_alarm(), "text")`.
    pub fn send_alarm() -> MessageStream<'static> {
        Self::send(Gravity::Alarm)
    }
    /// `SendWarning()`; `write!(Message::send_warning(), "text")`.
    pub fn send_warning() -> MessageStream<'static> {
        Self::send(Gravity::Warning)
    }
    /// `SendInfo()`; `write!(Message::send_info(), "text")`.
    pub fn send_info() -> MessageStream<'static> {
        Self::send(Gravity::Info)
    }
    /// `SendTrace()`; `write!(Message::send_trace(), "text")`.
    pub fn send_trace() -> MessageStream<'static> {
        Self::send(Gravity::Trace)
    }
    /// `SendFail(const TCollection_AsciiString&)`; `Message::send_fail_text(&text)`.
    pub fn send_fail_text(text: &AsciiString) {
        Self::send_text(text, Gravity::Fail);
    }
    /// `SendAlarm(const TCollection_AsciiString&)`; `Message::send_alarm_text(&text)`.
    pub fn send_alarm_text(text: &AsciiString) {
        Self::send_text(text, Gravity::Alarm);
    }
    /// `SendWarning(const TCollection_AsciiString&)`; `Message::send_warning_text(&text)`.
    pub fn send_warning_text(text: &AsciiString) {
        Self::send_text(text, Gravity::Warning);
    }
    /// `SendInfo(const TCollection_AsciiString&)`; `Message::send_info_text(&text)`.
    pub fn send_info_text(text: &AsciiString) {
        Self::send_text(text, Gravity::Info);
    }
    /// `SendTrace(const TCollection_AsciiString&)`; `Message::send_trace_text(&text)`.
    pub fn send_trace_text(text: &AsciiString) {
        Self::send_text(text, Gravity::Trace);
    }
}

impl Message {
    /// `05h:12m:26.33s`, `06m:34.50s` or `4.50s`, with C `sprintf` rounding;
    /// `Message::fill_time(0, 0, 4.5).as_bytes() == b"4.50s"`.
    ///
    /// The native buffer holds 30 bytes; longer results (huge seconds) overflow it, which
    /// is undefined behavior; Rust returns the complete text.
    pub fn fill_time(hour: i32, minute: i32, second: f64) -> AsciiString {
        let text = if hour > 0 {
            c_sprintf(
                b"%02dh:%02dm:%.2fs",
                &[CArg::Int(hour), CArg::Int(minute), CArg::Real(second)],
            )
        } else if minute > 0 {
            c_sprintf(b"%02dm:%.2fs", &[CArg::Int(minute), CArg::Real(second)])
        } else {
            c_sprintf(b"%.2fs", &[CArg::Real(second)])
        };
        AsciiString::from_bytes(&text)
    }
    /// The process-wide report, created on request; `Message::default_report(true)`.
    pub fn default_report(to_create: bool) -> Option<Arc<Report>> {
        let mut report = DEFAULT_REPORT
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if report.is_none() && to_create {
            *report = Some(Arc::new(Report::new()));
        }
        report.clone()
    }
    /// The metric named exactly `text` (`MetricFromString(text, type)`);
    /// `Message::metric_from_string("WallClock") == Some(MetricType::WallClock)`.
    pub fn metric_from_string(text: &str) -> Option<MetricType> {
        let name = AsciiString::new(text);
        METRIC_NAMES
            .iter()
            .position(|candidate| name.is_equal(candidate.as_bytes()))
            .map(|index| MetricType::ALL[index])
    }
    /// `MetricFromString(text)`: the metric, or `MetricType::None` for unknown text;
    /// `Message::metric_from_string_or_none("x") == MetricType::None`.
    pub fn metric_from_string_or_none(text: &str) -> MetricType {
        Self::metric_from_string(text).unwrap_or(MetricType::None)
    }
    /// The native name of a metric; `Message::metric_to_string(MetricType::WallClock) == "WallClock"`.
    pub fn metric_to_string(metric: MetricType) -> &'static str {
        METRIC_NAMES[metric as usize]
    }
    /// The memory counter of a memory metric; `Message::to_osd_metric(MetricType::MemVirtual)`.
    pub fn to_osd_metric(metric: MetricType) -> Option<MemCounter> {
        MEMORY_METRICS
            .iter()
            .find(|(candidate, _)| *candidate == metric)
            .map(|(_, counter)| *counter)
    }
    /// The memory metric of a counter; `Message::to_message_metric(MemCounter::Virtual)`.
    pub fn to_message_metric(counter: MemCounter) -> Option<MetricType> {
        MEMORY_METRICS
            .iter()
            .find(|(_, candidate)| *candidate == counter)
            .map(|(metric, _)| *metric)
    }
}
