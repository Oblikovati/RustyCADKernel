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
        let source = self.offset("Value", source)?;
        let destination = self.offset("ChangeValue", destination)?;
        if source == destination {
            assign_self(&mut self.nodes[destination].value);
            return Ok(());
        }
        self.assign_distinct_nodes(destination, source);
        Ok(())
    }

    fn assign_distinct_nodes(&mut self, destination: usize, source: usize) {
        let split = destination.max(source);
        let (earlier, later) = self.nodes.split_at_mut(split);
        if destination > source {
            later[0].value.clone_from(&earlier[source].value);
            return;
        }
        earlier[destination].value.clone_from(&later[0].value);
    }
}
