// Rust port of ApplicationFramework/TKTObj/TObj/TObj_Object.cxx (GetChildren and the
// file-static addObjToOrderSequence) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The native body of `TObj_Object::GetChildren`: the child objects, sorted by
//! `GetOrder` when the object keeps its children ordered.
use super::{
    TObjBackReferences, TObjObject, TObjObjectIterator, TObjObjectState, TObjObjectStateFlags,
    TObjOcafObjectIterator, TObjSequenceIterator,
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Failure, Handle, RuntimeType},
};

/// The insertion cursor of `addObjToOrderSequence`: the 1-based index and the order of
/// the last placed object.
struct OrderCursor {
    last_index: usize,
    last_order: i32,
}

/// The order of the object at the 1-based `index`.
fn order_at(sequence: &Sequence<Handle<dyn TObjObject>>, index: usize) -> Result<i32, Failure> {
    let object = sequence.value(index)?;
    rustycad_tkcdf::live(object, "TObj_Object::GetChildren")?.get_order()
}

/// `addObjToOrderSequence` for a greater order: walks forward from the cursor and
/// inserts before the first object of a greater or equal order (appends past the end).
fn insert_forward(
    object: &Handle<dyn TObjObject>,
    order: i32,
    sequence: &mut Sequence<Handle<dyn TObjObject>>,
    length: usize,
    cursor: &mut OrderCursor,
) -> Result<(), Failure> {
    while order > cursor.last_order {
        cursor.last_index += 1;
        if cursor.last_index > length {
            sequence.append(object.clone());
            cursor.last_index = length + 1;
            cursor.last_order = order;
            return Ok(());
        }
        cursor.last_order = order_at(sequence, cursor.last_index)?;
    }
    sequence.insert_before(cursor.last_index, object.clone())?;
    cursor.last_order = order;
    Ok(())
}

/// `addObjToOrderSequence` for a lower or equal order: walks backward from the cursor
/// and inserts after the first object of a lower or equal order (prepends at the start).
fn insert_backward(
    object: &Handle<dyn TObjObject>,
    order: i32,
    sequence: &mut Sequence<Handle<dyn TObjObject>>,
    cursor: &mut OrderCursor,
) -> Result<(), Failure> {
    while order < cursor.last_order {
        if cursor.last_index <= 1 {
            sequence.insert_before(1, object.clone())?;
            cursor.last_index = 1;
            cursor.last_order = order;
            return Ok(());
        }
        cursor.last_index -= 1;
        cursor.last_order = order_at(sequence, cursor.last_index)?;
    }
    sequence.insert_after(cursor.last_index, object.clone())?;
    cursor.last_index += 1;
    cursor.last_order = order;
    Ok(())
}

/// The body of `TObj_Object::GetChildren(theType)`.
pub(crate) fn object_children<T: TObjObject + ?Sized>(
    object: &T,
    kind: Option<&'static RuntimeType>,
) -> Result<Handle<dyn TObjObjectIterator>, Failure> {
    let iterator: Handle<dyn TObjObjectIterator> =
        TObjOcafObjectIterator::new(&object.get_child_label()?, kind, true, false)?.cast();
    if !object.test_flags(TObjObjectStateFlags::ORDERED)? {
        return Ok(iterator);
    }
    let sequence = Handle::new(TObjBackReferences::default());
    let mut cursor: Option<OrderCursor> = None;
    let iterator = rustycad_tkcdf::live(&iterator, "TObj_Object::GetChildren")?;
    while iterator.more() {
        let child = iterator.value();
        if let Some(found) = child.get() {
            let order = found.get_order()?;
            place_child(&sequence, &child, order, &mut cursor)?;
        }
        iterator.next()?;
    }
    Ok(Handle::new(TObjSequenceIterator::new(&sequence, None)).cast())
}

/// Appends the first child, then places each next one with `addObjToOrderSequence`.
fn place_child(
    sequence: &Handle<TObjBackReferences>,
    child: &Handle<dyn TObjObject>,
    order: i32,
    cursor: &mut Option<OrderCursor>,
) -> Result<(), Failure> {
    sequence.change_sequence(|items| {
        if let Some(placed) = cursor.as_mut() {
            let length = items.len();
            return if order > placed.last_order {
                insert_forward(child, order, items, length, placed)
            } else {
                insert_backward(child, order, items, placed)
            };
        }
        items.append(child.clone());
        *cursor = Some(OrderCursor {
            last_index: 1,
            last_order: order,
        });
        Ok(())
    })
}
