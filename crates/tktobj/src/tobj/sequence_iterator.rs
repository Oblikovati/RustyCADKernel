// Rust port of ApplicationFramework/TKTObj/TObj/TObj_SequenceIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_SequenceIterator`: an iterator on a shared sequence of objects.
use super::{TObjBackReferences, TObjObject, TObjObjectIterator};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, RuntimeType, SharedState, StandardTransient},
};

/// `TObj_SequenceIterator`; it sees later changes of the shared sequence, as natively.
///
/// ```
/// use rustycad_tktobj::tobj::{TObjObjectIterator, TObjSequenceIterator};
/// let iterator = TObjSequenceIterator::empty();
/// assert!(!iterator.more());
/// ```
pub struct TObjSequenceIterator {
    transient: StandardTransient,
    index: SharedState<usize>,
    kind: Option<&'static RuntimeType>,
    objects: Handle<TObjBackReferences>,
}
impl_transient!(TObjSequenceIterator, "TObj_SequenceIterator", dyn TObjObjectIterator,
    transient, interfaces: [dyn TObjObjectIterator]);

impl TObjSequenceIterator {
    /// `TObj_SequenceIterator()`: an empty iterator.
    pub fn empty() -> Self {
        Self::new(&Handle::default(), None)
    }
    /// `TObj_SequenceIterator(theObjects, theType)`: the objects of that type (any when
    /// `None`).
    pub fn new(objects: &Handle<TObjBackReferences>, kind: Option<&'static RuntimeType>) -> Self {
        Self {
            transient: StandardTransient::default(),
            index: SharedState::new(1),
            kind,
            objects: objects.clone(),
        }
    }
    /// `myIndex` (protected): the 1-based current index.
    pub fn index(&self) -> usize {
        *self.index.read()
    }
    /// `myType` (protected): the type of the iterated objects, `None` for any.
    pub fn kind(&self) -> Option<&'static RuntimeType> {
        self.kind
    }
    /// `myObjects` (protected): the shared sequence.
    pub fn objects(&self) -> Handle<TObjBackReferences> {
        self.objects.clone()
    }
    /// The object at the current index; null past the end or on a null entry.
    fn current(&self) -> Handle<dyn TObjObject> {
        let index = *self.index.read();
        let Some(objects) = self.objects.get() else {
            return Handle::default();
        };
        objects.sequence(|items| items.value(index).cloned().unwrap_or_default())
    }
}

impl TObjObjectIterator for TObjSequenceIterator {
    /// `More`: a non-null current object; an object of another type is skipped (the
    /// native const member moves the iterator).
    fn more(&self) -> bool {
        loop {
            let current = self.current();
            if current.is_null() {
                return false;
            }
            if self.kind.is_none_or(|kind| current.is_kind(kind)) {
                return true;
            }
            *self.index.write() += 1;
        }
    }
    /// `Next`.
    fn next(&self) -> Result<(), Failure> {
        *self.index.write() += 1;
        Ok(())
    }
    /// `Value`: the current object (null where the native `Value` raises
    /// `Standard_OutOfRange` or dereferences a null sequence).
    fn value(&self) -> Handle<dyn TObjObject> {
        self.current()
    }
}
