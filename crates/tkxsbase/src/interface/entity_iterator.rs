// Rust port of DataExchange/TKXSBase/Interface/Interface_EntityIterator (.hxx, .cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Interface_EntityIterator`: a list of entities with a cursor, both shared by the
//! copies of the iterator (natively two handles), so moving a copy moves the original.
use super::InterfaceIntVal;
use rustycad_tkernel::standard::{
    Failure, Handle, RuntimeType, SharedState, StandardTransient, exceptions,
};
use std::sync::Arc;

/// The shared list of an iterator (`occ::handle<NCollection_HSequence<occ::handle<Standard_Transient>>>`).
pub type InterfaceEntityListHandle = Arc<SharedState<Vec<Handle<StandardTransient>>>>;

/// `Interface_EntityIterator`. No OCCT class overrides the virtual `Start`, so it is an
/// inherent method; `Interface_GraphContent` embeds the iterator.
///
/// ```
/// use rustycad_tkxsbase::interface::InterfaceEntityIterator;
/// use rustycad_tkernel::standard::{Handle, StandardTransient};
/// let mut iter = InterfaceEntityIterator::new();
/// iter.add_item(&Handle::new(StandardTransient::default()));
/// iter.add_item(&Handle::new(StandardTransient::default()));
/// let mut count = 0;
/// iter.start();
/// while iter.more() {
///     count += 1;
///     iter.next();
/// }
/// assert_eq!(count, iter.nb_entities());
/// ```
#[derive(Clone, Default)]
pub struct InterfaceEntityIterator {
    thecurr: Handle<InterfaceIntVal>,
    thelist: Option<InterfaceEntityListHandle>,
}

impl InterfaceEntityIterator {
    /// `Interface_EntityIterator()`: empty, without cursor.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Interface_EntityIterator(list)`: iterates on `list` (shared).
    pub fn from_list(list: &InterfaceEntityListHandle) -> Self {
        Self {
            thecurr: Handle::new(InterfaceIntVal::new()),
            thelist: Some(list.clone()),
        }
    }
    /// `AddList(list)`: appends the entities of `list` and resets the cursor.
    pub fn add_list(&mut self, list: &[Handle<StandardTransient>]) {
        let target = self.list_or_create();
        self.cursor_or_create();
        *self.thecurr.c_value() = 0;
        target.write().extend(list.iter().cloned());
    }
    /// `AddItem(anentity)`: appends a non-null entity and resets the cursor.
    pub fn add_item(&mut self, anentity: &Handle<StandardTransient>) {
        if anentity.is_null() {
            return;
        }
        self.cursor_or_create();
        let target = self.list_or_create();
        *self.thecurr.c_value() = 0;
        target.write().push(anentity.clone());
    }
    /// `GetOneItem(anentity)`: `AddItem`.
    pub fn get_one_item(&mut self, anentity: &Handle<StandardTransient>) {
        self.add_item(anentity);
    }
    /// `Reset` (protected): a new empty list, the cursor reset.
    pub fn reset(&mut self) {
        self.cursor_or_create();
        *self.thecurr.c_value() = 0;
        self.thelist = Some(Arc::new(SharedState::new(Vec::new())));
    }
    /// `SelectType(atype, keep)`: keeps the entities kind of `atype` (or the others), in a
    /// new list.
    pub fn select_type(&mut self, atype: &RuntimeType, keep: bool) {
        let Some(list) = &self.thelist else { return };
        let kept: Vec<Handle<StandardTransient>> = list
            .read()
            .iter()
            .filter(|e| e.is_kind(atype) == keep)
            .cloned()
            .collect();
        self.thelist = Some(Arc::new(SharedState::new(kept)));
    }
    /// `NbEntities`: prepares the iteration when not started.
    pub fn nb_entities(&self) -> i32 {
        let Some(list) = &self.thelist else { return 0 };
        if self.thecurr.get().is_some_and(|c| c.value() == 0) {
            self.start();
        }
        list.read().len() as i32
    }
    /// `NbTyped(type)`.
    pub fn nb_typed(&self, atype: &RuntimeType) -> i32 {
        let Some(list) = &self.thelist else { return 0 };
        list.read().iter().filter(|e| e.is_kind(atype)).count() as i32
    }
    /// `Typed(type)`: a new iterator on the entities kind of `atype`.
    pub fn typed(&self, atype: &RuntimeType) -> InterfaceEntityIterator {
        let mut res = InterfaceEntityIterator::new();
        let Some(list) = &self.thelist else {
            return res;
        };
        for entity in list.read().iter().filter(|e| e.is_kind(atype)) {
            res.add_item(entity);
        }
        res
    }
    /// `Start`: the cursor on the first entity.
    pub fn start(&self) {
        if let Some(curr) = self.thecurr.get() {
            *curr.c_value() = 1;
        }
    }
    /// `More`: prepares the iteration when not started.
    pub fn more(&self) -> bool {
        let Some(curr) = self.thecurr.get() else {
            return false;
        };
        if curr.value() == 0 {
            self.start();
        }
        let Some(list) = &self.thelist else {
            return false;
        };
        curr.value() <= list.read().len() as i32
    }
    /// `Next`: no check, as natively (a null cursor does nothing instead of crashing).
    pub fn next(&self) {
        if let Some(curr) = self.thecurr.get() {
            *curr.c_value() += 1;
        }
    }
    /// `Value`: `Standard_NoSuchObject` without list or out of the list.
    pub fn value(&self) -> Result<Handle<StandardTransient>, Failure> {
        let no_such = || {
            exceptions::no_such_object(
                "Interface_EntityIterator: no current entity; expected More() to be true",
            )
        };
        let list = self.thelist.as_ref().ok_or_else(no_such)?;
        let current = self.thecurr.get().map_or(0, InterfaceIntVal::value);
        let index = usize::try_from(current - 1).map_err(|_| no_such())?;
        list.read().get(index).cloned().ok_or_else(no_such)
    }
    /// `Content`: a copy of the list (empty when none); prepares the iteration.
    pub fn content(&self) -> Vec<Handle<StandardTransient>> {
        if self.thecurr.get().is_some_and(|c| c.value() == 0) {
            self.start();
        }
        self.thelist
            .as_ref()
            .map(|l| l.read().clone())
            .unwrap_or_default()
    }
    /// The shared list itself (natively the handle `Content` returns), when it exists.
    pub fn content_handle(&self) -> Option<InterfaceEntityListHandle> {
        self.thelist.clone()
    }
    /// `Destroy`: forgets the cursor (the iterator is empty again for `More`).
    pub fn destroy(&mut self) {
        self.thecurr.nullify();
    }
    fn cursor_or_create(&mut self) {
        if self.thecurr.is_null() {
            self.thecurr = Handle::new(InterfaceIntVal::new());
        }
    }
    fn list_or_create(&mut self) -> InterfaceEntityListHandle {
        self.thelist
            .get_or_insert_with(|| Arc::new(SharedState::new(Vec::new())))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::standard::RuntimeTyped;
    use rustycad_tkernel::tcollection::HAsciiString;

    #[test]
    fn copies_share_list_and_cursor() {
        let mut iter = InterfaceEntityIterator::new();
        assert!(!iter.more());
        iter.add_item(&Handle::default());
        assert_eq!(iter.nb_entities(), 0);
        let text: Handle<StandardTransient> = Handle::new(HAsciiString::new("a")).cast();
        iter.add_list(&[Handle::new(StandardTransient::default()), text.clone()]);
        let copy = iter.clone();
        assert!(copy.more());
        copy.next();
        assert!(iter.value().unwrap() == text);
        iter.next();
        assert!(!copy.more());
        assert_eq!(
            iter.value().err().unwrap().exception_type(),
            "Standard_NoSuchObject"
        );
        assert_eq!(iter.nb_typed(HAsciiString::static_type()), 1);
        assert_eq!(iter.typed(HAsciiString::static_type()).nb_entities(), 1);
        iter.select_type(HAsciiString::static_type(), false);
        assert_eq!(iter.content().len(), 1);
        assert_eq!(copy.content().len(), 2);
        iter.destroy();
        assert!(!iter.more());
        iter.reset();
        assert_eq!(iter.nb_entities(), 0);
        let shared = copy.content_handle().unwrap();
        let other = InterfaceEntityIterator::from_list(&shared);
        assert_eq!(other.nb_entities(), 2);
    }
}
