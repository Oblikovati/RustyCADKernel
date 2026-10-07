// Rust port of the members ApplicationFramework/TKLCAF/TDataStd/TDataStd_IntegerList.cxx,
// TDataStd_RealList.cxx, TDataStd_ExtStringList.cxx, TDataStd_BooleanList.cxx and
// TDataStd_ReferenceList.cxx repeat, at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The list core of the five list attributes (ocaf-design.md section 7): the native
//! `NCollection_List` is a `Vec` in list order, and every member is one generic function.
use super::support::cast_attribute;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{TdfAttribute, TdfAttributeState, TdfLabel};
use rustycad_tkcdf::json_dump::class_begin;
use rustycad_tkernel::standard::{
    Failure, Guid, Handle, RuntimeTyped, SharedState, dump::DumpStream, exceptions,
};
use std::io;

/// The native fields `myList` and `myID`.
pub(crate) struct ListState<T> {
    pub(crate) list: Vec<T>,
    pub(crate) id: Guid,
}

impl<T> ListState<T> {
    /// The native constructor: an empty list and the default ID.
    pub(crate) fn new(id: Guid) -> Self {
        Self {
            list: Vec::new(),
            id,
        }
    }
}

/// A list attribute.
pub(crate) trait ListAttribute: TdfAttribute + RuntimeTyped + Sized {
    /// The stored item (`uint8_t` for `TDataStd_BooleanList`).
    type Item: Clone + PartialEq + Send + Sync + 'static;
    /// The default ID (`GetID`).
    const DEFAULT_ID: Guid;
    /// The short class name of `Dump` (`IntegerList`).
    const DUMP_NAME: &'static str;
    /// A new empty attribute.
    fn make() -> Self;
    /// The fields.
    fn list_state(&self) -> &SharedState<ListState<Self::Item>>;
    /// One `OCCT_DUMP_FIELD_VALUE_*` of an item in `DumpJson`.
    fn dump_item(stream: &mut DumpStream, item: &Self::Item) -> io::Result<()>;
}

/// The file-static `SetAttr`: the attribute of `guid`, added (with the ID) when absent.
pub(crate) fn set_attr<A: ListAttribute>(
    label: &TdfLabel,
    guid: &Guid,
) -> Result<Handle<A>, Failure> {
    let mut found: Handle<A> = Handle::default();
    if label.find_attribute_as(guid, &mut found)? {
        return Ok(found);
    }
    let created = Handle::new(A::make());
    live(&created, A::TYPE_NAME)?.set_id(guid)?;
    label.add_attribute(&created.cast(), false)?;
    Ok(created)
}

/// `List`: a copy of the items in order.
pub(crate) fn items<A: ListAttribute>(attribute: &A) -> Vec<A::Item> {
    attribute.list_state().read().list.clone()
}

/// `Extent`.
pub(crate) fn extent<A: ListAttribute>(attribute: &A) -> i32 {
    attribute.list_state().read().list.len() as i32
}

/// Backs up, then applies `edit` to the list (`Prepend`, `Append`, `Clear`).
pub(crate) fn edit<A: ListAttribute>(
    attribute: &A,
    edit: impl FnOnce(&mut Vec<A::Item>),
) -> Result<(), Failure> {
    attribute.backup()?;
    edit(&mut attribute.list_state().write().list);
    Ok(())
}

/// Backs up and edits the list at `position` when there is one; true when it edited
/// (the `InsertBefore`, `InsertAfter` and `Remove` members).
fn edit_at<A: ListAttribute>(
    attribute: &A,
    position: Option<usize>,
    edit: impl FnOnce(&mut Vec<A::Item>, usize),
) -> Result<bool, Failure> {
    let Some(position) = position else {
        return Ok(false);
    };
    attribute.backup()?;
    edit(&mut attribute.list_state().write().list, position);
    Ok(true)
}

/// The position of the first item equal to `value`.
fn position_of<A: ListAttribute>(attribute: &A, value: &A::Item) -> Option<usize> {
    attribute
        .list_state()
        .read()
        .list
        .iter()
        .position(|item| item == value)
}

/// The position of the 1-based `index`, when it is in the list.
fn position_at<A: ListAttribute>(attribute: &A, index: i32) -> Option<usize> {
    let position = usize::try_from(index).ok()?.checked_sub(1)?;
    (position < attribute.list_state().read().list.len()).then_some(position)
}

/// `InsertBefore(value, before_value)`: before the first item equal to `before_value`.
pub(crate) fn insert_before<A: ListAttribute>(
    attribute: &A,
    value: A::Item,
    before: &A::Item,
) -> Result<bool, Failure> {
    let position = position_of(attribute, before);
    edit_at(attribute, position, |list, at| list.insert(at, value))
}

/// `InsertBefore(index, value)`: before the item at the 1-based `index`.
pub(crate) fn insert_before_by_index<A: ListAttribute>(
    attribute: &A,
    index: i32,
    value: A::Item,
) -> Result<bool, Failure> {
    let position = position_at(attribute, index);
    edit_at(attribute, position, |list, at| list.insert(at, value))
}

/// `InsertAfter(value, after_value)`: after the first item equal to `after_value`.
pub(crate) fn insert_after<A: ListAttribute>(
    attribute: &A,
    value: A::Item,
    after: &A::Item,
) -> Result<bool, Failure> {
    let position = position_of(attribute, after);
    edit_at(attribute, position, |list, at| list.insert(at + 1, value))
}

/// `InsertAfter(index, value)`: after the item at the 1-based `index`.
pub(crate) fn insert_after_by_index<A: ListAttribute>(
    attribute: &A,
    index: i32,
    value: A::Item,
) -> Result<bool, Failure> {
    let position = position_at(attribute, index);
    edit_at(attribute, position, |list, at| list.insert(at + 1, value))
}

/// `Remove(value)`: the first item equal to `value`.
pub(crate) fn remove<A: ListAttribute>(attribute: &A, value: &A::Item) -> Result<bool, Failure> {
    let position = position_of(attribute, value);
    edit_at(attribute, position, |list, at| {
        list.remove(at);
    })
}

/// `Remove(index)`: the item at the 1-based `index`.
pub(crate) fn remove_by_index<A: ListAttribute>(
    attribute: &A,
    index: i32,
) -> Result<bool, Failure> {
    let position = position_at(attribute, index);
    edit_at(attribute, position, |list, at| {
        list.remove(at);
    })
}

/// `First`; `Standard_NoSuchObject` for an empty list, as `NCollection_List::First`.
pub(crate) fn first<A: ListAttribute>(attribute: &A) -> Result<A::Item, Failure> {
    let state = attribute.list_state().read();
    state
        .list
        .first()
        .cloned()
        .ok_or_else(|| empty_list::<A>("First"))
}

/// `Last`; `Standard_NoSuchObject` for an empty list, as `NCollection_List::Last`.
pub(crate) fn last<A: ListAttribute>(attribute: &A) -> Result<A::Item, Failure> {
    let state = attribute.list_state().read();
    state
        .list
        .last()
        .cloned()
        .ok_or_else(|| empty_list::<A>("Last"))
}

/// The failure of `First`/`Last` on an empty list.
fn empty_list<A: ListAttribute>(member: &str) -> Failure {
    let message = format!(
        "{}::{member}: the list is empty; expected at least one item",
        A::TYPE_NAME
    );
    exceptions::no_such_object(&message)
}

/// `Restore(With)`: the items and the ID of the other list.
pub(crate) fn restore<A: ListAttribute>(
    attribute: &A,
    with: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let other = cast_attribute::<A>(with, A::TYPE_NAME)?;
    let (list, id) = {
        let state = live(&other, A::TYPE_NAME)?.list_state().read();
        (state.list.clone(), state.id)
    };
    let mut state = attribute.list_state().write();
    (state.list, state.id) = (list, id);
    Ok(())
}

/// `Paste(Into, RT)`: `Into` is cleared, takes `items` one by one and the ID;
/// `Standard_NullObject` when it is not the same class (a null dereference natively).
pub(crate) fn paste_items<A: ListAttribute>(
    into: &Handle<dyn TdfAttribute>,
    items: Vec<A::Item>,
    id: &Guid,
) -> Result<(), Failure> {
    let target = cast_attribute::<A>(into, A::TYPE_NAME)?;
    let target = live(&target, A::TYPE_NAME)?;
    edit(target, Vec::clear)?;
    for item in items {
        edit(target, |list| list.push(item))?;
    }
    target.set_id(id)
}

/// `Paste` of the lists without references: every item is copied.
pub(crate) fn paste<A: ListAttribute>(
    attribute: &A,
    into: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let (list, id) = {
        let state = attribute.list_state().read();
        (state.list.clone(), state.id)
    };
    paste_items::<A>(into, list, &id)
}

/// `SetID(guid)`: backed up when it changes.
pub(crate) fn set_id<A: ListAttribute>(attribute: &A, guid: &Guid) -> Result<(), Failure> {
    if attribute.list_state().read().id == *guid {
        return Ok(());
    }
    attribute.backup()?;
    attribute.list_state().write().id = *guid;
    Ok(())
}

/// `SetID()`: backs up, then the default ID.
pub(crate) fn set_default_id<A: ListAttribute>(attribute: &A) -> Result<(), Failure> {
    attribute.backup()?;
    attribute.list_state().write().id = A::DEFAULT_ID;
    Ok(())
}

/// `Dump`: `"\n<Name>: <ID>\n"`.
pub(crate) fn dump<A: ListAttribute>(attribute: &A, stream: &mut dyn io::Write) -> io::Result<()> {
    let id = attribute.list_state().read().id;
    writeln!(stream, "\n{}: {id}", A::DUMP_NAME)
}

/// `DumpJson`: the class, the attribute base and one field per item.
pub(crate) fn dump_json<A: ListAttribute>(
    attribute: &A,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, A::TYPE_NAME)?;
    dump_attribute_json_base(attribute, stream, depth)?;
    for item in items(attribute) {
        A::dump_item(stream, &item)?;
    }
    Ok(())
}
