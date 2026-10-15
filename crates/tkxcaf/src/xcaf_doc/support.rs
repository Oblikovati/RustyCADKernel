//! The code the XCAFDoc attributes repeat natively: the static `Set` that finds the attribute
//! of a label or adds a new one first in its list (`L.AddAttribute(A)`, `append = false`), and
//! the down cast of `Restore` and `Paste`.
use rustycad_tkernel::standard::{Failure, Guid, Handle, RuntimeTyped};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeState, TdfLabel};

/// The attribute of `id` on the label, created with `make` and added (first, as the native
/// default `AddAttribute(A)`) when absent.
pub(crate) fn find_or_add<T>(
    label: &TdfLabel,
    id: &Guid,
    make: impl FnOnce() -> T,
) -> Result<Handle<T>, Failure>
where
    T: TdfAttribute + RuntimeTyped,
{
    let mut found: Handle<T> = Handle::default();
    if label.find_attribute_as(id, &mut found)? {
        return Ok(found);
    }
    let created = Handle::new(make());
    label.add_attribute(&created.clone().cast(), false)?;
    Ok(created)
}

/// The attribute of `id` on the label as a `T`, or null.
pub(crate) fn find<T: RuntimeTyped + ?Sized>(
    label: &TdfLabel,
    id: &Guid,
) -> Result<Handle<T>, Failure> {
    let mut found: Handle<T> = Handle::default();
    label.find_attribute_as(id, &mut found)?;
    Ok(found)
}

/// The `occ::down_cast<T>(with)` of `Restore` and `Paste`, dereferenced:
/// `Standard_NullObject` when the attribute is not a `T` (a null dereference natively).
pub(crate) fn downcast<T: RuntimeTyped + ?Sized>(
    attribute: &Handle<dyn TdfAttribute>,
    who: &str,
) -> Result<Handle<T>, Failure> {
    let cast = attribute.cast::<T>();
    rustycad_tklcaf::live(&cast, who)?;
    Ok(cast)
}

/// The live value behind a handle, `Standard_NullObject` for a null handle.
pub(crate) fn live<'a, T: RuntimeTyped + ?Sized>(
    handle: &'a Handle<T>,
    who: &str,
) -> Result<&'a T, Failure> {
    rustycad_tklcaf::live(handle, who)
}

/// The children of a label (`TDF_ChildIterator(label)`), in order.
pub(crate) fn child_labels(label: &TdfLabel) -> Result<Vec<TdfLabel>, Failure> {
    let mut iterator = rustycad_tklcaf::tdf::TdfChildIterator::new(label, false)?;
    let mut labels = Vec::new();
    while iterator.more() {
        labels.push(iterator.value());
        iterator.next();
    }
    Ok(labels)
}

/// The labels of the attributes of `id` on the children of a label
/// (`TDF_ChildIDIterator(label, id)`), in order.
pub(crate) fn child_labels_with(label: &TdfLabel, id: &Guid) -> Result<Vec<TdfLabel>, Failure> {
    let mut iterator = rustycad_tklcaf::tdf::TdfChildIdIterator::new(label, id, false)?;
    let mut labels = Vec::new();
    while iterator.more() {
        labels.push(live(&iterator.value(), "TDF_ChildIDIterator")?.label());
        iterator.next()?;
    }
    Ok(labels)
}
