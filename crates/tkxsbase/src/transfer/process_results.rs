// Rust port of DataExchange/TKXSBase/Transfer/Transfer_ProcessForTransient_0.cxx and
// Transfer_ProcessForFinder_0.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871: the result
// iterators and the check lists.

//! The results and the check lists of [`TransferProcess`].
use super::{
    TransferBinder, TransferIteratorOfProcess, TransferProcess, TransferProcessKind,
    TransferStatusExec,
};
use crate::interface::InterfaceCheckIterator;
use rustycad_tkernel::standard::Handle;

/// Neither initial nor done: an abnormal execution status.
fn is_abnormal(binder: &Handle<dyn TransferBinder>) -> bool {
    !matches!(
        binder.status_exec(),
        TransferStatusExec::StatusInitial | TransferStatusExec::StatusDone
    )
}

impl<K: TransferProcessKind> TransferProcess<K> {
    /// The binder and the start of map index `i` (a null binder when out of range).
    fn item_at(&self, i: i32) -> (Handle<dyn TransferBinder>, K::Start) {
        let st = self.read();
        let index = usize::try_from(i).unwrap_or(0);
        let binder = st
            .themap
            .find_from_index(index)
            .cloned()
            .unwrap_or_default();
        (
            binder,
            st.themap.find_key(index).cloned().unwrap_or_default(),
        )
    }
    /// Adds the binder of map index `i` (when not null) to `iter`.
    fn add_result(&self, iter: &mut TransferIteratorOfProcess<K>, i: i32, withstart: bool) {
        let (binder, start) = self.item_at(i);
        if binder.is_null() {
            return;
        }
        if withstart {
            iter.add_with_start(&binder, &start);
        } else {
            iter.add_item(&binder);
        }
    }
    /// `RootResult(withstart = false)`: the binders of the roots.
    pub fn root_result(&self, withstart: bool) -> TransferIteratorOfProcess<K> {
        let mut iter = TransferIteratorOfProcess::new(withstart);
        let roots: Vec<i32> = self.read().theroots.iter().copied().collect();
        for i in roots {
            self.add_result(&mut iter, i, withstart);
        }
        iter
    }
    /// `CompleteResult(withstart = false)`: all the binders.
    pub fn complete_result(&self, withstart: bool) -> TransferIteratorOfProcess<K> {
        let mut iter = TransferIteratorOfProcess::new(withstart);
        for i in 1..=self.nb_mapped() {
            self.add_result(&mut iter, i, withstart);
        }
        iter
    }
    /// `AbnormalResult`: the binders neither initial nor done, with their starts.
    pub fn abnormal_result(&self) -> TransferIteratorOfProcess<K> {
        let mut iter = TransferIteratorOfProcess::new(true);
        for i in 1..=self.nb_mapped() {
            let (binder, start) = self.item_at(i);
            if binder.is_some() && is_abnormal(&binder) {
                iter.add_with_start(&binder, &start);
            }
        }
        iter
    }
    /// `CheckList(erronly)`: the checks with fails (and warnings unless `erronly`), each
    /// numbered by `CheckNum` (else the map index) and bound to its start. An abnormal
    /// binder without fail first gets the fail "Transfer in Abnormal Status (!= Initial or
    /// Done)".
    pub fn check_list(&self, erronly: bool) -> InterfaceCheckIterator {
        let mut list = InterfaceCheckIterator::new();
        for i in 1..=self.nb_mapped() {
            let (binder, ent) = self.item_at(i);
            if binder.is_null() {
                continue;
            }
            let check = binder.check();
            if is_abnormal(&binder) && !check.has_failed() {
                check.add_fail_text("Transfer in Abnormal Status (!= Initial or Done)", "");
            }
            if !check.has_failed() && (erronly || check.nb_warnings() == 0) {
                continue;
            }
            let num = match self.check_num(&ent) {
                0 => i,
                num => num,
            };
            check.set_entity(&K::as_transient(&ent));
            list.add(&check, num);
        }
        list
    }
    /// `ResultOne(start, level, withstart = false)`. Native defect kept: the scope array
    /// it reads is never filled, so the result is always empty.
    pub fn result_one(
        &self,
        _start: &K::Start,
        _level: i32,
        withstart: bool,
    ) -> TransferIteratorOfProcess<K> {
        TransferIteratorOfProcess::new(withstart)
    }
    /// `CheckListOne(start, level, erronly)`. Native defect kept: the scope array it reads
    /// is never filled, so the list is always empty.
    pub fn check_list_one(
        &self,
        _start: &K::Start,
        _level: i32,
        _erronly: bool,
    ) -> InterfaceCheckIterator {
        InterfaceCheckIterator::new()
    }
    /// `IsCheckListEmpty(start, level, erronly)`: false when `start` is not mapped. Native
    /// defect kept: the scope array it reads is never filled, so a mapped `start` is
    /// always true.
    pub fn is_check_list_empty(&self, start: &K::Start, _level: i32, _erronly: bool) -> bool {
        self.map_index(start) != 0
    }
    /// `RemoveResult(start, level, compute = true)`. Native defect kept: the scope array
    /// it reads is never filled, so nothing is removed.
    pub fn remove_result(&self, _start: &K::Start, _level: i32, _compute: bool) {}
}
