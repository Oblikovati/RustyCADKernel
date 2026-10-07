// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming.cxx (FindUniqueContext,
// FindUniqueContextSet, OuterWire, OuterShell and their file-static helpers)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TNaming` context queries: the unique container of a selection, and the outer wire
//! of a face or shell of a solid.
use super::TNaming;
use super::package::compound_of;
use rustycad_tkbo::int_tools::FClass2d;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator, Shell, Solid, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::{DataMap, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// `NCollection_DataMap<TopoDS_Shape, TopoDS_Shape>` (the default hasher: `IsEqual`, so
/// orientation counts).
type ContextMap = DataMap<Shape, Shape>;

impl TNaming {
    /// `FindUniqueContext(S, Context)`: the container of `S` in `Context` (`S` must be found
    /// with its orientation), null when none.
    pub fn find_unique_context(selection: &Shape, context: &Shape) -> Result<Shape, Failure> {
        let mut map = ContextMap::default();
        build_map(context, selection.shape_type()?, &mut map)?;
        Ok(map.seek(selection).cloned().unwrap_or_default())
    }
    /// `FindUniqueContextSet(S, Context, Arr)`: for a compound selection, its container in
    /// `Context`, else the context when every item is in it directly, else the containers of
    /// the items (one, or a compound); `Arr` receives the container of each item. Null for
    /// another selection.
    pub fn find_unique_context_set(
        selection: &Shape,
        context: &Shape,
        containers: &mut Handle<HArray1<Shape>>,
    ) -> Result<Shape, Failure> {
        if selection.shape_type()? != ShapeEnum::Compound {
            return Ok(Shape::default());
        }
        let items: Vec<Shape> = ShapeIterator::new(selection, true, true).collect();
        let mut stop_type = ShapeEnum::Compound;
        for item in &items {
            let item_type = item.shape_type()?;
            if item_type as i32 > stop_type as i32 {
                stop_type = item_type;
            }
        }
        if !items.is_empty() {
            let size = i32::try_from(items.len()).unwrap_or(i32::MAX);
            *containers = Handle::new(HArray1::with_bounds(1, size).map_err(array_failure)?);
        }
        if stop_type == ShapeEnum::Shape {
            stop_type = selection.shape_type()?;
        }
        let mut map = ContextMap::default();
        build_map(context, stop_type, &mut map)?;
        if let Some(found) = map.seek(selection) {
            return Ok(found.clone());
        }
        containers_of_items(&items, context, &map, containers)
    }
    /// `OuterWire(theFace, theWire)`: the first wire of the face that is not a hole (each
    /// wire is classified alone in an empty copy of the face, by one classifier that keeps
    /// the wires of the earlier calls, as natively); `theWire` is the last wire tried.
    pub fn outer_wire(face: &topods::Face, wire: &mut Wire) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        let tolerance = BRepTool::face_tolerance(face)?;
        let mut classifier = FClass2d::default();
        let mut tried = Wire::default();
        let mut is_hole = false;
        for child in ShapeIterator::new(face, true, true) {
            tried = Wire::unchecked(child);
            let mut alone = face.as_ref().clone();
            alone.empty_copy()?;
            builder.add(&mut alone, &tried)?;
            classifier.init(&topods::Face::unchecked(alone), tolerance)?;
            is_hole = classifier.is_hole();
            if !is_hole {
                break;
            }
        }
        *wire = tried;
        Ok(!is_hole)
    }
    /// `OuterShell(theSolid, theShell)`: the first shell (not internal or external) whose
    /// solid alone leaves the infinite point outside; `theShell` is the last shell tried.
    pub fn outer_shell(solid: &Solid, shell: &mut Shell) -> Result<bool, Failure> {
        let builder = BRepBuilder::default();
        let mut classifier = BRepClass3dSolidClassifier::new();
        let mut tried = Shell::default();
        let mut found = false;
        for child in ShapeIterator::new(solid, true, true) {
            if child.shape_type()? != ShapeEnum::Shell || is_internal(&child) {
                continue;
            }
            tried = Shell::unchecked(child);
            let mut alone = solid.as_ref().clone();
            alone.empty_copy()?;
            builder.add(&mut alone, &tried)?;
            classifier.load(&alone)?;
            classifier.perform_infinite_point(1.0e-7)?;
            if classifier.state() == State::Out {
                found = true;
                break;
            }
        }
        *shell = tried;
        Ok(found)
    }
}

/// The failure of an array allocation.
fn array_failure(error: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    error.into_failure("TNaming::FindUniqueContextSet")
}

/// The compound branch of `FindUniqueContextSet`.
fn containers_of_items(
    items: &[Shape],
    context: &Shape,
    map: &ContextMap,
    containers: &Handle<HArray1<Shape>>,
) -> Result<Shape, Failure> {
    let mut seen = MapOfShape::default();
    let mut unique = Vec::new();
    let mut in_context = 0;
    for (index, item) in items.iter().enumerate() {
        let Some(container) = map.seek(item) else {
            continue;
        };
        if seen.add(container.clone()) {
            unique.push(container.clone());
        }
        if let Some(array) = containers.get() {
            let position = i32::try_from(index + 1).unwrap_or(i32::MAX);
            array
                .change_array1()
                .set_value(position, container.clone())
                .map_err(array_failure)?;
        }
        if container == context {
            in_context += 1;
        }
    }
    if items.len() == in_context && in_context > 0 {
        return Ok(context.clone());
    }
    if unique.len() == 1 {
        return Ok(unique[0].clone());
    }
    compound_of(unique.iter())
}

/// The file-static `IsInternal`: the first child is internal or external.
fn is_internal(shape: &Shape) -> bool {
    let mut children = ShapeIterator::new(shape, true, true);
    Iterator::next(&mut children).is_some_and(|child| {
        matches!(
            child.orientation(),
            Orientation::Internal | Orientation::External
        )
    })
}

/// The file-static `BuildMapIn(Context, StopType, Map)`: the sub-shapes of a solid or face
/// skip one level unless they are the stop type's direct parents; deeper levels recurse down
/// to the stop type (NMT fix: internal and external children are mapped too).
fn build_map_in(context: &Shape, stop: ShapeEnum, map: &mut ContextMap) -> Result<(), Failure> {
    let context_type = context.shape_type()? as i32;
    let skip = matches!(context.shape_type()?, ShapeEnum::Solid | ShapeEnum::Face)
        && stop as i32 - context_type != 1;
    let Some(kind) = ShapeEnum::ALL
        .get((context_type + if skip { 2 } else { 1 }) as usize)
        .copied()
    else {
        return Ok(());
    };
    let mut explorer = Explorer::new(context, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        let current = explorer.current().clone();
        map.bind(current.clone(), context.clone());
        if (current.shape_type()? as i32) < stop as i32 {
            build_map_in(&current, stop, map)?;
        }
        explorer.next()?;
    }
    if context_type < stop as i32 {
        for child in ShapeIterator::new(context, true, true) {
            if !matches!(
                child.orientation(),
                Orientation::Forward | Orientation::Reversed
            ) {
                map.bind(child, context.clone());
            }
        }
    }
    Ok(())
}

/// The file-static `BuildMapC0(Context, C0, StopType, Map)`: the children of a container map
/// to the top context.
fn build_map_c0(
    context: &Shape,
    top: &Shape,
    stop: ShapeEnum,
    map: &mut ContextMap,
) -> Result<(), Failure> {
    for key in ShapeIterator::new(context, true, true) {
        map.bind(key.clone(), top.clone());
        descend(&key, context, stop, map)?;
    }
    Ok(())
}

/// The file-static `BuildMap(Context, StopType, Map)`: the children of the context map to it.
fn build_map(context: &Shape, stop: ShapeEnum, map: &mut ContextMap) -> Result<(), Failure> {
    for key in ShapeIterator::new(context, true, true) {
        map.bind(key.clone(), context.clone());
        descend(&key, context, stop, map)?;
    }
    Ok(())
}

/// The common recursion of `BuildMap` and `BuildMapC0`: containers above solids go on with
/// `BuildMapC0` (natively with the current context as the top), solids and below with
/// `BuildMapIn`.
fn descend(
    key: &Shape,
    context: &Shape,
    stop: ShapeEnum,
    map: &mut ContextMap,
) -> Result<(), Failure> {
    let key_type = key.shape_type()? as i32;
    if key_type >= stop as i32 {
        return Ok(());
    }
    if key_type < ShapeEnum::Solid as i32 {
        return build_map_c0(key, context, stop, map);
    }
    build_map_in(key, stop, map)
}
