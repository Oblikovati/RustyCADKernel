// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ResultFromTransient (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_ResultFromTransient`: the result of the transfer of one starting entity,
//! with the results of its sub-transfers.
use super::{TransferBinder, TransferTransientProcess};
use crate::interface::{InterfaceCheck, InterfaceCheckStatus};
use rustycad_tkernel::{
    impl_transient,
    ncollection::IndexedMap,
    standard::{Handle, SharedState, StandardTransient},
};
use std::sync::OnceLock;

/// The native static `voidcheck`: one empty check shared by every result without binder.
fn void_check() -> Handle<InterfaceCheck> {
    static VOIDCHECK: OnceLock<Handle<InterfaceCheck>> = OnceLock::new();
    VOIDCHECK
        .get_or_init(|| Handle::new(InterfaceCheck::new()))
        .clone()
}

/// The private fields of `Transfer_ResultFromTransient`.
#[derive(Default)]
struct ResultState {
    thestart: Handle<StandardTransient>,
    thebinder: Handle<dyn TransferBinder>,
    thesubs: Option<Vec<Handle<StandardTransient>>>,
}

/// `Transfer_ResultFromTransient`.
///
/// ```
/// use rustycad_tkxsbase::transfer::TransferResultFromTransient;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let main = Handle::new(TransferResultFromTransient::new());
/// let start = Handle::new(StandardTransient::default());
/// main.set_start(&start);
/// assert!(TransferResultFromTransient::result_from_key(&main, &start) == main);
/// assert!(!main.has_result() && main.nb_sub_results() == 0);
/// ```
#[derive(Default)]
pub struct TransferResultFromTransient {
    transient: StandardTransient,
    state: SharedState<ResultState>,
}
impl_transient!(
    TransferResultFromTransient,
    "Transfer_ResultFromTransient",
    StandardTransient,
    transient
);

impl TransferResultFromTransient {
    /// `Transfer_ResultFromTransient()`: no start, no binder, no sub-result.
    pub fn new() -> Self {
        Self::default()
    }
    /// `SetStart(start)`.
    pub fn set_start(&self, start: &Handle<StandardTransient>) {
        self.state.write().thestart = start.clone();
    }
    /// `SetBinder(binder)`.
    pub fn set_binder(&self, binder: &Handle<dyn TransferBinder>) {
        self.state.write().thebinder = binder.clone();
    }
    /// `Start`.
    pub fn start(&self) -> Handle<StandardTransient> {
        self.state.read().thestart.clone()
    }
    /// `Binder`.
    pub fn binder(&self) -> Handle<dyn TransferBinder> {
        self.state.read().thebinder.clone()
    }
    /// `HasResult`: the binder has a result.
    pub fn has_result(&self) -> bool {
        self.binder().get().is_some_and(|b| b.has_result())
    }
    /// `Check`: the check of the binder, else the shared empty check.
    pub fn check(&self) -> Handle<InterfaceCheck> {
        self.binder().get().map_or_else(void_check, |b| b.check())
    }
    /// `CheckStatus`: the status of the binder check (OK without binder).
    pub fn check_status(&self) -> InterfaceCheckStatus {
        self.binder()
            .get()
            .map_or(InterfaceCheckStatus::CheckOK, |b| b.check().status())
    }
    /// `ClearSubs`.
    pub fn clear_subs(&self) {
        self.state.write().thesubs = None;
    }
    /// `AddSubResult(sub)`: nothing for null.
    pub fn add_sub_result(&self, sub: &Handle<TransferResultFromTransient>) {
        if sub.is_null() {
            return;
        }
        self.state
            .write()
            .thesubs
            .get_or_insert_with(Vec::new)
            .push(sub.cast());
    }
    /// `NbSubResults`.
    pub fn nb_sub_results(&self) -> i32 {
        self.state
            .read()
            .thesubs
            .as_ref()
            .map_or(0, |s| s.len() as i32)
    }
    /// `SubResult(num)`: null out of 1..NbSubResults.
    pub fn sub_result(&self, num: i32) -> Handle<TransferResultFromTransient> {
        let st = self.state.read();
        let found = st
            .thesubs
            .as_ref()
            .and_then(|s| usize::try_from(num - 1).ok().and_then(|i| s.get(i)));
        found.map(|sub| sub.cast()).unwrap_or_default()
    }
    /// `ResultFromKey(key)`: `this` when `key` is its start, else the first sub-result
    /// (in depth) with that start, else null. `this` is the receiving handle.
    pub fn result_from_key(this: &Handle<Self>, key: &Handle<StandardTransient>) -> Handle<Self> {
        let Some(live) = this.get() else {
            return Handle::default();
        };
        if *key == live.start() {
            return this.clone();
        }
        (1..=live.nb_sub_results())
            .map(|i| Self::result_from_key(&live.sub_result(i), key))
            .find(|res| res.is_some())
            .unwrap_or_default()
    }
    /// `FillMap(map)`: adds the sub-results, then their own sub-results, recursively.
    pub fn fill_map(&self, map: &mut IndexedMap<Handle<StandardTransient>>) {
        let subs = self.state.read().thesubs.clone().unwrap_or_default();
        for sub in &subs {
            map.add(sub.clone());
        }
        for i in 1..=subs.len() as i32 {
            if let Some(live) = self.sub_result(i).get() {
                live.fill_map(map);
            }
        }
    }
    /// `Fill(TP)`: nothing (natively empty).
    pub fn fill(&self, _tp: &Handle<TransferTransientProcess>) {}
    /// `Strip`: nothing (natively empty).
    pub fn strip(&self) {}
    /// `FillBack(TP)`: nothing (natively empty).
    pub fn fill_back(&self, _tp: &Handle<TransferTransientProcess>) {}
}
