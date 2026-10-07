//! Opt-in native destructor policy without changing the ordinary filler's type or variance.
use super::{BoundingTreeInsertion, BoundingVolume, ObjBnd, UBTreeFiller};
use std::ops::{Deref, DerefMut};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Owns the existing queue and applies one self-assignment policy during Fill and Drop.
/// A panic from automatic filling aborts, matching OCCT's noexcept destructor domain.
/// Explicit `fill()` can unwind; its queue and partial tree remain available for reset/retry.
/// Example: `let filler = filler.with_self_assignment_policy(|pair| pair.object += 0);`.
/// Destruction of Rust policy captures is outside the native payload contract.
pub struct UBTreeFillerWithSelfAssignment<
    't,
    T: Clone + Default,
    B: BoundingVolume,
    Tree: BoundingTreeInsertion<T, B> + ?Sized,
    Policy: FnMut(&mut ObjBnd<T, B>),
> {
    inner: UBTreeFiller<'t, T, B, Tree>,
    policy: Policy,
}
impl<'t, T: Clone + Default, B: BoundingVolume, Tree: BoundingTreeInsertion<T, B> + ?Sized>
    UBTreeFiller<'t, T, B, Tree>
{
    /// Selects identical-slot assignment for explicit filling and automatic native destruction.
    /// Example: `filler.with_self_assignment_policy(|pair| pair.assign_self_with(assign_obj, assign_bnd))`.
    pub fn with_self_assignment_policy<Policy: FnMut(&mut ObjBnd<T, B>)>(
        self,
        policy: Policy,
    ) -> UBTreeFillerWithSelfAssignment<'t, T, B, Tree, Policy> {
        UBTreeFillerWithSelfAssignment {
            inner: self,
            policy,
        }
    }
}
impl<T, B, Tree, Policy> UBTreeFillerWithSelfAssignment<'_, T, B, Tree, Policy>
where
    T: Clone + Default,
    B: BoundingVolume,
    Tree: BoundingTreeInsertion<T, B> + ?Sized,
    Policy: FnMut(&mut ObjBnd<T, B>),
{
    /// Uses the retained policy on the original queue; `assert_eq!(filler.fill(), 0)`.
    /// Catching a panic preserves native partial progress; reset explicitly to cancel retries.
    pub fn fill(&mut self) -> i32 {
        self.inner.fill_with_self_assignment(&mut self.policy)
    }
}
impl<'t, T, B, Tree, Policy> Deref for UBTreeFillerWithSelfAssignment<'t, T, B, Tree, Policy>
where
    T: Clone + Default,
    B: BoundingVolume,
    Tree: BoundingTreeInsertion<T, B> + ?Sized,
    Policy: FnMut(&mut ObjBnd<T, B>),
{
    type Target = UBTreeFiller<'t, T, B, Tree>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<T, B, Tree, Policy> DerefMut for UBTreeFillerWithSelfAssignment<'_, T, B, Tree, Policy>
where
    T: Clone + Default,
    B: BoundingVolume,
    Tree: BoundingTreeInsertion<T, B> + ?Sized,
    Policy: FnMut(&mut ObjBnd<T, B>),
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl<T, B, Tree, Policy> Drop for UBTreeFillerWithSelfAssignment<'_, T, B, Tree, Policy>
where
    T: Clone + Default,
    B: BoundingVolume,
    Tree: BoundingTreeInsertion<T, B> + ?Sized,
    Policy: FnMut(&mut ObjBnd<T, B>),
{
    fn drop(&mut self) {
        if self.inner.pending.is_empty() {
            return;
        }
        // OCCT 3d097a0328 ~UBTreeFiller is noexcept. Abort before field Drop can replay Fill.
        if catch_unwind(AssertUnwindSafe(|| self.fill())).is_err() {
            std::process::abort();
        }
    }
}
