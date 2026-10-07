use super::{DynamicArray, DynamicArrayClearState, Failure};
impl<T> DynamicArray<T> {
    /// Transfers coherent blocks to a raw Clear state; `let state = array.into_clear_state();`.
    /// A rejected shape returns the unchanged array and its domain error before any transfer.
    /// Abandoning the admitted state leaks payload allocations; explicitly complete or repair it.
    pub fn into_clear_state(self) -> Result<DynamicArrayClearState<T>, (Self, Box<Failure>)> {
        if !self.clear_state_prefix_is_coherent() {
            let message = format!(
                "Clear state size {}, stride {}, blocks {}: expected coherent registered prefixes (zero stride admits only retained slot0)",
                self.used,
                self.increment,
                self.blocks.len()
            );
            let failure = Failure::with_type("Standard_DomainError", &message, "");
            return Err((self, Box::new(failure)));
        }
        Ok(DynamicArrayClearState::take_array(self))
    }
    fn clear_state_prefix_is_coherent(&self) -> bool {
        if self.increment == 0 {
            return self.zero_clear_prefix_is_coherent();
        }
        if self.used > self.blocks.len().saturating_mul(self.increment) {
            return false;
        }
        self.blocks.iter().enumerate().all(|(index, block)| {
            let expected = self
                .used
                .saturating_sub(index.saturating_mul(self.increment))
                .min(self.increment);
            block.len() == expected
        })
    }
    fn zero_clear_prefix_is_coherent(&self) -> bool {
        if self.used > 1 || (self.used == 1 && size_of::<T>() == 0) {
            return false;
        }
        if self.used == 1 && self.blocks.is_empty() {
            return false;
        }
        self.blocks
            .iter()
            .enumerate()
            .all(|(index, block)| block.len() == if index == 0 { self.used } else { 0 })
    }
}
