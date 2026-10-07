// Rust port of DataExchange/TKXSBase/Interface/Interface_Check.cxx (Complies, Remove,
// Mend, GetMessages, GetAsWarning, Print, Trace) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The edits and printing of an `Interface_Check`.
use super::check::{CheckState, MessageKind, MessageList};
use super::{InterfaceCheck, InterfaceCheckStatus};
use rustycad_tkernel::{
    message::Message,
    standard::{Failure, Handle, exceptions},
    tcollection::HAsciiString,
};
use std::fmt::Write as _;
use std::io::{self, Write};

impl InterfaceCheck {
    /// `Complies(status)`: whether the check has this status (or matches the criterion).
    pub fn complies(&self, status: InterfaceCheckStatus) -> bool {
        let state = self.state.read();
        let now = state.warns.0.is_none();
        let nof = state.fails.0.is_none();
        match status {
            InterfaceCheckStatus::CheckOK => nof && now,
            InterfaceCheckStatus::CheckWarning => nof && !now,
            InterfaceCheckStatus::CheckFail => !nof,
            InterfaceCheckStatus::CheckAny => true,
            InterfaceCheckStatus::CheckMessage => !nof || !now,
            InterfaceCheckStatus::CheckNoFail => nof,
        }
    }
    /// `Complies(mess, incl, status)`: whether a message equals (`incl` 0) or contains
    /// (`incl` < 0) `mess`. Native quirks kept: the fail criterion also scans the
    /// warnings, and `incl` < 0 matches whenever `mess` is not empty (it is searched in
    /// itself).
    pub fn complies_message(
        &self,
        mess: &Handle<HAsciiString>,
        incl: i32,
        status: InterfaceCheckStatus,
    ) -> Result<bool, Failure> {
        let Some(text) = mess.get() else {
            return Ok(false);
        };
        let scans = scanned_lists(status);
        for _ in 0..scans {
            for i in (1..=self.nb_warnings()).rev() {
                let ames = self.warning(i, true)?;
                if message_matches(text, mess, &ames, incl)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    /// `Remove(mess, incl, status)`: removes the matching messages. Native quirk kept: the
    /// fail criterion scans the warnings and removes the fails of the same ranks (a
    /// missing rank raises `Standard_OutOfRange`, a null fail list `Standard_NullObject`).
    pub fn remove(
        &self,
        mess: &Handle<HAsciiString>,
        incl: i32,
        status: InterfaceCheckStatus,
    ) -> Result<bool, Failure> {
        let Some(text) = mess.get() else {
            return Ok(false);
        };
        let mut res = false;
        let kinds = [
            (
                matches!(
                    status,
                    InterfaceCheckStatus::CheckWarning | InterfaceCheckStatus::CheckAny
                ),
                MessageKind::Warning,
            ),
            (
                matches!(
                    status,
                    InterfaceCheckStatus::CheckFail | InterfaceCheckStatus::CheckAny
                ),
                MessageKind::Fail,
            ),
        ];
        for (_, removed) in kinds.into_iter().filter(|(on, _)| *on) {
            for i in (1..=self.nb_warnings()).rev() {
                let ames = self.warning(i, true)?;
                if message_rem(text, mess, &ames, incl)? {
                    remove_both(&mut self.state.write(), removed, i)?;
                    res = true;
                }
            }
        }
        Ok(res)
    }
    /// `Mend(pref, num = 0)`: "FM" mends with "Mended"; "CA" clears all; "CW"/"CF" clear
    /// one warning/fail (all for 0); else prefixes the fail `num` (all for 0) with
    /// "pref : " and moves it to the end as a warning would not.
    pub fn mend(&self, pref: Option<&str>, num: i32) -> Result<bool, Failure> {
        if let Some(code) = pref.filter(|p| p.len() == 2)
            && let Some(done) = self.mend_code(code, num)?
        {
            return Ok(done);
        }
        if num == 0 {
            for i in (1..=self.nb_fails()).rev() {
                self.mend(pref, i)?;
            }
            return Ok(true);
        }
        if num < 0 || num > self.nb_fails() {
            return Ok(false);
        }
        self.mend_one(pref, num)?;
        Ok(true)
    }
    /// The two-letter commands of `Mend`; `None` for another text.
    fn mend_code(&self, code: &str, num: i32) -> Result<Option<bool>, Failure> {
        let kind = match code {
            "FM" => return self.mend(Some("Mended"), num).map(Some),
            "CA" => {
                self.clear();
                return Ok(Some(true));
            }
            "CW" => MessageKind::Warning,
            "CF" => MessageKind::Fail,
            _ => return Ok(None),
        };
        let mut state = self.state.write();
        if num == 0 {
            *state.lists_mut(kind) = (None, None);
            return Ok(Some(true));
        }
        if num < 0 || num > state.count(kind) {
            return Ok(Some(false));
        }
        remove_both(&mut state, kind, num)?;
        Ok(Some(true))
    }
    fn mend_one(&self, pref: Option<&str>, num: i32) -> Result<(), Failure> {
        let strf = self.fail(num, true)?;
        let stro = self.fail(num, false)?;
        if let Some(prefix) = pref.filter(|p| !p.is_empty()) {
            strf.insert(1, " : ")?;
            strf.insert(1, prefix)?;
            if stro != strf {
                stro.insert(1, " : ")?;
                stro.insert(1, prefix)?;
            }
        }
        let mut state = self.state.write();
        remove_both(&mut state, MessageKind::Fail, num)?;
        state.append(MessageKind::Fail, &strf, &stro);
        Ok(())
    }
    /// `GetMessages(other)`: appends the fails, warnings and infos of `other`.
    pub fn get_messages(&self, other: &InterfaceCheck) {
        let source = other.state.read().clone();
        let mut state = self.state.write();
        for kind in [MessageKind::Fail, MessageKind::Warning, MessageKind::Info] {
            append_lists(state.lists_mut(kind), source.lists(kind));
        }
    }
    /// `GetAsWarning(other, failsonly)`: appends the fails of `other` as warnings, and its
    /// warnings unless `failsonly`. Native quirk kept: without `failsonly`, the warning
    /// lists are created even when `other` has no warning.
    pub fn get_as_warning(&self, other: &InterfaceCheck, failsonly: bool) {
        let source = other.state.read().clone();
        let mut state = self.state.write();
        append_lists(&mut state.warns, &source.fails);
        if source.count(MessageKind::Warning) != 0 || !failsonly {
            let warns = &mut state.warns;
            warns.0.get_or_insert_with(Vec::new);
            warns.1.get_or_insert_with(Vec::new);
            append_lists(warns, &source.warns);
        }
    }
    /// `Print(S, level, final = 1)`: fails from level 1, warnings from level 2, infos from
    /// level 0; the final texts for `final` >= 0, else the originals.
    pub fn print(&self, s: &mut dyn Write, level: i32, final_form: i32) -> io::Result<()> {
        let state = self.state.read().clone();
        let use_final = final_form >= 0;
        let kinds = [
            (1, MessageKind::Fail),
            (2, MessageKind::Warning),
            (0, MessageKind::Info),
        ];
        for (minimum, kind) in kinds {
            if level < minimum {
                continue;
            }
            let (finals, origs) = state.lists(kind);
            let list = if use_final { finals } else { origs };
            for text in list.iter().flatten() {
                s.write_all(text.string().as_bytes())?;
                s.write_all(b"\n")?;
            }
        }
        Ok(())
    }
    /// `Trace(level = -1, final = 1)`: `Print` on the info stream of the default messenger.
    pub fn trace(&self, level: i32, final_form: i32) {
        let mut text = Vec::new();
        // Writing into a Vec cannot fail.
        let _ = self.print(&mut text, level, final_form);
        let mut stream = Message::send_info();
        let _ = stream.write_str(&String::from_utf8_lossy(&text));
    }
}

/// How many times `Complies` scans the warnings for a criterion (once for warning or
/// fail, twice for any).
fn scanned_lists(status: InterfaceCheckStatus) -> usize {
    let warn = matches!(
        status,
        InterfaceCheckStatus::CheckWarning | InterfaceCheckStatus::CheckAny
    );
    let fail = matches!(
        status,
        InterfaceCheckStatus::CheckFail | InterfaceCheckStatus::CheckAny
    );
    usize::from(warn) + usize::from(fail)
}

/// The match test of `Complies` (any of the three tests).
fn message_matches(
    text: &HAsciiString,
    mess: &Handle<HAsciiString>,
    ames: &Handle<HAsciiString>,
    incl: i32,
) -> Result<bool, Failure> {
    if incl == 0 && text.is_same_string(ames)? {
        return Ok(true);
    }
    if incl < 0 {
        let length = ames.length();
        if ames.location(text, 1, length)? > 0 || text.location(mess, 1, text.length())? > 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The match test of `Remove`: the last test that applies decides, as natively.
fn message_rem(
    text: &HAsciiString,
    mess: &Handle<HAsciiString>,
    ames: &Handle<HAsciiString>,
    incl: i32,
) -> Result<bool, Failure> {
    let mut rem = false;
    if incl == 0 {
        rem = text.is_same_string(ames)?;
    }
    if incl < 0 {
        // Evaluated for its failures only: the next test overwrites the result.
        ames.location(text, 1, ames.length())?;
        rem = text.location(mess, 1, text.length())? > 0;
    }
    Ok(rem)
}

/// Removes the message `num` of a kind from both lists (`Remove(num)` of the sequences).
fn remove_both(state: &mut CheckState, kind: MessageKind, num: i32) -> Result<(), Failure> {
    let (finals, origs) = state.lists_mut(kind);
    remove_at(finals, num)?;
    remove_at(origs, num)
}

fn remove_at(list: &mut MessageList, num: i32) -> Result<(), Failure> {
    let live = list.as_mut().ok_or_else(|| {
        exceptions::null_object(
            "Interface_Check::Remove: null message list; expected a created list",
        )
    })?;
    let index = usize::try_from(num - 1).ok().filter(|i| *i < live.len());
    let index = index.ok_or_else(|| {
        exceptions::out_of_range(&format!(
            "NCollection_Sequence::Remove: index {num}; expected [1, {}]",
            live.len()
        ))
    })?;
    live.remove(index);
    Ok(())
}

/// Appends the messages of `source` (both forms) when it has some.
fn append_lists(target: &mut (MessageList, MessageList), source: &(MessageList, MessageList)) {
    let Some(finals) = source.0.as_ref().filter(|l| !l.is_empty()) else {
        return;
    };
    target
        .0
        .get_or_insert_with(Vec::new)
        .extend(finals.iter().cloned());
    let origs = source.1.clone().unwrap_or_default();
    target.1.get_or_insert_with(Vec::new).extend(origs);
}
