//! Copy arithmetic reloads through the original owner; postfix retains the pre-navigation cache.
use super::{DynamicArray, DynamicArrayCachedIterator, Failure};
mod mutable;
mod shared;

impl<T, const M: bool> DynamicArrayCachedIterator<T, M> {
    /// Subtracts native signed positions, independently of original owner identity.
    /// `let distance = cursor.positional_difference(&other)?;` permits different const specializations.
    /// Overflow is a checked Rust extension to the native signed-arithmetic precondition.
    pub fn positional_difference<const O: bool>(
        &self,
        other: &DynamicArrayCachedIterator<T, O>,
    ) -> Result<isize, Failure> {
        let left = self.position.index as isize;
        let right = other.position.index as isize;
        left.checked_sub(right).ok_or_else(|| {
            let reason =
                format!("positions {left} and {right}: expected representable signed difference");
            Failure::with_type("Standard_OutOfRange", &reason, "")
        })
    }
    fn postfix_operation(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<(), Failure>,
    ) -> Result<Self, Failure> {
        let original = *self;
        operation(self)?;
        Ok(original)
    }
    fn copied_operation(
        &self,
        operation: impl FnOnce(&mut Self) -> Result<(), Failure>,
    ) -> Result<Self, Failure> {
        let mut displaced = *self;
        operation(&mut displaced)?;
        Ok(displaced)
    }
    fn require_subscript_slot(&self, offset: isize) -> Result<(), Failure> {
        if self.more() {
            return Ok(());
        }
        let reason = format!(
            "subscript offset {offset}, index {}: expected a live initialized slot",
            self.position.index
        );
        Err(Failure::with_type("Standard_OutOfRange", &reason, ""))
    }
}
fn negate_displacement(offset: isize) -> Result<isize, Failure> {
    offset.checked_neg().ok_or_else(|| {
        let reason = format!("offset {offset}: expected representable signed negation");
        Failure::with_type("Standard_OutOfRange", &reason, "")
    })
}
