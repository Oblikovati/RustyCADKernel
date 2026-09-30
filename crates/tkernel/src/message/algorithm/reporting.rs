//! Status messages of `Message_Algorithm`: naming, report text and status merging.
use super::Algorithm;
use crate::message::{ExecStatus, Gravity, Msg, MsgFile, Status, StatusType};
use crate::ncollection::{PackedMap, packed_map_algo};
use crate::standard::Failure;
use crate::standard::Handle;
use crate::tcollection::{AsciiString, ExtendedString, HExtendedString};

/// `".Done<n>"`-style suffix of a status, or `None` outside the four groups.
fn status_suffix(status: Status) -> Option<AsciiString> {
    let group = match ExecStatus::type_of_status(status) {
        StatusType::DONE => ".Done",
        StatusType::WARN => ".Warn",
        StatusType::ALARM => ".Alarm",
        StatusType::FAIL => ".Fail",
        _ => return None,
    };
    let mut suffix = AsciiString::new(group);
    suffix.assign_cat(ExecStatus::local_status_index(status));
    Some(suffix)
}

/// Every status index with its status, `FirstStatus..=LastStatus`.
fn all_statuses() -> impl Iterator<Item = (usize, Status)> {
    (ExecStatus::FIRST_STATUS..=ExecStatus::LAST_STATUS)
        .map(|index| (index as usize, ExecStatus::status_by_index(index)))
}

impl Algorithm {
    /// The first `<Type><suffix>` key known to the catalog walking up the type chain; the
    /// root type's key when none is known.
    fn message_name(&self, suffix: &AsciiString) -> AsciiString {
        let mut name = AsciiString::default();
        let mut kind = Some(self.dynamic_type());
        while let Some(current) = kind {
            name = AsciiString::new(current.name());
            name.append(suffix);
            if MsgFile::global().has_msg(&name) {
                break;
            }
            kind = current.parent();
        }
        name
    }
    /// The catalog message of one status with its recorded integers and strings.
    fn status_message(&self, status: Status, max_count: i32) -> Result<Option<Msg>, Failure> {
        let Some(suffix) = status_suffix(status) else {
            return Ok(None);
        };
        let mut message = Msg::from_key(&self.message_name(&suffix));
        if let Some(numbers) = self.get_message_numbers(status) {
            message.arg_extended(&Self::prepare_report_numbers(&numbers, max_count))?;
        }
        if let Some(strings) = self.get_message_strings(status) {
            message.arg_extended(&Self::prepare_report_strings(&strings, max_count))?;
        }
        Ok(Some(message))
    }
    /// Sends one message per status set both in `filter` and here: the custom message,
    /// else the catalog message `<Type>.<Group><n>` filled with the recorded data;
    /// `algorithm.send_status_messages(&filter, Gravity::Warning, 20)?`.
    ///
    /// Native quirk kept: a custom message is sent through `Get()`, which rewrites it in
    /// place, so sending twice inserts `UNKNOWN` again for unfilled arguments.
    pub fn send_status_messages(
        &self,
        filter: &ExecStatus,
        trace_level: Gravity,
        max_count: i32,
    ) -> Result<(), Failure> {
        let messenger = self.get_messenger();
        for (index, status) in all_statuses() {
            if !filter.is_set(status) || !self.get_status().is_set(status) {
                continue;
            }
            if let Some(text) = self.status_text(index, status, max_count)? {
                messenger.send_extended(&text, trace_level);
            }
        }
        Ok(())
    }
    fn status_text(
        &self,
        index: usize,
        status: Status,
        max_count: i32,
    ) -> Result<Option<ExtendedString>, Failure> {
        let custom = self
            .report_messages
            .read()
            .as_ref()
            .and_then(|slots| slots[index].clone());
        if let Some(custom) = custom {
            let mut message = custom
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            return Ok(Some(message.get()?.clone()));
        }
        self.status_message(status, max_count)?
            .map(|mut message| message.get().cloned())
            .transpose()
    }
    /// Sends the warn, alarm and fail messages; `algorithm.send_messages(Gravity::Warning, 20)?`.
    pub fn send_messages(&self, trace_level: Gravity, max_count: i32) -> Result<(), Failure> {
        let mut filter = ExecStatus::new();
        filter.set_all_warn();
        filter.set_all_alarm();
        filter.set_all_fail();
        self.send_status_messages(&filter, trace_level, max_count)
    }
    /// Copies every status of `other` with its data; `algorithm.add_status(&other)`.
    pub fn add_status(&self, other: &Algorithm) {
        let allowed = other.get_status();
        self.add_status_filtered(&allowed, other);
    }
    /// Copies the statuses of `other` allowed by the filter, uniting integers and adding
    /// strings; `algorithm.add_status_filtered(&allowed, &other)`.
    ///
    /// Native quirk kept: the string loop runs `n < Length()`, so the last string of each
    /// status is not copied.
    pub fn add_status_filtered(&self, allowed: &ExecStatus, other: &Algorithm) {
        for (index, status) in all_statuses() {
            if !allowed.is_set(status) || !other.get_status().is_set(status) {
                continue;
            }
            self.set_status(status);
            self.merge_status_numbers(index, status, other);
            let strings = other.get_message_strings(status).unwrap_or_default();
            for text in strings.iter().take(strings.len().saturating_sub(1)) {
                self.set_status_with_h_extended(status, text, true);
            }
        }
    }
    fn merge_status_numbers(&self, index: usize, status: Status, other: &Algorithm) {
        let Some(numbers) = other.get_message_numbers(status) else {
            return;
        };
        let mut recorded = self.report_integers.write();
        let slots = recorded.get_or_insert_with(super::status_slots);
        packed_map_algo::unite(
            slots[index].get_or_insert_with(PackedMap::default),
            &numbers,
        );
    }
}

impl Algorithm {
    /// Up to `max_count` integers separated by spaces, then ` ... (total N)` when more
    /// remain; `Algorithm::prepare_report_numbers(&map, 20)`.
    pub fn prepare_report_numbers(numbers: &PackedMap<i32>, max_count: i32) -> ExtendedString {
        let mut report = ExtendedString::default();
        let mut values = numbers.iter().peekable();
        let mut count = 1;
        while count <= max_count {
            let Some(value) = values.next() else {
                break;
            };
            if count > 1 {
                report.assign_cat(ExtendedString::new(" "));
            }
            report.assign_cat(value);
            count += 1;
        }
        if values.peek().is_some() {
            report.assign_cat(ExtendedString::new(" ... (total "));
            report.assign_cat(numbers.extent());
            report.assign_cat(ExtendedString::new(")"));
        }
        report
    }
    /// Up to `max_count` quoted strings separated by `, `, then ` ... (total N) ` when the
    /// sequence is longer; `Algorithm::prepare_report_strings(&strings, 20)`.
    pub fn prepare_report_strings(
        strings: &[Handle<HExtendedString>],
        max_count: i32,
    ) -> ExtendedString {
        let mut report = ExtendedString::default();
        for (number, text) in strings.iter().enumerate().take(max_count.max(0) as usize) {
            report.assign_cat(ExtendedString::new(if number > 0 { ", '" } else { "'" }));
            report.assign_cat(&*text.string());
            report.assign_cat(ExtendedString::new("'"));
        }
        if strings.len() as i64 > i64::from(max_count) {
            report.assign_cat(ExtendedString::new(" ... (total "));
            report.assign_cat(strings.len() as i32);
            report.assign_cat(ExtendedString::new(") "));
        }
        report
    }
}
