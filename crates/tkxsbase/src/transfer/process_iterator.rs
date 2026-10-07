// Rust port of DataExchange/TKXSBase/Transfer/Transfer_IteratorOfProcessForTransient and
// Transfer_IteratorOfProcessForFinder (.hxx, _0.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TransferIteratorOfProcess<K>`: a [`TransferTransferIterator`] with the starting
//! objects of its binders (the two native classes are one template).
use super::{TransferBinder, TransferProcessKind, TransferTransferIterator};
use rustycad_tkernel::{
    ncollection::IndexedDataMap,
    standard::{Failure, Handle, exceptions},
};
use std::ops::{Deref, DerefMut};

/// `Transfer_IteratorOfProcessForTransient` / `Transfer_IteratorOfProcessForFinder`;
/// dereferences to its [`TransferTransferIterator`].
///
/// ```
/// use rustycad_tkxsbase::transfer::{TransferIteratorOfProcess, TransferTransientKind, TransferVoidBinder};
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let mut iter = TransferIteratorOfProcess::<TransferTransientKind>::new(true);
/// let start = Handle::new(StandardTransient::default());
/// iter.add_with_start(&Handle::new(TransferVoidBinder::new()).cast(), &start);
/// iter.start();
/// assert!(iter.more() && iter.starting().unwrap() == start);
/// ```
pub struct TransferIteratorOfProcess<K: TransferProcessKind> {
    base: TransferTransferIterator,
    thestarts: Option<Vec<K::Start>>,
}

impl<K: TransferProcessKind> Clone for TransferIteratorOfProcess<K> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            thestarts: self.thestarts.clone(),
        }
    }
}

impl<K: TransferProcessKind> Deref for TransferIteratorOfProcess<K> {
    type Target = TransferTransferIterator;
    fn deref(&self) -> &TransferTransferIterator {
        &self.base
    }
}

impl<K: TransferProcessKind> DerefMut for TransferIteratorOfProcess<K> {
    fn deref_mut(&mut self) -> &mut TransferTransferIterator {
        &mut self.base
    }
}

impl<K: TransferProcessKind> TransferIteratorOfProcess<K> {
    /// `Transfer_IteratorOfProcessFor...(withstarts)`: records the starts when asked.
    pub fn new(withstarts: bool) -> Self {
        Self {
            base: TransferTransferIterator::new(),
            thestarts: withstarts.then(Vec::new),
        }
    }
    /// `Add(binder)`: `Standard_NoSuchObject` when the iterator records the starts.
    pub fn add(&mut self, binder: &Handle<dyn TransferBinder>) -> Result<(), Failure> {
        if self.thestarts.is_some() {
            let text = format!(
                "{} : Add, Starting Object required not provided",
                K::ITERATOR_NAME
            );
            return Err(exceptions::no_such_object(&text));
        }
        self.base.add_item(binder);
        Ok(())
    }
    /// `Add(binder, start)`: the start is recorded when the iterator records them.
    pub fn add_with_start(&mut self, binder: &Handle<dyn TransferBinder>, start: &K::Start) {
        self.base.add_item(binder);
        if let Some(starts) = self.thestarts.as_mut() {
            starts.push(start.clone());
        }
    }
    /// `Filter(list, keep = true)`: keeps (or removes) the items whose start is in `list`
    /// and removes (or keeps) the others. Native quirk kept: the item selected is the rank
    /// of the start in a map of the starts, so a repeated start shifts the selection.
    pub fn filter(&mut self, list: Option<&[K::Start]>, keep: bool) {
        let (Some(list), Some(starts)) = (list, self.thestarts.as_ref()) else {
            return;
        };
        if starts.is_empty() {
            return;
        }
        let mut amap: IndexedDataMap<K::Start, (), K::Hasher> = IndexedDataMap::new(starts.len());
        let ranks: Vec<usize> = starts.iter().map(|s| amap.add(s.clone(), ())).collect();
        for j in ranks {
            self.base.select_item(j as i32, !keep);
        }
        for start in list {
            let j = amap.find_index(start);
            if j > 0 {
                self.base.select_item(j as i32, keep);
            }
        }
    }
    /// `HasStarting`.
    pub fn has_starting(&self) -> bool {
        self.thestarts.is_some()
    }
    /// `Starting`: the start of the current item; `Standard_NoSuchObject` without starts,
    /// `Standard_OutOfRange` without a current item.
    pub fn starting(&self) -> Result<K::Start, Failure> {
        let starts = self.thestarts.as_ref().ok_or_else(|| {
            exceptions::no_such_object("TransferIterator : No Starting defined at all")
        })?;
        let curr = self.base.thecurr;
        let found = usize::try_from(curr - 1).ok().and_then(|i| starts.get(i));
        found.cloned().ok_or_else(|| {
            exceptions::out_of_range(&format!(
                "Transfer_IteratorOfProcess::Starting: item {curr}; expected [1, {}]",
                starts.len()
            ))
        })
    }
}
