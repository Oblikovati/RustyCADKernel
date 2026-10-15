// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeMapTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_ShapeMapTool`: the map of the sub-shapes of a shape label.
use super::support::find_or_add;
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkcdf::json_dump::field_pointer;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle, SharedState, StandardTransient};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeBase, TdfLabel, TdfRelocationTable};
use rustycad_tkservice::json_dump::class_begin;
use std::io;

/// `XCAFDoc_ShapeMapTool::GetID()`.
const SHAPE_MAP_TOOL_ID: Guid = guid_from_text("3B913F4D-4A82-44ef-A0BF-9E01E9FF317A");

/// `XCAFDoc_ShapeMapTool`.
///
/// ```
/// use rustycad_tkbrep::topods::{Builder, Compound};
/// use rustycad_tklcaf::tdf::TdfData;
/// use rustycad_tkxcaf::xcaf_doc::XCAFDocShapeMapTool;
/// let data = TdfData::new();
/// let tool = XCAFDocShapeMapTool::set(&data.root())?;
/// let mut compound = Compound::default();
/// Builder.make_compound(&mut compound);
/// tool.set_shape(&compound);
/// assert_eq!(tool.get_map().extent(), 0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Default)]
pub struct XCAFDocShapeMapTool {
    transient: StandardTransient,
    base: TdfAttributeBase,
    map: SharedState<IndexedMapOfShape>,
}
rustycad_tkernel::impl_transient!(XCAFDocShapeMapTool, "XCAFDoc_ShapeMapTool", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

/// `AddSubShape`: the shape and, depth first, every sub-shape.
fn add_sub_shape(shape: &Shape, map: &mut IndexedMapOfShape) {
    map.add(shape.clone());
    let mut iterator = ShapeIterator::new(shape, true, true);
    while let Ok(child) = iterator.value() {
        add_sub_shape(child, map);
        iterator.next();
    }
}

impl XCAFDocShapeMapTool {
    /// `XCAFDoc_ShapeMapTool()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `GetID`.
    pub fn get_id() -> Guid {
        SHAPE_MAP_TOOL_ID
    }
    /// `Set(L)`: the tool of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocShapeMapTool>, Failure> {
        find_or_add(label, &SHAPE_MAP_TOOL_ID, Self::new)
    }
    /// `IsSubShape(sub)`: the shape is in the map.
    pub fn is_sub_shape(&self, sub: &Shape) -> bool {
        self.map.read().contains(sub)
    }
    /// `SetShape(S)`: the map of the sub-shapes of the shape (not the shape itself); not
    /// backed up, as natively.
    pub fn set_shape(&self, shape: &Shape) {
        let mut map = IndexedMapOfShape::default();
        let mut iterator = ShapeIterator::new(shape, true, true);
        while let Ok(child) = iterator.value() {
            add_sub_shape(child, &mut map);
            iterator.next();
        }
        *self.map.write() = map;
    }
    /// `GetMap`: a copy of the map.
    pub fn get_map(&self) -> IndexedMapOfShape {
        self.map.read().clone()
    }
}

impl TdfAttribute for XCAFDocShapeMapTool {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        SHAPE_MAP_TOOL_ID
    }
    /// `Restore`: does nothing.
    fn restore(&self, _with: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        Ok(())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: does nothing.
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `DumpJson`: the address of the TShape of each mapped shape.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "XCAFDoc_ShapeMapTool")?;
        dump_attribute_json_base(self, stream, depth)?;
        let map = self.get_map();
        for shape in map.iter() {
            field_pointer(stream, "aShape", shape.tshape().object_address())?;
        }
        Ok(())
    }
}
