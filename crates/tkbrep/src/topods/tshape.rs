// Created on: 1990-12-13
// Created by: Remi Lequette
// Copyright (c) 1990-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingData/TKBRep/TopoDS/TopoDS_TShape.hxx and
// TopoDS_TShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopoDS_TShape`: the shared topology that shapes reference (docs/topology-design.md
//! sections 2 to 4).
use super::Shape;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::dump::{self, DumpOutput, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient, Transient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::io;

/// `TopoDS_TShape::BitLayout`: the layout of the 16-bit state word. Bits 0-3 hold the
/// `TopAbs_ShapeEnum` value, bits 4-11 the flags, bits 12-15 are reserved.
pub mod bit_layout {
    /// `Bits_ShapeType_Mask`: bits 0-3, the shape type.
    pub const BITS_SHAPE_TYPE_MASK: u16 = 0x000F;
    /// `Bits_ShapeType_Shift`: the shape type is not shifted.
    pub const BITS_SHAPE_TYPE_SHIFT: u16 = 0;
    /// `Bit_Free`: bit 4.
    pub const BIT_FREE: u16 = 0x0010;
    /// `Bit_Modified`: bit 5.
    pub const BIT_MODIFIED: u16 = 0x0020;
    /// `Bit_Checked`: bit 6.
    pub const BIT_CHECKED: u16 = 0x0040;
    /// `Bit_Orientable`: bit 7.
    pub const BIT_ORIENTABLE: u16 = 0x0080;
    /// `Bit_Closed`: bit 8.
    pub const BIT_CLOSED: u16 = 0x0100;
    /// `Bit_Infinite`: bit 9.
    pub const BIT_INFINITE: u16 = 0x0200;
    /// `Bit_Convex`: bit 10.
    pub const BIT_CONVEX: u16 = 0x0400;
    /// `Bit_Locked`: bit 11.
    pub const BIT_LOCKED: u16 = 0x0800;
    /// `Bits_Reserved`: bits 12-15.
    pub const BITS_RESERVED: u16 = 0xF000;
}
use bit_layout::*;

/// The locked fields of a TShape: the private `myShapes` list and `myState` word.
#[derive(Clone, Default)]
struct TShapeFields {
    shapes: Vec<Shape>,
    state: u16,
}

/// The data of the abstract `TopoDS_TShape` that every concrete TShape embeds; the
/// [`TShape`] trait reads it through `tshape_base()`.
///
/// ```
/// use rustycad_tkbrep::topods::TShapeBase;
/// use rustycad_tkg3d::top_abs::ShapeEnum;
/// let base = TShapeBase::new(ShapeEnum::Wire);
/// assert_eq!(base.state() & 0xF, ShapeEnum::Wire as u16);
/// ```
#[derive(Default)]
pub struct TShapeBase {
    fields: SharedState<TShapeFields>,
}

impl TShapeBase {
    /// The protected `TopoDS_TShape(theType)`: the type in bits 0-3 and the flags Free,
    /// Modified and Orientable; `TShapeBase::new(ShapeEnum::Face)`.
    pub fn new(shape_type: ShapeEnum) -> Self {
        let state = shape_type as u16 | BIT_FREE | BIT_MODIFIED | BIT_ORIENTABLE;
        Self {
            fields: SharedState::new(TShapeFields {
                shapes: Vec::new(),
                state,
            }),
        }
    }

    /// The compact state word (`myState`, dumped as `State`); `base.state()`.
    pub fn state(&self) -> u16 {
        self.fields.read().state
    }

    /// The protected `setBit`: sets or clears `bit`; `base.set_bit(bit_layout::BIT_CLOSED, true)`.
    pub fn set_bit(&self, bit: u16, on: bool) {
        let mut fields = self.fields.write();
        fields.state = with_bit(fields.state, bit, on);
    }

    fn bit(&self, bit: u16) -> bool {
        self.state() & bit != 0
    }

    /// `Modified(true)` also clears Checked, in one locked update.
    fn set_modified(&self, on: bool) {
        let mut fields = self.fields.write();
        fields.state = modified_state(fields.state, on);
    }

    /// The child at `index` (0-based), the list read by `TopoDS_Iterator`.
    pub(crate) fn child(&self, index: usize) -> Option<Shape> {
        self.fields.read().shapes.get(index).cloned()
    }

    /// `myShapes.Length()`.
    fn child_count(&self) -> usize {
        self.fields.read().shapes.len()
    }

    /// `myShapes.Append(aChild); Modified(true)` of `TopoDS_Builder::Add`.
    pub(crate) fn append_child(&self, child: Shape) {
        let mut fields = self.fields.write();
        fields.shapes.push(child);
        fields.state = modified_state(fields.state, true);
    }

    /// The removal loop of `TopoDS_Builder::Remove`: removes the first child equal to
    /// `child` and marks the TShape modified; false when no child is equal.
    pub(crate) fn remove_child(&self, child: &Shape) -> bool {
        let mut fields = self.fields.write();
        let Some(position) = fields.shapes.iter().position(|shape| shape == child) else {
            return false;
        };
        fields.shapes.remove(position);
        fields.state = modified_state(fields.state, true);
        true
    }
}

fn with_bit(state: u16, bit: u16, on: bool) -> u16 {
    if on { state | bit } else { state & !bit }
}

/// "when a TShape is modified it is also unchecked".
fn modified_state(state: u16, on: bool) -> u16 {
    let state = with_bit(state, BIT_MODIFIED, on);
    if on {
        with_bit(state, BIT_CHECKED, false)
    } else {
        state
    }
}

/// `TopoDS_TShape`: a topological structure made of other shapes; abstract, because
/// `EmptyCopy` is pure virtual. The flag and type members are the native non-virtual
/// members, written once over [`TShapeBase`].
///
/// ```
/// use rustycad_tkbrep::topods::{TShape, TWire};
/// let wire = TWire::new();
/// assert!(wire.free() && wire.modified() && wire.orientable());
/// wire.set_checked(true);
/// wire.set_modified(true);
/// assert!(!wire.checked());
/// ```
pub trait TShape: Transient {
    /// The embedded `TopoDS_TShape` data.
    fn tshape_base(&self) -> &TShapeBase;

    /// `EmptyCopy()`: a new TShape of the same class with the same geometry and no
    /// sub-shapes. It returns `Result` because a derived class can raise from it
    /// (`BRep_TEdge::EmptyCopy` copies its curve representations, which evaluates their
    /// curves); `tshape.empty_copy()?`.
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure>;

    /// `DumpJson(theOStream, theDepth)`: the static class name `TopoDS_TShape`, the
    /// object address, the type, the child count, the state word and the eight flags;
    /// `tshape.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, output: &mut DumpStream, _depth: i32) -> io::Result<()> {
        let address = std::ptr::from_ref(self).cast::<()>() as usize;
        dump_tshape(self.tshape_base(), address, output)
    }

    /// `Free()`; `tshape.free()`.
    fn free(&self) -> bool {
        self.tshape_base().bit(BIT_FREE)
    }
    /// `Free(theIsFree)`; `tshape.set_free(false)`.
    fn set_free(&self, is_free: bool) {
        self.tshape_base().set_bit(BIT_FREE, is_free);
    }
    /// `Locked()`; `tshape.locked()`.
    fn locked(&self) -> bool {
        self.tshape_base().bit(BIT_LOCKED)
    }
    /// `Locked(theIsLocked)`; `tshape.set_locked(true)`.
    fn set_locked(&self, is_locked: bool) {
        self.tshape_base().set_bit(BIT_LOCKED, is_locked);
    }
    /// `Modified()`; `tshape.modified()`.
    fn modified(&self) -> bool {
        self.tshape_base().bit(BIT_MODIFIED)
    }
    /// `Modified(theIsModified)`: setting it also clears Checked; `tshape.set_modified(true)`.
    fn set_modified(&self, is_modified: bool) {
        self.tshape_base().set_modified(is_modified);
    }
    /// `Checked()`; `tshape.checked()`.
    fn checked(&self) -> bool {
        self.tshape_base().bit(BIT_CHECKED)
    }
    /// `Checked(theIsChecked)`; `tshape.set_checked(true)`.
    fn set_checked(&self, is_checked: bool) {
        self.tshape_base().set_bit(BIT_CHECKED, is_checked);
    }
    /// `Orientable()`; `tshape.orientable()`.
    fn orientable(&self) -> bool {
        self.tshape_base().bit(BIT_ORIENTABLE)
    }
    /// `Orientable(theIsOrientable)`; `tshape.set_orientable(false)`.
    fn set_orientable(&self, is_orientable: bool) {
        self.tshape_base().set_bit(BIT_ORIENTABLE, is_orientable);
    }
    /// `Closed()`; `tshape.closed()`.
    fn closed(&self) -> bool {
        self.tshape_base().bit(BIT_CLOSED)
    }
    /// `Closed(theIsClosed)`; `tshape.set_closed(true)`.
    fn set_closed(&self, is_closed: bool) {
        self.tshape_base().set_bit(BIT_CLOSED, is_closed);
    }
    /// `Infinite()`; `tshape.infinite()`.
    fn infinite(&self) -> bool {
        self.tshape_base().bit(BIT_INFINITE)
    }
    /// `Infinite(theIsInfinite)`; `tshape.set_infinite(true)`.
    fn set_infinite(&self, is_infinite: bool) {
        self.tshape_base().set_bit(BIT_INFINITE, is_infinite);
    }
    /// `Convex()`; `tshape.convex()`.
    fn convex(&self) -> bool {
        self.tshape_base().bit(BIT_CONVEX)
    }
    /// `Convex(theIsConvex)`; `tshape.set_convex(true)`.
    fn set_convex(&self, is_convex: bool) {
        self.tshape_base().set_bit(BIT_CONVEX, is_convex);
    }
    /// `ShapeType()`: the type stored in bits 0-3; `tshape.shape_type()`.
    fn shape_type(&self) -> ShapeEnum {
        stored_shape_type(self.tshape_base().state())
    }
    /// `NbChildren()`: the number of direct sub-shapes; `tshape.nb_children()`.
    fn nb_children(&self) -> i32 {
        self.tshape_base().child_count() as i32
    }
}
impl_transient_interface!(dyn TShape, "TopoDS_TShape", StandardTransient);

/// `static_cast<TopAbs_ShapeEnum>(myState & Bits_ShapeType_Mask)`. The constructor only
/// stores a `TopAbs_ShapeEnum` and no member changes those bits, so the value is one
/// of the nine enumerators.
fn stored_shape_type(state: u16) -> ShapeEnum {
    ShapeEnum::ALL[usize::from(state & BITS_SHAPE_TYPE_MASK)]
}

/// The body of `TopoDS_TShape::DumpJson` for the TShape at `address`, also called by
/// the BRep TShapes (`OCCT_DUMP_BASE_CLASS`).
pub(crate) fn dump_tshape(
    base: &TShapeBase,
    address: usize,
    output: &mut DumpStream,
) -> io::Result<()> {
    let state = base.state();
    dump::dump_class_begin(output, "TopoDS_TShape")?;
    let pointer = dump::get_pointer_info(Some(address), true).map_err(io::Error::other)?;
    dump_text_field(output, "this", pointer.as_bytes())?;
    dump::dump_integer(output, "ShapeType", stored_shape_type(state) as i32)?;
    dump::dump_integer(output, "NbChildren", base.child_count() as i32)?;
    dump::dump_integer(output, "State", i32::from(state))?;
    let flags = [
        ("Free", BIT_FREE),
        ("Locked", BIT_LOCKED),
        ("Modified", BIT_MODIFIED),
        ("Checked", BIT_CHECKED),
        ("Orientable", BIT_ORIENTABLE),
        ("Closed", BIT_CLOSED),
        ("Infinite", BIT_INFINITE),
        ("Convex", BIT_CONVEX),
    ];
    for (name, bit) in flags {
        dump::dump_integer(output, name, i32::from(state & bit != 0))?;
    }
    Ok(())
}

/// `OCCT_DUMP_FIELD_VALUE_POINTER`: `"name": "text"` after a separator.
fn dump_text_field(output: &mut DumpStream, name: &str, text: &[u8]) -> io::Result<()> {
    dump::add_values_separator(output)?;
    output.write_fragment(format!("\"{name}\": \"").as_bytes())?;
    output.write_fragment(text)?;
    output.write_fragment(b"\"")
}
