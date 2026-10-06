//! Explicit projection supplies the derived ChangeSequence dispatch that Rust inheritance lacks.
use super::HSequence;
use crate::standard::{Failure, Handle, RuntimeTyped};

fn sequence_handle_source<U: ?Sized + RuntimeTyped>(other: &Handle<U>) -> Result<&U, Failure> {
    other.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "NCollection_HSequence::Append null handle; expected a handled sequence",
            "",
        )
    })
}

impl<T: Clone + Send + Sync + 'static> HSequence<T> {
    /// Moves every item of the handled sequence to the end of this one, leaving it empty
    /// (OCCT `Append(const handle<T>&)`); appending a sequence to itself does nothing and a
    /// null handle reports `Standard_NullObject`; `items.append_handle(&others)?`.
    pub fn append_handle(&self, other: &Handle<HSequence<T>>) -> Result<(), Failure> {
        self.append_handle_projected(other, |source| source)
    }

    /// Projects a derived handled object's sequence before native self/empty checks.
    /// The callback is an explicit Rust policy for C++ ChangeSequence lookup, not a native callback.
    /// Null rejects before projection; reentrant locks must be avoided by the caller.
    /// `items.append_handle_projected(&derived, |object| &object.sequence_base)?`.
    pub fn append_handle_projected<U: ?Sized + RuntimeTyped>(
        &self,
        other: &Handle<U>,
        project: impl FnOnce(&U) -> &Self,
    ) -> Result<(), Failure> {
        let source = project(sequence_handle_source(other)?);
        if std::ptr::eq(source, self) {
            return Ok(());
        }
        source.change_sequence(|moved| self.append_sequence(moved));
        Ok(())
    }
}
