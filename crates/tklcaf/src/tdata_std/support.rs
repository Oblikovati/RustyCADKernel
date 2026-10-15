//! The code every TDataStd attribute repeats natively: the static `Set` that finds or adds
//! the attribute (the file-static `SetAttr`), the user-ID members and the down casts of
//! `Restore`/`Paste` (ocaf-design.md section 4).
use crate::live;
use crate::tdf::{TdfAttribute, TdfAttributeState, TdfLabel};
use rustycad_tkernel::standard::{Failure, Guid, Handle, RuntimeTyped, SharedState};

/// The native `SetAttr`: the attribute of `id` on the label, created with `make` (and given
/// the ID through `SetID` when `with_id`) and added when absent.
pub fn find_or_add<T>(
    label: &TdfLabel,
    id: &Guid,
    with_id: bool,
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
    if with_id {
        live(&created, "TDataStd SetAttr")?.set_id(id)?;
    }
    label.add_attribute(&created.cast(), true)?;
    Ok(created)
}

/// `SetID(guid)` of the attributes with a user ID: backed up when it changes.
pub fn set_user_id<A: TdfAttribute + ?Sized>(
    attribute: &A,
    id: &SharedState<Guid>,
    guid: &Guid,
) -> Result<(), Failure> {
    set_backed_up(attribute, id, guid)
}

/// `SetID()` of the attributes with a user ID: back up, then the default ID.
pub fn reset_user_id<A: TdfAttribute + ?Sized>(
    attribute: &A,
    id: &SharedState<Guid>,
    default: Guid,
) -> Result<(), Failure> {
    attribute.backup()?;
    *id.write() = default;
    Ok(())
}

/// The `occ::down_cast<T>(with)` of `Restore` and `Paste`; `Standard_NullObject` when the
/// attribute is not a `T` (a null dereference natively).
pub fn cast_attribute<T: RuntimeTyped + ?Sized>(
    attribute: &Handle<dyn TdfAttribute>,
    who: &str,
) -> Result<Handle<T>, Failure> {
    let cast = attribute.cast::<T>();
    live(&cast, who)?;
    Ok(cast)
}

/// Assigns a value through a backup unless it is unchanged (the native OCC2932 pattern).
pub fn set_backed_up<A: TdfAttribute + ?Sized, V: PartialEq + Clone>(
    attribute: &A,
    slot: &SharedState<V>,
    value: &V,
) -> Result<(), Failure> {
    if *slot.read() == *value {
        return Ok(());
    }
    attribute.backup()?;
    *slot.write() = value.clone();
    Ok(())
}

/// `Backup()` through the attribute's own handle, so that the virtual `BackupCopy` is the
/// most derived one when a native class derives from this attribute; an attribute without
/// handle has no label and backs up nothing. Example: `backup_as_dynamic(&attribute)?`.
pub fn backup_as_dynamic<A: TdfAttribute + ?Sized>(attribute: &A) -> Result<(), Failure> {
    match attribute.attribute_base().this() {
        Ok(this) => live(&this, "TDF_Attribute::Backup")?.backup(),
        Err(_) => attribute.backup(),
    }
}

/// The `this` pointer of a native `Dump` (`anOS << this`).
pub fn address_of<T>(value: &T) -> String {
    format!("{:p}", std::ptr::from_ref(value))
}
