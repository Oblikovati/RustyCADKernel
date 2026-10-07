// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_UsedShapes.hxx, .lxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_UsedShapes`: the root attribute that maps every named shape of a framework to
//! its `TNaming_RefShape`.
use super::TNamingRefShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkcdf::json_dump::class_begin;
use rustycad_tkernel::{
    impl_transient,
    ncollection::DataMap,
    standard::{
        Failure, Guid, Handle, RuntimeTyped, SharedState, StandardTransient, dump::DumpStream,
    },
};
use rustycad_tklcaf::guid_from_text;
use rustycad_tklcaf::json::dump_attribute_json_base;
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeBase, TdfAttributeDelta, TdfDataSet, TdfDeltaOnAddition,
    TdfDeltaOnRemoval, TdfRelocationTable,
};
use std::io;
use std::sync::{Arc, RwLockWriteGuard};

/// `TNaming_UsedShapes::GetID()`.
const USED_SHAPES_ID: Guid = guid_from_text("c4ef4201-568f-11d1-8940-080009dc3333");

/// `NCollection_DataMap<TopoDS_Shape, TNaming_RefShape*, TopTools_ShapeMapHasher>`.
pub type TNamingShapeRefMap = DataMap<Shape, Arc<TNamingRefShape>, ShapeMapHasher>;

/// `TNaming_UsedShapes`: created on the root label by the first `TNaming_Builder`.
///
/// ```
/// use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingUsedShapes};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// TNamingBuilder::new(&data.root()).unwrap();
/// let mut used: Handle<TNamingUsedShapes> = Handle::default();
/// assert!(data.root().find_attribute_as(&TNamingUsedShapes::get_id(), &mut used).unwrap());
/// assert!(used.map().is_empty());
/// ```
pub struct TNamingUsedShapes {
    transient: StandardTransient,
    base: TdfAttributeBase,
    map: SharedState<TNamingShapeRefMap>,
}
impl_transient!(TNamingUsedShapes, "TNaming_UsedShapes", dyn TdfAttribute, transient, interfaces: [dyn TdfAttribute]);

impl TNamingUsedShapes {
    /// `GetID`.
    pub fn get_id() -> Guid {
        USED_SHAPES_ID
    }
    /// The private constructor (reached by `TNaming_Builder` and `NewEmpty`).
    pub(crate) fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            base: TdfAttributeBase::default(),
            map: SharedState::new(TNamingShapeRefMap::default()),
        }
    }
    /// `Map()`: the map itself; drop the guard before calling back into the framework.
    pub fn map(&self) -> RwLockWriteGuard<'_, TNamingShapeRefMap> {
        self.map.write()
    }
    /// `Destroy`: clears the map (the ref shapes are not deleted natively; the nodes
    /// still reach them).
    pub fn destroy(&self) {
        let released = std::mem::take(&mut *self.map.write());
        drop(released);
    }
}

impl Drop for TNamingUsedShapes {
    /// `~TNaming_UsedShapes`: `Destroy()`.
    fn drop(&mut self) {
        self.destroy();
    }
}

impl TdfAttribute for TNamingUsedShapes {
    fn attribute_base(&self) -> &TdfAttributeBase {
        &self.base
    }
    fn id(&self) -> Guid {
        USED_SHAPES_ID
    }
    /// `BackupCopy`: a null handle (the map is never backed up).
    fn backup_copy(&self) -> Result<Handle<dyn TdfAttribute>, Failure> {
        Ok(Handle::default())
    }
    /// `Restore`: does nothing.
    fn restore(&self, _attribute: &Handle<dyn TdfAttribute>) -> Result<(), Failure> {
        Ok(())
    }
    /// `BeforeRemoval`: `Destroy()`.
    fn before_removal(&self) -> Result<(), Failure> {
        self.destroy();
        Ok(())
    }
    /// `AfterUndo`: an undone addition calls the attribute's `BeforeRemoval`.
    fn after_undo(
        &self,
        delta: &Handle<dyn TdfAttributeDelta>,
        _force_it: bool,
    ) -> Result<bool, Failure> {
        if delta.is_kind(TdfDeltaOnAddition::static_type()) {
            let delta = rustycad_tklcaf::live(delta, "TNaming_UsedShapes::AfterUndo")?;
            let attribute = delta.attribute();
            rustycad_tklcaf::live(&attribute, "TNaming_UsedShapes::AfterUndo")?.before_removal()?;
        }
        Ok(true)
    }
    /// `DeltaOnAddition`: a null delta (nothing to undo).
    fn delta_on_addition(&self) -> Result<Handle<TdfDeltaOnAddition>, Failure> {
        Ok(Handle::default())
    }
    /// `DeltaOnRemoval`: a null delta.
    fn delta_on_removal(&self) -> Result<Handle<dyn TdfDeltaOnRemoval>, Failure> {
        Ok(Handle::default())
    }
    fn new_empty(&self) -> Handle<dyn TdfAttribute> {
        Handle::new(Self::new()).cast()
    }
    /// `Paste`: does nothing.
    fn paste(
        &self,
        _into: &Handle<dyn TdfAttribute>,
        _relocation_table: &Handle<TdfRelocationTable>,
    ) -> Result<(), Failure> {
        Ok(())
    }
    /// `References`: adds nothing.
    fn references(&self, _data_set: &Handle<TdfDataSet>) {}
    /// `Dump`: one line per shape of the map, in map order.
    fn dump(&self, stream: &mut dyn io::Write) -> io::Result<()> {
        writeln!(stream, "The content of UsedShapes attribute:")?;
        let entries: Vec<(Shape, Arc<TNamingRefShape>)> = self
            .map
            .read()
            .items()
            .map(|(shape, reference)| (shape.clone(), reference.clone()))
            .collect();
        for (shape, reference) in &entries {
            dump_used_shape(stream, shape, reference)?;
        }
        Ok(())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "TNaming_UsedShapes")?;
        dump_attribute_json_base(self, stream, depth)
    }
}

/// One line of `TNaming_UsedShapes::Dump`.
fn dump_used_shape(
    stream: &mut dyn io::Write,
    shape: &Shape,
    reference: &TNamingRefShape,
) -> io::Result<()> {
    let label = reference.label().map_err(io::Error::other)?;
    if shape.is_null() {
        write!(stream, "Empty Shape at label =")?;
        label.entry_dump(stream)?;
        return writeln!(stream);
    }
    write!(stream, "  ")?;
    let shape_type = shape.shape_type().map_err(io::Error::other)?;
    rustycad_tkg3d::top_abs::print_shape_type(shape_type, &mut &mut *stream)?;
    write!(stream, "  ")?;
    label.entry_dump(stream)?;
    write!(
        stream,
        " Key_TShape   = {:p}",
        shape.tshape().object_address()
    )?;
    let value = reference.shape();
    writeln!(
        stream,
        " Value_TShape = {:p}",
        value.tshape().object_address()
    )
}
