//! Borrowed assignment preserves native item identity and invokes copy-assignment hooks.
use super::Sequence;
use crate::standard::Failure;

impl<T: Clone> Sequence<T> {
    /// Copies into an existing item through its assignment hook; `sequence.set_value_ref(1, &item)?`.
    pub fn set_value_ref(&mut self, index: usize, value: &T) -> Result<(), Failure> {
        self.change_value(index)?.clone_from(value);
        Ok(())
    }

    /// Copies another indexed item, with an explicit hook for native aliasing assignment.
    /// Example: `sequence.set_value_from_index_with(1, 2, |item| item.assign_self())?`.
    /// Rust cannot call clone_from with aliased mutable/shared references. On equal indices,
    /// the hook represents the item's C++ self-assignment; different indices use clone_from.
    pub fn set_value_from_index_with(
        &mut self,
        destination: usize,
        source: usize,
        assign_self: impl FnOnce(&mut T),
    ) -> Result<(), Failure> {
        // The native source Value expression runs before entering SetValue.
        let source = self.checked_node("Value", source, true)?;
        let destination = self.checked_node("ChangeValue", destination, true)?;
        if source == destination {
            assign_self(unsafe { super::owner::item_mut(destination) });
            return Ok(());
        }
        // Distinct live owned nodes provide disjoint item borrows without aliasing their links.
        unsafe {
            super::owner::item_mut::<T>(destination).clone_from(super::owner::item_ref(source))
        };
        Ok(())
    }
}
