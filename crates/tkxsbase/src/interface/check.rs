// Rust port of DataExchange/TKXSBase/Interface/Interface_Check (.hxx, .cxx: messages and
// queries) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_Check`: the fails, warnings and information messages about an entity, each
//! kept in its final form and in its original form. The edits (`Complies`, `Remove`,
//! `Mend`, `GetMessages`, `Print`) are in `check_edit`.
use super::InterfaceCheckStatus;
use rustycad_tkernel::{
    impl_transient,
    message::Msg,
    standard::{Failure, Handle, SharedState, StandardTransient, exceptions},
    tcollection::{AsciiString, ExtendedString, HAsciiString},
};

/// A list of messages; `None` is the native null sequence (never created).
pub(super) type MessageList = Option<Vec<Handle<HAsciiString>>>;

/// The three kinds of messages of a check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MessageKind {
    Fail,
    Warning,
    Info,
}

/// The fields of a check: (final, original) lists of each kind and the entity.
#[derive(Clone, Default)]
pub(super) struct CheckState {
    pub(super) fails: (MessageList, MessageList),
    pub(super) warns: (MessageList, MessageList),
    pub(super) infos: (MessageList, MessageList),
    pub(super) ent: Handle<StandardTransient>,
}

impl CheckState {
    pub(super) fn lists(&self, kind: MessageKind) -> &(MessageList, MessageList) {
        match kind {
            MessageKind::Fail => &self.fails,
            MessageKind::Warning => &self.warns,
            MessageKind::Info => &self.infos,
        }
    }
    pub(super) fn lists_mut(&mut self, kind: MessageKind) -> &mut (MessageList, MessageList) {
        match kind {
            MessageKind::Fail => &mut self.fails,
            MessageKind::Warning => &mut self.warns,
            MessageKind::Info => &mut self.infos,
        }
    }
    /// Appends a message (creating both lists when null, as natively).
    pub(super) fn append(
        &mut self,
        kind: MessageKind,
        mess: &Handle<HAsciiString>,
        orig: &Handle<HAsciiString>,
    ) {
        let (finals, origs) = self.lists_mut(kind);
        finals.get_or_insert_with(Vec::new).push(mess.clone());
        origs.get_or_insert_with(Vec::new).push(orig.clone());
    }
    pub(super) fn count(&self, kind: MessageKind) -> i32 {
        self.lists(kind).0.as_ref().map_or(0, |l| l.len() as i32)
    }
}

/// `Interface_Check`.
///
/// ```
/// use rustycad_tkxsbase::interface::{InterfaceCheck, InterfaceCheckStatus};
/// let check = InterfaceCheck::new();
/// check.add_fail_text("Bad value", "Bad value %d");
/// assert_eq!(check.status(), InterfaceCheckStatus::CheckFail);
/// assert_eq!(check.c_fail(1, false).unwrap().as_bytes(), b"Bad value %d");
/// ```
#[derive(Default)]
pub struct InterfaceCheck {
    transient: StandardTransient,
    pub(super) state: SharedState<CheckState>,
}
impl_transient!(
    InterfaceCheck,
    "Interface_Check",
    StandardTransient,
    transient
);

fn h_text(text: &[u8]) -> Handle<HAsciiString> {
    Handle::new(HAsciiString::new(text))
}

fn ascii(text: &ExtendedString) -> Handle<HAsciiString> {
    let mut converted = AsciiString::default();
    converted.assign_cat_extended(text, None);
    Handle::new(HAsciiString::from(converted))
}

impl InterfaceCheck {
    /// `Interface_Check()`: empty, no entity.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Interface_Check(anentity)`.
    pub fn with_entity(anentity: &Handle<StandardTransient>) -> Self {
        let check = Self::default();
        check.state.write().ent = anentity.clone();
        check
    }
    /// `SendFail(amsg)`: `AddFail(amsg)`.
    pub fn send_fail(&self, amsg: &Msg) {
        self.add_fail_msg(amsg);
    }
    /// `AddFail(mess)`: the same message as final and original.
    pub fn add_fail(&self, mess: &Handle<HAsciiString>) {
        self.state.write().append(MessageKind::Fail, mess, mess);
    }
    /// `AddFail(mess, orig)`.
    pub fn add_fail_with_original(&self, mess: &Handle<HAsciiString>, orig: &Handle<HAsciiString>) {
        self.state.write().append(MessageKind::Fail, mess, orig);
    }
    /// `AddFail(amess, orig = "")`: nothing for an empty message.
    pub fn add_fail_text(&self, amess: &str, orig: &str) {
        self.add_text(MessageKind::Fail, amess, orig);
    }
    /// `AddFail(const Message_Msg&)`: the edited value with its original, else the value.
    pub fn add_fail_msg(&self, amsg: &Msg) {
        self.add_msg(MessageKind::Fail, amsg);
    }
    /// `HasFailed`: a fail list exists (it may be emptied by `Remove`).
    pub fn has_failed(&self) -> bool {
        self.state.read().fails.0.is_some()
    }
    /// `NbFails`.
    pub fn nb_fails(&self) -> i32 {
        self.state.read().count(MessageKind::Fail)
    }
    /// `Fail(num, final = true)`: `Standard_OutOfRange` out of range.
    pub fn fail(&self, num: i32, final_form: bool) -> Result<Handle<HAsciiString>, Failure> {
        self.message(MessageKind::Fail, num, final_form)
    }
    /// `CFail(num, final = true)`.
    pub fn c_fail(&self, num: i32, final_form: bool) -> Result<AsciiString, Failure> {
        Ok(self.fail(num, final_form)?.string().clone())
    }
    /// `Fails(final = true)`: a copy of the list (empty when none).
    pub fn fails(&self, final_form: bool) -> Vec<Handle<HAsciiString>> {
        self.messages(MessageKind::Fail, final_form)
    }
    /// `SendWarning(amsg)`.
    pub fn send_warning(&self, amsg: &Msg) {
        self.add_warning_msg(amsg);
    }
    /// `AddWarning(mess)`.
    pub fn add_warning(&self, mess: &Handle<HAsciiString>) {
        self.state.write().append(MessageKind::Warning, mess, mess);
    }
    /// `AddWarning(mess, orig)`.
    pub fn add_warning_with_original(
        &self,
        mess: &Handle<HAsciiString>,
        orig: &Handle<HAsciiString>,
    ) {
        self.state.write().append(MessageKind::Warning, mess, orig);
    }
    /// `AddWarning(amess, orig = "")`: nothing for an empty message.
    pub fn add_warning_text(&self, amess: &str, orig: &str) {
        self.add_text(MessageKind::Warning, amess, orig);
    }
    /// `AddWarning(const Message_Msg&)`.
    pub fn add_warning_msg(&self, amsg: &Msg) {
        self.add_msg(MessageKind::Warning, amsg);
    }
    /// `HasWarnings`.
    pub fn has_warnings(&self) -> bool {
        self.state.read().warns.0.is_some()
    }
    /// `NbWarnings`.
    pub fn nb_warnings(&self) -> i32 {
        self.state.read().count(MessageKind::Warning)
    }
    /// `Warning(num, final = true)`.
    pub fn warning(&self, num: i32, final_form: bool) -> Result<Handle<HAsciiString>, Failure> {
        self.message(MessageKind::Warning, num, final_form)
    }
    /// `CWarning(num, final = true)`.
    pub fn c_warning(&self, num: i32, final_form: bool) -> Result<AsciiString, Failure> {
        Ok(self.warning(num, final_form)?.string().clone())
    }
    /// `Warnings(final = true)`.
    pub fn warnings(&self, final_form: bool) -> Vec<Handle<HAsciiString>> {
        self.messages(MessageKind::Warning, final_form)
    }
    /// `SendMsg(amsg)`: an information message. Native quirk kept: the original list
    /// gets the final text too, even for an edited message.
    pub fn send_msg(&self, amsg: &Msg) {
        let mess = ascii(amsg.value());
        self.state.write().append(MessageKind::Info, &mess, &mess);
    }
    /// `NbInfoMsgs`.
    pub fn nb_info_msgs(&self) -> i32 {
        self.state.read().count(MessageKind::Info)
    }
    /// `InfoMsg(num, final = true)`.
    pub fn info_msg(&self, num: i32, final_form: bool) -> Result<Handle<HAsciiString>, Failure> {
        self.message(MessageKind::Info, num, final_form)
    }
    /// `CInfoMsg(num, final = true)`.
    pub fn c_info_msg(&self, num: i32, final_form: bool) -> Result<AsciiString, Failure> {
        Ok(self.info_msg(num, final_form)?.string().clone())
    }
    /// `InfoMsgs(final = true)`.
    pub fn info_msgs(&self, final_form: bool) -> Vec<Handle<HAsciiString>> {
        self.messages(MessageKind::Info, final_form)
    }
    /// `Status`: fail, else warning, else OK.
    pub fn status(&self) -> InterfaceCheckStatus {
        let state = self.state.read();
        if state.fails.0.is_some() {
            return InterfaceCheckStatus::CheckFail;
        }
        if state.warns.0.is_some() {
            return InterfaceCheckStatus::CheckWarning;
        }
        InterfaceCheckStatus::CheckOK
    }
    /// `HasEntity`.
    pub fn has_entity(&self) -> bool {
        self.state.read().ent.is_some()
    }
    /// `Entity`.
    pub fn entity(&self) -> Handle<StandardTransient> {
        self.state.read().ent.clone()
    }
    /// `SetEntity`.
    pub fn set_entity(&self, anentity: &Handle<StandardTransient>) {
        self.state.write().ent = anentity.clone();
    }
    /// `GetEntity`: sets the entity only when none is set.
    pub fn get_entity(&self, anentity: &Handle<StandardTransient>) {
        let mut state = self.state.write();
        if state.ent.is_null() {
            state.ent = anentity.clone();
        }
    }
    /// `Clear`: no message, no entity.
    pub fn clear(&self) {
        *self.state.write() = CheckState::default();
    }
    /// `ClearFails`.
    pub fn clear_fails(&self) {
        self.state.write().fails = (None, None);
    }
    /// `ClearWarnings`.
    pub fn clear_warnings(&self) {
        self.state.write().warns = (None, None);
    }
    /// `ClearInfoMsgs`.
    pub fn clear_info_msgs(&self) {
        self.state.write().infos = (None, None);
    }
    fn add_text(&self, kind: MessageKind, amess: &str, orig: &str) {
        if amess.is_empty() {
            return;
        }
        let mess = h_text(amess.as_bytes());
        let orig = if orig.is_empty() {
            mess.clone()
        } else {
            h_text(orig.as_bytes())
        };
        self.state.write().append(kind, &mess, &orig);
    }
    fn add_msg(&self, kind: MessageKind, amsg: &Msg) {
        let mess = ascii(amsg.value());
        let orig = if amsg.is_edited() {
            ascii(amsg.original())
        } else {
            mess.clone()
        };
        self.state.write().append(kind, &mess, &orig);
    }
    /// The message `num` of a kind: `Standard_OutOfRange` for a null list or a rank out
    /// of it.
    pub(super) fn message(
        &self,
        kind: MessageKind,
        num: i32,
        final_form: bool,
    ) -> Result<Handle<HAsciiString>, Failure> {
        let state = self.state.read();
        let (finals, origs) = state.lists(kind);
        let list = if final_form { finals } else { origs };
        let found = list
            .as_ref()
            .and_then(|l| usize::try_from(num - 1).ok().and_then(|i| l.get(i)));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Interface_Check: {kind:?} message {num}; expected [1, {}]",
                state.count(kind)
            ))
        })
    }
    fn messages(&self, kind: MessageKind, final_form: bool) -> Vec<Handle<HAsciiString>> {
        let state = self.state.read();
        let (finals, origs) = state.lists(kind);
        let list = if final_form { finals } else { origs };
        list.clone().unwrap_or_default()
    }
}
