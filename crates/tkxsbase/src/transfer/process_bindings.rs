// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ProcessForTransient_0.cxx and
// Transfer_ProcessForFinder_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871: the messages,
// the transient results, the mapped objects and the roots.

//! The messages, results, mapped objects and roots of [`TransferProcess`].
use super::{
    TransferBinder, TransferMultipleBinder, TransferProcess, TransferProcessKind,
    TransferSimpleBinderOfTransient, TransferStatusResult, transfer_failure,
};
use crate::interface::InterfaceCheck;
use rustycad_tkernel::{
    message::{Gravity, Msg},
    standard::{Failure, Handle, RuntimeType, RuntimeTyped, StandardTransient},
    tcollection::ExtendedString,
};
use std::fmt::Write as _;

/// A message text as `TCollection_AsciiString(ext).ToCString()`: UTF-8 up to the first NUL.
fn c_text_of(text: &ExtendedString) -> String {
    let utf8 = text.to_utf8();
    utf8.split('\0').next().unwrap_or_default().to_owned()
}

impl<K: TransferProcessKind> TransferProcess<K> {
    /// The binder of `start`, else a new void binder bound to it (the head of the
    /// native message members).
    fn binder_else_void(&self, start: &K::Start) -> Handle<dyn TransferBinder> {
        let binder = self.find_and_mask(start);
        if binder.is_some() {
            return binder;
        }
        self.bind_void(start)
    }
    /// Sends one line of the trace to the messenger, with the native " [from: orig]"
    /// suffix at trace level 3 when there is an origin.
    fn send_trace_line(&self, gravity: Gravity, head: &str, from: Option<&str>) {
        let messenger = self.messenger();
        let mut line = String::from(head);
        if let Some(orig) = from.filter(|_| self.trace_level() > 2) {
            let _ = write!(line, " [from: {orig}]");
        }
        line.push('\n');
        messenger.send_string_stream(line.as_bytes(), gravity);
    }
    /// `SendFail(start, amsg)`: `AddFail(start, amsg)`.
    pub fn send_fail(&self, start: &K::Start, amsg: &Msg) {
        self.add_fail_msg(start, amsg);
    }
    /// `SendWarning(start, amsg)`: `AddWarning(start, amsg)`.
    pub fn send_warning(&self, start: &K::Start, amsg: &Msg) {
        self.add_warning_msg(start, amsg);
    }
    /// `SendMsg(start, amsg)`: binds `start` when needed and traces the message (trace
    /// level 1 and more); the binder check does not record it.
    pub fn send_msg(&self, start: &K::Start, amsg: &Msg) {
        let binder = self.binder_else_void(start);
        if self.trace_level() <= 0 {
            return;
        }
        self.start_trace(&binder, start, self.nesting_level(), 6);
        let orig = amsg.original().to_string();
        let head = amsg.value().to_string();
        self.send_trace_line(
            Gravity::Info,
            &head,
            amsg.is_edited().then_some(orig.as_str()),
        );
    }
    /// `AddFail(start, mess, orig = "")`: a fail on the binder of `start`, traced from
    /// level 1.
    pub fn add_fail_text(&self, start: &K::Start, mess: &str, orig: &str) {
        let binder = self.binder_else_void(start);
        binder.add_fail(mess, orig);
        if self.trace_level() <= 0 {
            return;
        }
        self.start_trace(&binder, start, self.nesting_level(), 1);
        let head = format!("    --> Fail : {mess}");
        self.send_trace_line(Gravity::Fail, &head, (!orig.is_empty()).then_some(orig));
    }
    /// `AddError(start, mess, orig = "")`: `AddFail`.
    pub fn add_error(&self, start: &K::Start, mess: &str, orig: &str) {
        self.add_fail_text(start, mess, orig);
    }
    /// `AddFail(start, amsg)`: the edited value with its original, else the value.
    pub fn add_fail_msg(&self, start: &K::Start, amsg: &Msg) {
        let orig = if amsg.is_edited() {
            c_text_of(amsg.original())
        } else {
            String::new()
        };
        self.add_fail_text(start, &c_text_of(amsg.value()), &orig);
    }
    /// `AddWarning(start, mess, orig = "")`: a warning on the binder of `start`, traced
    /// from level 2.
    pub fn add_warning_text(&self, start: &K::Start, mess: &str, orig: &str) {
        let binder = self.binder_else_void(start);
        binder.add_warning(mess, orig);
        if self.trace_level() <= 1 {
            return;
        }
        self.start_trace(&binder, start, self.nesting_level(), 2);
        let head = format!("    --> Warning : {mess}");
        self.send_trace_line(Gravity::Warning, &head, (!orig.is_empty()).then_some(orig));
    }
    /// `AddWarning(start, amsg)`.
    pub fn add_warning_msg(&self, start: &K::Start, amsg: &Msg) {
        let orig = if amsg.is_edited() {
            c_text_of(amsg.original())
        } else {
            String::new()
        };
        self.add_warning_text(start, &c_text_of(amsg.value()), &orig);
    }
    /// `Mend(start, pref = "")`: mends the check of the binder of `start`
    /// (`Interface_Check::Mend`, whose failures are returned).
    pub fn mend(&self, start: &K::Start, pref: &str) -> Result<(), Failure> {
        let binder = self.find_and_mask(start);
        match binder.get() {
            Some(live) => live.c_check().mend(Some(pref), 0).map(|_| ()),
            None => Ok(()),
        }
    }
    /// `Check(start)`: the check of the binder of `start` (null when not mapped).
    pub fn check(&self, start: &K::Start) -> Handle<InterfaceCheck> {
        self.find(start)
            .get()
            .map(|b| b.check())
            .unwrap_or_default()
    }
    /// `BindTransient(start, res)`: sets the result of a void simple binder, else binds
    /// a new simple binder (`Transfer_TransferFailure` from `Rebind`).
    pub fn bind_transient(
        &self,
        start: &K::Start,
        res: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        if res.is_null() {
            return Ok(());
        }
        let former = self.find(start);
        let simple = former.cast::<TransferSimpleBinderOfTransient>();
        if let Some(binder) = simple
            .get()
            .filter(|b| b.status() == TransferStatusResult::StatusVoid)
        {
            return binder.set_result(res);
        }
        let binder = TransferSimpleBinderOfTransient::new();
        binder.set_result(res)?;
        self.bind(start, &Handle::new(binder).cast())
    }
    /// `FindTransient(start)`: the result of the simple binder of `start`, else null.
    pub fn find_transient(&self, start: &K::Start) -> Handle<StandardTransient> {
        let simple = self.find(start).cast::<TransferSimpleBinderOfTransient>();
        simple
            .get()
            .filter(|b| b.has_result())
            .map(TransferSimpleBinderOfTransient::result)
            .unwrap_or_default()
    }
    /// `BindMultiple(start)`: binds a multiple binder; `Transfer_TransferFailure` when
    /// `start` has another kind of binder.
    pub fn bind_multiple(&self, start: &K::Start) -> Result<(), Failure> {
        let binder = self.find_and_mask(start);
        if binder.is_null() {
            return self.bind(start, &Handle::new(TransferMultipleBinder::new()).cast());
        }
        if binder.cast::<TransferMultipleBinder>().is_some() {
            return Ok(());
        }
        let last = self.read().thelastbnd.clone();
        self.start_trace(&last, start, self.nesting_level(), 4);
        Err(transfer_failure("TransferProcess : BindMultiple"))
    }
    /// `AddMultiple(start, res)`: adds a result to the multiple binder of `start`
    /// (`Transfer_TransferFailure` without one).
    pub fn add_multiple(
        &self,
        start: &K::Start,
        res: &Handle<StandardTransient>,
    ) -> Result<(), Failure> {
        let binder = self.find_and_mask(start);
        let multr = binder.cast::<TransferMultipleBinder>();
        if let Some(live) = multr.get() {
            live.add_transient_result(res);
            return Ok(());
        }
        self.start_trace(&binder, start, self.nesting_level(), 4);
        if binder.is_null() {
            return Err(transfer_failure(
                "TransferProcess : AddMultiple, nothing bound",
            ));
        }
        Err(transfer_failure(
            "TransferProcess : AddMultiple, Binder not a MultipleBinder",
        ))
    }
    /// `FindTypedTransient(start, atype, val)`.
    pub fn find_typed_transient(
        &self,
        start: &K::Start,
        atype: &RuntimeType,
        val: &mut Handle<StandardTransient>,
    ) -> bool {
        self.get_typed_transient(&self.find(start), atype, val)
    }
    /// `GetTypedTransient(binder, atype, val)`: `Transfer_SimpleBinderOfTransient::GetTypedResult`.
    pub fn get_typed_transient(
        &self,
        binder: &Handle<dyn TransferBinder>,
        atype: &RuntimeType,
        val: &mut Handle<StandardTransient>,
    ) -> bool {
        TransferSimpleBinderOfTransient::get_typed_result(binder, atype, val)
    }
    /// `FindTypedTransient<T>(start, atype, val)` (template): the typed result, cast to
    /// `T` (false when the cast is null; `val` is kept when nothing is found).
    pub fn find_typed_transient_as<T: ?Sized + RuntimeTyped>(
        &self,
        start: &K::Start,
        atype: &RuntimeType,
        val: &mut Handle<T>,
    ) -> bool {
        self.get_typed_transient_as(&self.find(start), atype, val)
    }
    /// `GetTypedTransient<T>(binder, atype, val)` (template): the typed result, cast to `T`.
    pub fn get_typed_transient_as<T: ?Sized + RuntimeTyped>(
        &self,
        binder: &Handle<dyn TransferBinder>,
        atype: &RuntimeType,
        val: &mut Handle<T>,
    ) -> bool {
        let mut res: Handle<StandardTransient> = val.cast();
        if !self.get_typed_transient(binder, atype, &mut res) {
            return false;
        }
        *val = res.cast();
        val.is_some()
    }
    /// `NbMapped`.
    pub fn nb_mapped(&self) -> i32 {
        self.read().themap.extent() as i32
    }
    /// `Mapped(num)`: `Standard_OutOfRange` out of 1..NbMapped.
    pub fn mapped(&self, num: i32) -> Result<K::Start, Failure> {
        let index = usize::try_from(num).unwrap_or(0);
        self.read().themap.find_key(index).cloned()
    }
    /// `MapIndex(start)`: 0 when not mapped.
    pub fn map_index(&self, start: &K::Start) -> i32 {
        self.read().themap.find_index(start) as i32
    }
    /// `MapItem(num)`: `Standard_OutOfRange` out of 1..NbMapped.
    pub fn map_item(&self, num: i32) -> Result<Handle<dyn TransferBinder>, Failure> {
        let index = usize::try_from(num).unwrap_or(0);
        self.read().themap.find_from_index(index).cloned()
    }
    /// `SetRoot(start)`: records a mapped `start` as a root.
    pub fn set_root(&self, start: &K::Start) {
        let index = self.map_index(start);
        if index == 0 {
            return;
        }
        self.write().theroots.add(index);
        if self.trace_level() > 2 {
            let item = self.map_item(index).unwrap_or_default();
            self.start_trace(&item, start, self.nesting_level(), 3);
        }
    }
    /// `SetRootManagement(stat)`: whether `Transferring` records the roots.
    pub fn set_root_management(&self, stat: bool) {
        self.write().therootm = stat;
    }
    /// `NbRoots`.
    pub fn nb_roots(&self) -> i32 {
        self.read().theroots.extent() as i32
    }
    /// The map index of root `num` (0 out of range, as natively).
    fn root_map_index(&self, num: i32) -> usize {
        let st = self.read();
        let found = usize::try_from(num)
            .ok()
            .and_then(|n| st.theroots.find_key(n).ok().copied());
        found.and_then(|ind| usize::try_from(ind).ok()).unwrap_or(0)
    }
    /// `Root(num)`: `Standard_OutOfRange` out of 1..NbRoots (the native map index 0).
    pub fn root(&self, num: i32) -> Result<K::Start, Failure> {
        let index = self.root_map_index(num);
        self.read().themap.find_key(index).cloned()
    }
    /// `RootItem(num)`: `Standard_OutOfRange` out of 1..NbRoots.
    pub fn root_item(&self, num: i32) -> Result<Handle<dyn TransferBinder>, Failure> {
        let index = self.root_map_index(num);
        self.read().themap.find_from_index(index).cloned()
    }
    /// `RootIndex(start)`: the rank of `start` among the roots, 0 when not a root.
    pub fn root_index(&self, start: &K::Start) -> i32 {
        let index = self.map_index(start);
        if index == 0 {
            return 0;
        }
        self.read().theroots.find_index(&index) as i32
    }
    /// `NestingLevel`.
    pub fn nesting_level(&self) -> i32 {
        self.read().thelevel
    }
    /// `ResetNestingLevel`.
    pub fn reset_nesting_level(&self) {
        self.write().thelevel = 0;
    }
}
