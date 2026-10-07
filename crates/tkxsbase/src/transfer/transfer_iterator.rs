// Rust port of DataExchange/TKXSBase/Transfer/Transfer_TransferIterator (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Transfer_TransferIterator`: iterates the binders of a transfer result, with selections
//! by binder type, result type and multiplicity.
use super::{TransferBinder, TransferSimpleBinderOfTransient, TransferStatusExec};
use crate::interface::InterfaceCheck;
use rustycad_tkernel::standard::{
    Failure, Handle, RuntimeType, RuntimeTyped, StandardTransient, exceptions,
};

/// `Transfer_TransferIterator`.
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferTransferIterator, TransferVoidBinder};
/// use rustycad_tkernel::standard::Handle;
/// let mut iter = TransferTransferIterator::new();
/// iter.add_item(&Handle::new(TransferVoidBinder::new()).cast());
/// iter.start();
/// assert!(iter.more() && !iter.has_result().unwrap());
/// assert_eq!(iter.number(), 1);
/// ```
#[derive(Clone, Debug)]
pub struct TransferTransferIterator {
    /// `thecurr` (protected): the current item, from 1.
    pub(super) thecurr: i32,
    theitems: Vec<Handle<dyn TransferBinder>>,
    theselect: Vec<i32>,
    themaxi: i32,
}

impl Default for TransferTransferIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl TransferTransferIterator {
    /// `Transfer_TransferIterator()`: empty, current item 1.
    pub fn new() -> Self {
        Self {
            thecurr: 1,
            theitems: Vec::new(),
            theselect: Vec::new(),
            themaxi: 0,
        }
    }
    /// `AddItem(atr)`: appends a selected binder.
    pub fn add_item(&mut self, atr: &Handle<dyn TransferBinder>) {
        self.theitems.push(atr.clone());
        self.theselect.push(1);
        self.themaxi = self.theselect.len() as i32;
    }
    /// Unselects item `i` (from 1), lowering the maximum when it is the last one.
    fn unselect(&mut self, i: usize) {
        self.theselect[i - 1] = 0;
        if self.themaxi == i as i32 {
            self.themaxi = i as i32 - 1;
        }
    }
    /// `SelectBinder(atype, keep)`: keeps (or removes) the binders of type `atype`.
    /// A null binder is natively dereferenced: `Standard_NullObject`.
    pub fn select_binder(&mut self, atype: &RuntimeType, keep: bool) -> Result<(), Failure> {
        for i in (1..=self.theitems.len()).rev() {
            let item = &self.theitems[i - 1];
            let kind = item.dynamic_type().ok_or_else(|| null_binder(i))?;
            if kind.is_subtype(atype) != keep {
                self.unselect(i);
            }
        }
        Ok(())
    }
    /// `SelectResult(atype, keep)`: keeps (or removes) the binders with one result of type
    /// `atype`. Native quirk kept: the result type compared is the one of the current
    /// item (`ResultType()`), not of each item, so `Standard_NoSuchObject` without a
    /// current item; every type is a subtype of `Standard_Transient`, so the comparison is
    /// always the dynamic one.
    pub fn select_result(&mut self, atype: &RuntimeType, keep: bool) -> Result<(), Failure> {
        for i in (1..=self.theitems.len()).rev() {
            let atr = self.theitems[i - 1].clone();
            let btype = self.result_type()?;
            let live = atr.get().ok_or_else(|| null_binder(i))?;
            let matchtype = if !live.has_result() || live.is_multiple() {
                false
            } else {
                btype.ok_or_else(|| exceptions::null_object("Transfer_TransferIterator::SelectResult: null result type of a multiple current binder"))?.is_subtype(atype)
            };
            if matchtype != keep {
                self.unselect(i);
            }
        }
        Ok(())
    }
    /// `SelectUnique(keep)`: keeps the binders with a unique result (`keep`) or the
    /// multiple ones.
    pub fn select_unique(&mut self, keep: bool) -> Result<(), Failure> {
        for i in (1..=self.theitems.len()).rev() {
            let live = self.theitems[i - 1].get().ok_or_else(|| null_binder(i))?;
            if live.is_multiple() == keep {
                self.unselect(i);
            }
        }
        Ok(())
    }
    /// `SelectItem(num, keep)`: selects or unselects one item (nothing out of range; the
    /// maximum is not raised again).
    pub fn select_item(&mut self, num: i32, keep: bool) {
        if num < 1 || num as usize > self.theselect.len() {
            return;
        }
        self.theselect[num as usize - 1] = i32::from(keep);
    }
    /// `Number`: the count of selected items up to the maximum.
    pub fn number(&self) -> i32 {
        let maxi = self.themaxi.max(0) as usize;
        self.theselect[..maxi].iter().filter(|s| **s != 0).count() as i32
    }
    /// `Start`.
    pub fn start(&mut self) {
        self.thecurr = 0;
        self.next();
    }
    /// `More`.
    pub fn more(&mut self) -> bool {
        if self.thecurr > self.themaxi {
            return false;
        }
        if self.selected(self.thecurr) == 0 {
            self.next();
        }
        if self.thecurr > self.themaxi {
            return false;
        }
        self.selected(self.thecurr) > 0
    }
    /// `Next`: the next selected item (the native recursion is a loop).
    pub fn next(&mut self) {
        self.thecurr += 1;
        while self.thecurr <= self.themaxi && self.selected(self.thecurr) == 0 {
            self.thecurr += 1;
        }
    }
    /// The selection flag of item `num` (0 out of the list).
    fn selected(&self, num: i32) -> i32 {
        usize::try_from(num - 1)
            .ok()
            .and_then(|i| self.theselect.get(i))
            .copied()
            .unwrap_or(0)
    }
    /// `Value`: the current binder; `Standard_NoSuchObject` without a current item.
    pub fn value(&self) -> Result<Handle<dyn TransferBinder>, Failure> {
        if self.thecurr == 0 || self.thecurr > self.themaxi || self.selected(self.thecurr) == 0 {
            return Err(Failure::with_type(
                "Standard_NoSuchObject",
                &format!(
                    "TransferIterator : Value; current item {} out of the {} selected",
                    self.thecurr, self.themaxi
                ),
                "",
            ));
        }
        Ok(self.theitems[self.thecurr as usize - 1].clone())
    }
    /// The current binder, live (a null one is natively dereferenced).
    fn live_value(&self) -> Result<Handle<dyn TransferBinder>, Failure> {
        let value = self.value()?;
        if value.is_null() {
            return Err(null_binder(self.thecurr as usize));
        }
        Ok(value)
    }
    /// `HasResult`.
    pub fn has_result(&self) -> Result<bool, Failure> {
        Ok(self.live_value()?.has_result())
    }
    /// `HasUniqueResult`.
    pub fn has_unique_result(&self) -> Result<bool, Failure> {
        let atr = self.live_value()?;
        Ok(!atr.is_multiple() && atr.has_result())
    }
    /// `ResultType`: the result type of the current binder, `None` (null) when multiple.
    pub fn result_type(&self) -> Result<Option<&'static RuntimeType>, Failure> {
        let atr = self.live_value()?;
        Ok((!atr.is_multiple()).then(|| atr.result_type()))
    }
    /// `HasTransientResult`.
    pub fn has_transient_result(&self) -> Result<bool, Failure> {
        Ok(self
            .result_type()?
            .is_some_and(|t| t.is_subtype(StandardTransient::static_type())))
    }
    /// `TransientResult`: the result of a simple binder, else null.
    pub fn transient_result(&self) -> Result<Handle<StandardTransient>, Failure> {
        let simple = self.value()?.cast::<TransferSimpleBinderOfTransient>();
        Ok(simple
            .get()
            .map(TransferSimpleBinderOfTransient::result)
            .unwrap_or_default())
    }
    /// `Status`.
    pub fn status(&self) -> Result<TransferStatusExec, Failure> {
        Ok(self.live_value()?.status_exec())
    }
    /// `HasFails`.
    pub fn has_fails(&self) -> Result<bool, Failure> {
        Ok(self.live_value()?.check().has_failed())
    }
    /// `HasWarnings`.
    pub fn has_warnings(&self) -> Result<bool, Failure> {
        Ok(self.live_value()?.check().has_warnings())
    }
    /// `Check`.
    pub fn check(&self) -> Result<Handle<InterfaceCheck>, Failure> {
        Ok(self.live_value()?.check())
    }
}

/// The native null dereference of a null binder item.
fn null_binder(num: usize) -> Failure {
    exceptions::null_object(&format!(
        "Transfer_TransferIterator: item {num} is a null binder; expected a binder"
    ))
}
