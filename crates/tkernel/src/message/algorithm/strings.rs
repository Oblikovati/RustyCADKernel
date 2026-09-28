//@header Message_Algorithm.lxx
//! The string overloads of `Message_Algorithm::SetStatus` and the recorded strings.
use super::{Algorithm, status_slots};
use crate::message::{ExecStatus, Status};
use crate::standard::Handle;
use crate::tcollection::{AsciiString, ExtendedString, HAsciiString, HExtendedString};

impl Algorithm {
    /// `SetStatus(stat, const char*)`: bytes widened one per unit (no UTF-8 decoding);
    /// `algorithm.set_status_with_c_str(s, b"x", true)`.
    pub fn set_status_with_c_str(&mut self, status: Status, text: &[u8], no_repetitions: bool) {
        let text = Handle::new(HExtendedString::from_c_string(text));
        self.set_status_with_h_extended(status, &text, no_repetitions);
    }
    /// `SetStatus(stat, const TCollection_AsciiString&)`: decoded as UTF-8 when valid;
    /// `algorithm.set_status_with_ascii(s, &text, true)`.
    pub fn set_status_with_ascii(
        &mut self,
        status: Status,
        text: &AsciiString,
        no_repetitions: bool,
    ) {
        let text = Handle::new(HExtendedString::from(ExtendedString::from_ascii(text)));
        self.set_status_with_h_extended(status, &text, no_repetitions);
    }
    /// `SetStatus(stat, const occ::handle<TCollection_HAsciiString>&)`: decoded as UTF-8 when
    /// valid; `algorithm.set_status_with_h_ascii(s, &text, true)`.
    pub fn set_status_with_h_ascii(
        &mut self,
        status: Status,
        text: &HAsciiString,
        no_repetitions: bool,
    ) {
        let text = Handle::new(HExtendedString::from_h_ascii_string(text));
        self.set_status_with_h_extended(status, &text, no_repetitions);
    }
    /// `SetStatus(stat, const TCollection_ExtendedString&)`; `algorithm.set_status_with_extended(s, &text, true)`.
    pub fn set_status_with_extended(
        &mut self,
        status: Status,
        text: &ExtendedString,
        no_repetitions: bool,
    ) {
        let text = Handle::new(HExtendedString::from(text.clone()));
        self.set_status_with_h_extended(status, &text, no_repetitions);
    }
    /// Raises a flag and shares the string handle for it (a null handle records nothing),
    /// optionally skipping a handle whose string is equal;
    /// `algorithm.set_status_with_h_extended(s, &text, true)`.
    pub fn set_status_with_h_extended(
        &mut self,
        status: Status,
        text: &Handle<HExtendedString>,
        no_repetitions: bool,
    ) {
        self.set_status(status);
        let index = ExecStatus::status_index(status) as usize;
        if text.is_null() || index == 0 {
            return;
        }
        let slots = self.report_strings.get_or_insert_with(status_slots);
        let strings = slots[index].get_or_insert_with(Vec::new);
        let value = text.string().clone();
        if no_repetitions
            && strings
                .iter()
                .any(|existing| existing.string().is_equal(&value))
        {
            return;
        }
        strings.push(text.clone());
    }
    /// The string handles recorded for a status; `algorithm.get_message_strings(Status::FAIL1)`.
    pub fn get_message_strings(&self, status: Status) -> Option<&[Handle<HExtendedString>]> {
        let index = ExecStatus::status_index(status) as usize;
        self.report_strings
            .as_ref()?
            .get(index)?
            .as_deref()
            .filter(|_| index != 0)
    }
}
