// Rust port of the file-static checks of ApplicationFramework/TKCAF/TNaming/TNaming_Naming.cxx
// (BuildNS, FindIndex, CompareInGeneration, GetShapeEvolutions, CompareInModification,
// FillSMap, Compare, TestSolution, FindNewShapeInFather, NextModif, IsContSame,
// IsMultipleCase) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks the naming builder runs on candidate names.
use super::{
    TNamingBuilder, TNamingEvolution, TNamingIterator, TNamingNameType, TNamingNamedShape,
    TNamingNaming, TNamingNamingTool, TNamingNewShapeIterator, TNamingScope, TNamingTool,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::{DataMap, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfLabelMap};

/// The file-static `BuildNS(F, S, Name)`: a new child naming of `F` that selects `S` with the
/// given name type.
pub(crate) fn build_ns(
    father: &TdfLabel,
    shape: &Shape,
    name_type: TNamingNameType,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let naming = TNamingNaming::insert(father)?;
    let naming = live(&naming, "BuildNS")?;
    {
        let mut name = naming.change_name();
        name.set_shape_type(shape.shape_type()?);
        name.set_shape(shape);
        name.set_orientation(shape.orientation());
        name.set_type(name_type);
    }
    let builder = TNamingBuilder::new(&naming.label())?;
    builder.select(shape, shape)?;
    Ok(builder.named_shape())
}

/// The file-static `FindIndex(NS, S)`: the 1-based position of the initial shape of `S`
/// among the new shapes of `NS` (one past the end when absent).
pub(crate) fn find_index(named: &Handle<TNamingNamedShape>, shape: &Shape) -> Result<i32, Failure> {
    let mut labels = Vec::new();
    let initial =
        TNamingTool::initial_shape(shape, &live(named, "FindIndex")?.label(), &mut labels)?;
    let mut index = 1;
    let mut iterator = TNamingIterator::new(named)?;
    while iterator.more() {
        if initial.is_same(&iterator.new_shape()?) {
            break;
        }
        iterator.next()?;
        index += 1;
    }
    Ok(index)
}

/// The file-static `CompareInGeneration(NS, S)`: every new shape of `NS` is `S`.
pub(crate) fn compare_in_generation(
    named: &Handle<TNamingNamedShape>,
    shape: &Shape,
) -> Result<bool, Failure> {
    let mut iterator = TNamingIterator::new(named)?;
    while iterator.more() {
        if !iterator.new_shape()?.is_same(shape) {
            return Ok(false);
        }
        iterator.next()?;
    }
    Ok(true)
}

/// The file-static `GetShapeEvolutions(theTarget, theSource, aList)`: the backward chain of
/// modified shapes from the target up to a shape of the source label (the list is filled in
/// backward order).
fn get_shape_evolutions(
    target: &Shape,
    source: &Handle<TNamingNamedShape>,
    list: &mut Vec<Shape>,
) -> Result<bool, Failure> {
    let source_label = live(source, "GetShapeEvolutions")?.label();
    let made_by = TNamingTool::named_shape(target, &source_label)?;
    let Some(made_by_ref) = made_by.get() else {
        return Ok(false);
    };
    // The target is in the source.
    if made_by_ref.label() == source_label {
        return Ok(true);
    }
    let mut iterator = TNamingIterator::new(&made_by)?;
    while iterator.more() {
        let (old, new) = (iterator.old_shape()?, iterator.new_shape()?);
        if !old.is_null()
            && !new.is_null()
            && new.is_same(target)
            && get_shape_evolutions(&old, source, list)?
        {
            list.push(target.clone());
            return Ok(true);
        }
        iterator.next()?;
    }
    Ok(false)
}

/// The file-static `CompareInModification(NS, S)`: when `S` comes from the parent through
/// modifications and one of them splits a shape in two on the same label, the named shape of
/// the split shape (the name then needs more detail); null otherwise.
pub(crate) fn compare_in_modification(
    parent: &Handle<TNamingNamedShape>,
    target: &Shape,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let Some(parent_ref) = parent.get().filter(|_| !target.is_null()) else {
        return Ok(Handle::default());
    };
    let parent_label = parent_ref.label();
    let mut source: Handle<TNamingNamedShape> = Handle::default();
    let mut iterator = TNamingIterator::new(parent)?;
    while iterator.more() && source.is_null() {
        let new = iterator.new_shape()?;
        if !new.is_null() {
            source = TNamingTool::named_shape(&new, &parent_label)?;
        }
        iterator.next()?;
    }
    // Searching for 1:n modifications on one label (the current naming is insufficient).
    let mut list = Vec::new();
    if !get_shape_evolutions(target, &source, &mut list)? || list.is_empty() {
        return Ok(Handle::default());
    }
    for shape in &list {
        let result = TNamingTool::named_shape(shape, &parent_label)?;
        if live(&result, "CompareInModification")?.evolution() != TNamingEvolution::Modify {
            // The evolution must be a modification, otherwise everything is OK.
            return Ok(Handle::default());
        }
        let split = split_origin(&result, shape, &parent_label)?;
        if split.is_some() {
            return Ok(split);
        }
    }
    Ok(Handle::default())
}

/// The inner loops of `CompareInModification`: the named shape of an old shape of `shape`
/// that another evolution of the same named shape also modified.
fn split_origin(
    result: &Handle<TNamingNamedShape>,
    shape: &Shape,
    label: &TdfLabel,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    let mut olds = MapOfShape::default();
    let mut iterator = TNamingIterator::new(result)?;
    while iterator.more() {
        if iterator.new_shape()?.is_same(shape) {
            olds.add(iterator.old_shape()?);
        }
        iterator.next()?;
    }
    let mut others = TNamingIterator::new(result)?;
    while others.more() {
        if !others.new_shape()?.is_same(shape) && olds.contains(&others.old_shape()?) {
            let origin = TNamingTool::named_shape(&others.old_shape()?, label)?;
            if origin.is_some() {
                return Ok(origin);
            }
        }
        others.next()?;
    }
    Ok(Handle::default())
}

/// The file-static `FillSMap(S, MS)`: the simple shapes of a (nested) compound; true when
/// they are all of one type.
pub(crate) fn fill_s_map(shape: &Shape, map: &mut MapOfShape) -> Result<bool, Failure> {
    if shape.is_null() {
        return Ok(false);
    }
    let mut homogeneous = true;
    let mut previous: Option<ShapeEnum> = None;
    for child in ShapeIterator::new(shape, true, true) {
        let kind = child.shape_type()?;
        if (kind as i32) > ShapeEnum::CompSolid as i32 {
            map.add(child);
            match previous {
                None => previous = Some(kind),
                Some(previous) if previous != kind => homogeneous = false,
                Some(_) => {}
            }
        } else if !fill_s_map(&child, map)? {
            homogeneous = false;
        }
    }
    Ok(homogeneous)
}

/// The file-static `Compare(NS, MDF, Stop, S)`: the current shape of `NS` in the scope,
/// outside the descendants of the stop shape, is `S` alone.
pub(crate) fn compare(
    named: &Handle<TNamingNamedShape>,
    scope: &TNamingScope,
    stop: &Handle<TNamingNamedShape>,
    shape: &Shape,
) -> Result<bool, Failure> {
    let mut forbidden = TdfLabelMap::default();
    if stop.is_some() {
        TNamingNamingTool::build_descendants(stop, &mut forbidden)?;
    }
    let mut shapes = IndexedMapOfShape::default();
    TNamingNamingTool::current_shape(scope.get_valid(), &forbidden, named, &mut shapes)?;
    Ok(shapes.contains(shape) && shapes.extent() == 1)
}

/// The file-static `TestSolution(MDF, NS, S)`: the current shape of `NS` in the scope is the
/// selection (the same faces for a solid, the same simple shapes for a collection).
pub(crate) fn test_solution(
    scope: &TNamingScope,
    named: &Handle<TNamingNamedShape>,
    shape: &Shape,
) -> Result<bool, Failure> {
    if named.is_null() {
        return Ok(false);
    }
    let result = scope.current_shape(named)?;
    if shape.is_null() || result.is_null() {
        return Ok(false);
    }
    let kind = shape.shape_type()?;
    let simple = matches!(kind, ShapeEnum::Face | ShapeEnum::Edge | ShapeEnum::Vertex);
    if simple && result.shape_type()? != ShapeEnum::Compound {
        return Ok(result.is_same(shape));
    }
    if matches!(kind, ShapeEnum::Solid | ShapeEnum::CompSolid) {
        return same_faces(shape, &result);
    }
    test_collection(shape, &result)
}

/// The solid branch of `TestSolution`: the same faces.
fn same_faces(shape: &Shape, result: &Shape) -> Result<bool, Failure> {
    let mut faces = MapOfShape::default();
    for face in explore(shape, ShapeEnum::Face)? {
        faces.add(face);
    }
    for face in explore(result, ShapeEnum::Face)? {
        if !faces.remove(&face) {
            return Ok(false);
        }
    }
    Ok(faces.is_empty())
}

/// The collection branch of `TestSolution`: the simple shapes of the selection are those of
/// the result (as sub-shapes of a solid or compound result, else as its children).
fn test_collection(shape: &Shape, result: &Shape) -> Result<bool, Failure> {
    let mut simple = MapOfShape::default();
    let homogeneous = fill_s_map(shape, &mut simple)?;
    let mut kind = ShapeEnum::Shape;
    let mut kinds = Map::<i32>::default();
    for item in simple.iter() {
        kind = item.shape_type()?;
        if homogeneous {
            break;
        }
        kinds.add(kind as i32);
    }
    if simple.remove(result) && simple.is_empty() {
        return Ok(true);
    }
    let result_kind = result.shape_type()?;
    if matches!(
        result_kind,
        ShapeEnum::Solid | ShapeEnum::CompSolid | ShapeEnum::Compound
    ) {
        let explored: Vec<ShapeEnum> = if homogeneous {
            vec![kind]
        } else {
            kinds
                .iter()
                .filter_map(|&k| ShapeEnum::ALL.get(k as usize).copied())
                .collect()
        };
        for sub_kind in explored {
            for sub_shape in explore(result, sub_kind)? {
                simple.remove(&sub_shape);
            }
        }
        return Ok(simple.is_empty());
    }
    if shape.is_same(result) {
        return Ok(true);
    }
    for child in ShapeIterator::new(result, true, true) {
        if !simple.remove(&child) {
            return Ok(false);
        }
    }
    Ok(simple.is_empty())
}

/// The sub-shapes of a type in exploration order.
pub(crate) fn explore(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut shapes = Vec::new();
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    while explorer.more() {
        shapes.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(shapes)
}

/// The file-static `FindNewShapeInFather(NS, SC)`: the first new shape of the father label,
/// when it has one.
pub(crate) fn find_new_shape_in_father(
    named: &Handle<TNamingNamedShape>,
    context: &mut Shape,
) -> Result<(), Failure> {
    let father = live(named, "FindNewShapeInFather")?.label().father();
    let iterator = TNamingIterator::from_label(&father)?;
    if iterator.more() {
        *context = iterator.new_shape()?;
    }
    Ok(())
}

/// The file-static `NextModif(NS)`: the named shape of the first modification of the first
/// new shape of `NS`.
pub(crate) fn next_modif(
    named: &Handle<TNamingNamedShape>,
) -> Result<Handle<TNamingNamedShape>, Failure> {
    if named.is_null() {
        return Ok(Handle::default());
    }
    let first = TNamingIterator::new(named)?;
    let iterator = TNamingNewShapeIterator::from_iterator(&first)?;
    if iterator.more() && iterator.is_modification()? {
        return iterator.named_shape();
    }
    Ok(Handle::default())
}

/// The file-static `IsContSame(C1, C2)`: the father's candidate is homogeneous and holds
/// every child of the rebuilt child's shape.
pub(crate) fn is_cont_same(candidate: &Shape, rebuilt: &Shape) -> Result<bool, Failure> {
    if candidate.is_null() || rebuilt.is_null() {
        return Ok(false);
    }
    let mut map = MapOfShape::default();
    if !fill_s_map(candidate, &mut map)? {
        return Ok(false);
    }
    Ok(ShapeIterator::new(rebuilt, true, true).all(|child| map.contains(&child)))
}

/// The file-static `IsMultipleCase(S, Context, Neighbourgs)`: the edge and its neighbours have
/// one face in common, and there are fewer faces than edges (a filter cannot single it out).
pub(crate) fn is_multiple_case(
    shape: &Shape,
    context: &Shape,
    neighbours: &MapOfShape,
) -> Result<bool, Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    let mut all_faces = Map::<Shape>::default();
    let mut edges = MapOfShape::default();
    edges.assign(neighbours);
    edges.add(shape.clone());
    let mut faces_of: DataMap<Shape, Map<Shape>> = DataMap::default();
    topexp::map_shapes_and_ancestors(context, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    for edge in edges.iter() {
        let Some(faces) = ancestors.seek(edge) else {
            return Ok(false);
        };
        let mut own = Map::<Shape>::default();
        for face in faces.iter() {
            all_faces.add(face.clone());
            own.add(face.clone());
        }
        if own.extent() > 0 {
            faces_of.bind(edge.clone(), own);
        }
    }
    let mut common = true;
    for face in all_faces.iter() {
        common = faces_of.values().all(|faces| faces.contains(face));
        if common {
            // A single common face.
            break;
        }
    }
    Ok(common && all_faces.extent() < edges.extent())
}
