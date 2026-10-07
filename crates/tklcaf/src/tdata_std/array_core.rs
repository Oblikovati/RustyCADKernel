// Rust port of the members ApplicationFramework/TKLCAF/TDataStd/TDataStd_IntegerArray.cxx,
// TDataStd_RealArray.cxx, TDataStd_ExtStringArray.cxx and TDataStd_ByteArray.cxx repeat,
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The array core of `TDataStd_IntegerArray`, `RealArray`, `ExtStringArray` and
//! `ByteArray` (ocaf-design.md section 7): their fields are one [`ArrayState`] and their
//! members call these generic functions, with the native differences as the constants
//! of [`ArrayAttribute`].
use super::support::cast_attribute;
use crate::json::dump_attribute_json_base;
use crate::live;
use crate::tdf::{TdfAttribute, TdfAttributeState, TdfLabel};
use rustycad_tkcdf::json_dump::{class_begin, field_integer};
use rustycad_tkernel::{
    ncollection::HArray1,
    standard::{Failure, Guid, Handle, RuntimeTyped, SharedState, dump::DumpStream, exceptions},
};
use std::io;

/// The item of a native `NCollection_HArray1` attribute.
pub(crate) trait ArrayItem: Clone + PartialEq + Default + Send + Sync + 'static {}
impl<T: Clone + PartialEq + Default + Send + Sync + 'static> ArrayItem for T {}

/// The native fields `myValue`, `myIsDelta` and `myID`.
pub(crate) struct ArrayState<T: ArrayItem> {
    pub(crate) value: Handle<HArray1<T>>,
    pub(crate) is_delta: bool,
    pub(crate) id: Guid,
}

impl<T: ArrayItem> ArrayState<T> {
    /// The native constructor: no array, no delta, the default ID.
    pub(crate) fn new(id: Guid) -> Self {
        Self {
            value: Handle::default(),
            is_delta: false,
            id,
        }
    }
}

/// An array attribute and the native differences between the four classes.
pub(crate) trait ArrayAttribute: TdfAttribute + RuntimeTyped + Sized {
    /// The array item.
    type Item: ArrayItem;
    /// The value `Init` fills the new array with.
    const FILL: fn() -> Self::Item;
    /// `Value(index)` of an attribute without array.
    const NULL_VALUE: fn() -> Self::Item;
    /// `Upper()` of an attribute without array (`-1` for `TDataStd_ByteArray`).
    const NULL_UPPER: i32;
    /// `ChangeArray` compares the bounds only when the array exists (`TDataStd_RealArray`).
    const CHANGE_GUARDS_NULL: bool;
    /// `SetAttr` gives the ID before `Init` (`TDataStd_ExtStringArray`).
    const SET_ID_FIRST: bool;
    /// The default ID (`GetID`).
    const DEFAULT_ID: Guid;
    /// A new attribute without array (the native default constructor).
    fn make() -> Self;
    /// The fields.
    fn array_state(&self) -> &SharedState<ArrayState<Self::Item>>;
    /// One `OCCT_DUMP_FIELD_VALUE_*` of an item in `DumpJson`.
    fn dump_item(stream: &mut DumpStream, item: &Self::Item) -> io::Result<()>;
}

/// `myValue` (shared with the caller, as the native handle).
pub(crate) fn array<A: ArrayAttribute>(attribute: &A) -> Handle<HArray1<A::Item>> {
    attribute.array_state().read().value.clone()
}

/// The file-static `SetAttr` of `Set(label, [guid,] lower, upper, isDelta)`.
pub(crate) fn set_attr<A: ArrayAttribute>(
    label: &TdfLabel,
    guid: &Guid,
    (lower, upper): (i32, i32),
    is_delta: bool,
) -> Result<Handle<A>, Failure> {
    let mut found: Handle<A> = Handle::default();
    if label.find_attribute_as(guid, &mut found)? {
        let current = live(&found, A::TYPE_NAME)?;
        if lower != lower_of(current) || upper != upper_of(current) {
            init(current, lower, upper)?;
        }
        return Ok(found);
    }
    let created = Handle::new(A::make());
    initialize_new(
        live(&created, A::TYPE_NAME)?,
        guid,
        (lower, upper),
        is_delta,
    )?;
    label.add_attribute(&created.cast(), false)?;
    Ok(created)
}

/// The `Init`, `SetDelta` and `SetID` calls of `SetAttr` on a new attribute.
fn initialize_new<A: ArrayAttribute>(
    created: &A,
    guid: &Guid,
    (lower, upper): (i32, i32),
    is_delta: bool,
) -> Result<(), Failure> {
    if A::SET_ID_FIRST {
        created.set_id(guid)?;
    }
    init(created, lower, upper)?;
    created.array_state().write().is_delta = is_delta;
    if !A::SET_ID_FIRST {
        created.set_id(guid)?;
    }
    Ok(())
}

/// `Init(lower, upper)`: backs up, then a new array filled with the class's fill value;
/// `Standard_RangeError` when `upper < lower`.
pub(crate) fn init<A: ArrayAttribute>(
    attribute: &A,
    lower: i32,
    upper: i32,
) -> Result<(), Failure> {
    if upper < lower {
        let message = format!(
            "{}::Init: upper {upper} is below lower {lower}",
            A::TYPE_NAME
        );
        return Err(exceptions::range_error(&message));
    }
    attribute.backup()?;
    let array = HArray1::with_value(lower, upper, (A::FILL)())
        .map_err(|error| error.into_failure(A::TYPE_NAME))?;
    attribute.array_state().write().value = Handle::new(array);
    Ok(())
}

/// `SetValue(index, value)`: nothing without array or when unchanged, else backs up and
/// writes the item.
pub(crate) fn set_value<A: ArrayAttribute>(
    attribute: &A,
    index: i32,
    value: A::Item,
) -> Result<(), Failure> {
    let shared = array(attribute);
    let Some(items) = shared.get() else {
        return Ok(());
    };
    if *items
        .array1()
        .value(index)
        .map_err(|e| e.into_failure(A::TYPE_NAME))?
        == value
    {
        return Ok(());
    }
    attribute.backup()?;
    let mut items = items.change_array1();
    items
        .set_value(index, value)
        .map_err(|e| e.into_failure(A::TYPE_NAME))
}

/// `Value(index)`: the class's neutral value without array.
pub(crate) fn value<A: ArrayAttribute>(attribute: &A, index: i32) -> Result<A::Item, Failure> {
    let shared = array(attribute);
    let Some(items) = shared.get() else {
        return Ok((A::NULL_VALUE)());
    };
    let items = items.array1();
    items
        .value(index)
        .cloned()
        .map_err(|e| e.into_failure(A::TYPE_NAME))
}

/// `Lower()`: 0 without array.
pub(crate) fn lower_of<A: ArrayAttribute>(attribute: &A) -> i32 {
    array(attribute)
        .get()
        .map_or(0, |items| items.array1().lower())
}

/// `Upper()`: the class's neutral bound without array.
pub(crate) fn upper_of<A: ArrayAttribute>(attribute: &A) -> i32 {
    array(attribute)
        .get()
        .map_or(A::NULL_UPPER, |items| items.array1().upper())
}

/// `Length()`: 0 without array.
pub(crate) fn length_of<A: ArrayAttribute>(attribute: &A) -> i32 {
    array(attribute)
        .get()
        .map_or(0, |items| items.array1().length())
}

/// `ChangeArray(newArray, isCheckItems)`: nothing when the bounds and (if asked) the items
/// are the same; else backs up and copies the items, into a new array when the bounds
/// differ. A null `newArray` is `Standard_NullObject` (a null dereference natively).
pub(crate) fn change_array<A: ArrayAttribute>(
    attribute: &A,
    new_array: &Handle<HArray1<A::Item>>,
    check_items: bool,
) -> Result<(), Failure> {
    let items: Vec<A::Item> = live(new_array, A::TYPE_NAME)?
        .array1()
        .iter()
        .cloned()
        .collect();
    let lower = live(new_array, A::TYPE_NAME)?.array1().lower();
    let upper = lower + items.len() as i32 - 1;
    let current = array(attribute);
    let guarded = A::CHANGE_GUARDS_NULL && current.is_null();
    let same_bounds = !guarded && lower_of(attribute) == lower && upper_of(attribute) == upper;
    if same_bounds && check_items && same_items::<A>(&current, lower, &items)? {
        return Ok(());
    }
    attribute.backup()?;
    let target = match current.get().filter(|_| same_bounds) {
        Some(_) => current.clone(),
        None => new_default_array::<A>(lower, upper)?,
    };
    copy_items::<A>(&target, lower, &items)?;
    attribute.array_state().write().value = target;
    Ok(())
}

/// The item loop of `ChangeArray`; `Standard_NullObject` when it would read a missing array.
fn same_items<A: ArrayAttribute>(
    current: &Handle<HArray1<A::Item>>,
    lower: i32,
    items: &[A::Item],
) -> Result<bool, Failure> {
    if items.is_empty() {
        return Ok(true);
    }
    let current = live(current, A::TYPE_NAME)?.array1();
    let differs = |(offset, item): (usize, &A::Item)| {
        current.value(lower + offset as i32).map(|old| old != item)
    };
    for pair in items.iter().enumerate() {
        if differs(pair).map_err(|e| e.into_failure(A::TYPE_NAME))? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `new NCollection_HArray1<T>(lower, upper)` (default items).
pub(crate) fn new_default_array<A: ArrayAttribute>(
    lower: i32,
    upper: i32,
) -> Result<Handle<HArray1<A::Item>>, Failure> {
    let items = HArray1::with_bounds(lower, upper).map_err(|e| e.into_failure(A::TYPE_NAME))?;
    Ok(Handle::new(items))
}

/// Writes `items` from index `lower` on.
fn copy_items<A: ArrayAttribute>(
    target: &Handle<HArray1<A::Item>>,
    lower: i32,
    items: &[A::Item],
) -> Result<(), Failure> {
    let mut target = live(target, A::TYPE_NAME)?.change_array1();
    for (offset, item) in items.iter().enumerate() {
        let index = lower + offset as i32;
        target
            .set_value(index, item.clone())
            .map_err(|e| e.into_failure(A::TYPE_NAME))?;
    }
    Ok(())
}

/// `Restore(With)`: a copy of the other array with its delta flag and ID, or no array.
pub(crate) fn restore<A: ArrayAttribute>(
    attribute: &A,
    with: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let other = cast_attribute::<A>(with, A::TYPE_NAME)?;
    let (other_array, is_delta, id) = {
        let state = live(&other, A::TYPE_NAME)?.array_state().read();
        (state.value.clone(), state.is_delta, state.id)
    };
    let Some(items) = other_array.get() else {
        attribute.array_state().write().value = Handle::default();
        return Ok(());
    };
    let copy = Handle::new(HArray1::from_array(&items.array1()));
    let mut state = attribute.array_state().write();
    (state.value, state.is_delta, state.id) = (copy, is_delta, id);
    Ok(())
}

/// `Paste(Into, RT)`: with an array, `Into` (when it is the same class) takes the items,
/// the delta flag and the ID.
pub(crate) fn paste<A: ArrayAttribute>(
    attribute: &A,
    into: &Handle<dyn TdfAttribute>,
) -> Result<(), Failure> {
    let (items, is_delta, id) = {
        let state = attribute.array_state().read();
        (state.value.clone(), state.is_delta, state.id)
    };
    let target = into.cast::<A>();
    let (false, Some(target)) = (items.is_null(), target.get()) else {
        return Ok(());
    };
    change_array(target, &items, false)?;
    target.array_state().write().is_delta = is_delta;
    target.set_id(&id)
}

/// `SetID(guid)`: backed up when it changes.
pub(crate) fn set_id<A: ArrayAttribute>(attribute: &A, guid: &Guid) -> Result<(), Failure> {
    if attribute.array_state().read().id == *guid {
        return Ok(());
    }
    attribute.backup()?;
    attribute.array_state().write().id = *guid;
    Ok(())
}

/// `SetID()`: backs up, then the default ID.
pub(crate) fn set_default_id<A: ArrayAttribute>(attribute: &A) -> Result<(), Failure> {
    attribute.backup()?;
    attribute.array_state().write().id = A::DEFAULT_ID;
    Ok(())
}

/// `DumpJson`: the class, the attribute base, the bounds and items, and the delta flag.
pub(crate) fn dump_json<A: ArrayAttribute>(
    attribute: &A,
    stream: &mut DumpStream,
    depth: i32,
) -> io::Result<()> {
    class_begin(stream, A::TYPE_NAME)?;
    dump_attribute_json_base(attribute, stream, depth)?;
    let shared = array(attribute);
    if let Some(items) = shared.get() {
        let items = items.array1();
        field_integer(stream, "myValue->Lower()", items.lower())?;
        field_integer(stream, "myValue->Upper()", items.upper())?;
        for item in items.iter() {
            A::dump_item(stream, item)?;
        }
    }
    let is_delta = attribute.array_state().read().is_delta;
    field_integer(stream, "myIsDelta", i32::from(is_delta))
}

/// The items of the array, in index order (empty without array); for `Dump`.
pub(crate) fn items_of<A: ArrayAttribute>(attribute: &A) -> Option<Vec<A::Item>> {
    let shared = array(attribute);
    let items = shared.get()?.array1().iter().cloned().collect();
    Some(items)
}

/// `" Delta is ON"` or `" Delta is OFF"`, the ID and a newline: the tail of `Dump`.
pub(crate) fn dump_delta_tail<A: ArrayAttribute>(
    attribute: &A,
    stream: &mut dyn io::Write,
) -> io::Result<()> {
    let (is_delta, id) = {
        let state = attribute.array_state().read();
        (state.is_delta, state.id)
    };
    let flag = if is_delta { "ON" } else { "OFF" };
    writeln!(stream, " Delta is {flag}{id}")
}
