//! Native relational operators compare signed positions even when owner-sensitive equality differs.
use super::{DynamicArrayCachedIterator, Failure};
impl<T, const M: bool> DynamicArrayCachedIterator<T, M> {
    /// Checks the native left-minus-right ordering independently of container identity.
    /// `left.less_than(&right)?;` rejects native signed-overflow cases.
    pub fn less_than<const O: bool>(
        &self,
        other: &DynamicArrayCachedIterator<T, O>,
    ) -> Result<bool, Failure> {
        Ok(self.positional_difference(other)? < 0)
    }
    /// Preserves native right-minus-left ordering, including its own signed-overflow boundary.
    /// `left.greater_than(&right)?;` can compare cursors from different owners.
    pub fn greater_than<const O: bool>(
        &self,
        other: &DynamicArrayCachedIterator<T, O>,
    ) -> Result<bool, Failure> {
        other.less_than(self)
    }
    /// Negates native right-minus-left ordering, which differs from checking left-minus-right overflow.
    /// `left.at_most(&right)?;` compares positions without reading saved element pointers.
    pub fn at_most<const O: bool>(
        &self,
        other: &DynamicArrayCachedIterator<T, O>,
    ) -> Result<bool, Failure> {
        Ok(!other.less_than(self)?)
    }
    /// Negates native left-minus-right ordering independently of owner-sensitive equality.
    /// `left.at_least(&right)?;` remains meaningful after element allocation destruction.
    pub fn at_least<const O: bool>(
        &self,
        other: &DynamicArrayCachedIterator<T, O>,
    ) -> Result<bool, Failure> {
        Ok(!self.less_than(other)?)
    }
}
impl<T, const M: bool, const O: bool> PartialEq<DynamicArrayCachedIterator<T, O>>
    for DynamicArrayCachedIterator<T, M>
{
    fn eq(&self, other: &DynamicArrayCachedIterator<T, O>) -> bool {
        self.same_position(other)
    }
}
impl<T, const M: bool> Eq for DynamicArrayCachedIterator<T, M> {}
