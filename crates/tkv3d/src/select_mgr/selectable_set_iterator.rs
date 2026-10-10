//! Live subset iteration follows native More/Next separation, without a snapshot.
use super::{SelectMgrBvhSubset, SelectMgrSelectableObject, SelectMgrSelectableObjectSet};
use rustycad_tkernel::standard::{Failure, Handle};

/// Cursor over the five live indexed maps; `let mut it = SelectMgrSelectableObjectSetIterator::new(&set)`.
#[derive(Clone, Copy)]
pub struct SelectMgrSelectableObjectSetIterator<'a> {
    set: Option<&'a SelectMgrSelectableObjectSet>,
    subset: usize,
    index: usize,
}
impl Default for SelectMgrSelectableObjectSetIterator<'_> {
    fn default() -> Self {
        Self::empty()
    }
}
impl<'a> SelectMgrSelectableObjectSetIterator<'a> {
    /// Start without a set or current object; `SelectMgrSelectableObjectSetIterator::empty()`.
    pub fn empty() -> Self {
        Self {
            set: None,
            subset: SelectMgrBvhSubset::COUNT,
            index: 1,
        }
    }
    /// Initialize and skip empty starting subsets; `SelectMgrSelectableObjectSetIterator::new(&set)`.
    pub fn new(set: &'a SelectMgrSelectableObjectSet) -> Self {
        let mut cursor = Self::default();
        cursor.init(set);
        cursor
    }
    /// Restart on a set; `cursor.init(&set)`.
    pub fn init(&mut self, set: &'a SelectMgrSelectableObjectSet) {
        self.set = Some(set);
        self.subset = 0;
        self.index = 1;
        self.more();
    }
    /// Advance over empty or exhausted subsets; `while cursor.more() { cursor.next(); }`.
    pub fn more(&mut self) -> bool {
        let Some(set) = self.set else {
            return false;
        };
        while self.index > set.subsets[self.subset].read().objects.len() {
            if self.subset == SelectMgrBvhSubset::COUNT - 1 {
                return false;
            }
            self.subset += 1;
            self.index = 1;
        }
        true
    }
    /// Advance only the current map cursor; call More to cross subsets; `cursor.next()`.
    pub fn next(&mut self) {
        self.index = self.index.wrapping_add(1);
    }
    /// Read the current live object or raise for an exhausted cursor; `cursor.value()?`.
    pub fn value(&self) -> Result<Handle<dyn SelectMgrSelectableObject>, Failure> {
        let object = self.set.and_then(|set| {
            set.subsets[self.subset]
                .read()
                .objects
                .find_key(self.index)
                .ok()
                .cloned()
        });
        object.ok_or_else(|| {
            Failure::with_message(
                "Standard_NoSuchObject",
                &format!(
                    "selectable cursor subset={} index={}; expected a current object",
                    self.subset, self.index
                ),
            )
        })
    }
}
