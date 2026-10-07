// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_CopyShape.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_CopyShape`: a deep copy of a shape that shares its geometry, recording each
//! copied TShape and location datum in a transient map.
use super::TNamingTranslateTool;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, TShape};
use rustycad_tkernel::ncollection::IndexedDataMap;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::toploc::{Datum3, Location};

/// `NCollection_IndexedDataMap<occ::handle<Standard_Transient>, occ::handle<Standard_Transient>>`:
/// source TShape (or datum) to its copy.
pub type TNamingTransientMap = IndexedDataMap<Handle<StandardTransient>, Handle<StandardTransient>>;

/// `TNaming_CopyShape`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkcaf::tnaming::{TNamingCopyShape, TNamingTransientMap};
/// let mut map = TNamingTransientMap::default();
/// let mut copy = Shape::default();
/// TNamingCopyShape::copy_tool(&Shape::default(), &mut map, &mut copy).unwrap();
/// assert!(copy.is_null() && map.is_empty());
/// ```
pub struct TNamingCopyShape;

impl TNamingCopyShape {
    /// `CopyTool(aShape, aMap, aResult)`: `Translate` with a new `TNaming_TranslateTool`.
    pub fn copy_tool(
        shape: &Shape,
        map: &mut TNamingTransientMap,
        result: &mut Shape,
    ) -> Result<(), Failure> {
        let tool = Handle::new(TNamingTranslateTool::new());
        Self::translate(shape, map, result, &tool)
    }
    /// `Translate(aShape, aMap, aResult, TrTool)`: the copy of `shape`; a TShape already in
    /// the map is reused, a new one is made by the tool and its sub-shapes are translated.
    pub fn translate(
        shape: &Shape,
        map: &mut TNamingTransientMap,
        result: &mut Shape,
        tool: &Handle<TNamingTranslateTool>,
    ) -> Result<(), Failure> {
        result.nullify();
        if shape.is_null() {
            return Ok(());
        }
        let tool_ref = rustycad_tklcaf::live(tool, "TNaming_CopyShape::Translate")?;
        let key: Handle<StandardTransient> = shape.tshape().cast();
        if let Some(copied) = map.seek(&key) {
            result.set_tshape(&copied.cast::<dyn TShape>());
        } else {
            make_translated(shape, map, result, tool, tool_ref)?;
        }
        result.set_orientation(shape.orientation());
        result.set_location(&Self::translate_location(shape.location(), map)?, false)?;
        tool_ref.update_shape(shape, result)
    }
    /// `Translate(L, aMap)`: the location rebuilt on translated datums (a datum already in
    /// the map is reused).
    pub fn translate_location(
        location: &Location,
        map: &mut TNamingTransientMap,
    ) -> Result<Location, Failure> {
        if location.is_identity() {
            return Ok(Location::default());
        }
        let next = Self::translate_location(location.next_location(), map)?;
        let datum = translate_datum(location.first_datum()?, map);
        let first = Location::from_datum(datum).powered(location.first_power()?)?;
        next.multiplied(&first)
    }
}

/// The branch of `Translate` for a TShape not translated yet.
fn make_translated(
    shape: &Shape,
    map: &mut TNamingTransientMap,
    result: &mut Shape,
    tool: &Handle<TNamingTranslateTool>,
    tool_ref: &TNamingTranslateTool,
) -> Result<(), Failure> {
    make_empty(shape, map, result, tool_ref)?;
    map.add(shape.tshape().cast(), result.tshape().cast());
    let mut source = shape.clone();
    source.set_orientation(Orientation::Forward);
    source.set_location(&Location::default(), true)?;
    let was_free = result.free()?;
    result.set_free(true)?;
    let mut iterator = ShapeIterator::new(&source, false, true);
    while iterator.more() {
        let mut sub_shape = Shape::default();
        TNamingCopyShape::translate(iterator.value()?, map, &mut sub_shape, tool)?;
        tool_ref.add(result, &sub_shape)?;
        iterator.next();
    }
    result.set_free(was_free)
}

/// The `switch` of `Translate`: an empty shape of the source type with its BRep data.
fn make_empty(
    shape: &Shape,
    map: &mut TNamingTransientMap,
    result: &mut Shape,
    tool: &TNamingTranslateTool,
) -> Result<(), Failure> {
    match shape.shape_type()? {
        ShapeEnum::Vertex => {
            tool.make_vertex(result)?;
            tool.update_vertex(shape, result, map)
        }
        ShapeEnum::Edge => {
            tool.make_edge(result)?;
            tool.update_edge(shape, result, map)
        }
        ShapeEnum::Face => {
            tool.make_face(result)?;
            tool.update_face(shape, result, map)
        }
        kind => make_container(kind, shape, result, tool),
    }
}

/// The container shapes of the `switch` (the flags only).
fn make_container(
    kind: ShapeEnum,
    shape: &Shape,
    result: &mut Shape,
    tool: &TNamingTranslateTool,
) -> Result<(), Failure> {
    match kind {
        ShapeEnum::Wire => tool.make_wire(result)?,
        ShapeEnum::Shell => tool.make_shell(result)?,
        ShapeEnum::Solid => tool.make_solid(result)?,
        ShapeEnum::CompSolid => tool.make_comp_solid(result)?,
        ShapeEnum::Compound => tool.make_compound(result)?,
        _ => return Ok(()),
    }
    tool.update_shape(shape, result)
}

/// The file-static `TranslateDatum3D`.
fn translate_datum(datum: &Handle<Datum3>, map: &mut TNamingTransientMap) -> Handle<Datum3> {
    let key: Handle<StandardTransient> = datum.cast();
    if let Some(copied) = map.seek(&key) {
        return copied.cast::<Datum3>();
    }
    let copy = Handle::new(Datum3::new(
        datum.get().map(|d| *d.transformation()).unwrap_or_default(),
    ));
    map.add(key, copy.cast());
    copy
}
