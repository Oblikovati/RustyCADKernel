// Rust port of the GENERATION algorithm of ApplicationFramework/TKCAF/TNaming/TNaming_Name.cxx
// (Generated, FindShape, SearchModifUntil, FindModifUntil)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GENERATION name: the shapes a generation made from the current generators.
use super::name_solve::{NameArgs, argument, valid_args};
use super::package::compound_of;
use super::{
    TNamingBuilder, TNamingIterator, TNamingNamedShape, TNamingNaming, TNamingNewShapeIterator,
};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::{DataMap, Map};
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfLabelMap};

/// `NCollection_DataMap<TopoDS_Shape, NCollection_Map<TopoDS_Shape>>` (default hashers, so
/// orientation counts): each generator to the shapes generated from it.
type GeneratedMap = DataMap<Shape, Map<Shape>>;

/// GENERATION: the first argument is the generation, the others its generators. The
/// generators' current shapes give the generated shapes; the first solve selects the one of
/// the name's shape (and records its index), a regeneration the shape common to every
/// generator, or all candidates of the name's type.
pub(crate) fn generated(
    label: &TdfLabel,
    _valid: &TdfLabelMap,
    args: &NameArgs,
) -> Result<bool, Failure> {
    if args.len() < 2 {
        return Err(exceptions::construction_error(&format!(
            "TNaming_Name::Solve: => Generated with {} arguments; expected at least 2",
            args.len()
        )));
    }
    if !valid_args(args) {
        return Ok(false);
    }
    let generation = argument(args, 0)?;
    let generation_label = live(generation, "TNaming_Name::Solve")?.label();
    let generators = &args[1..];
    let mut current_generators = IndexedMapOfShape::default();
    search_modif_until(generation, generators, &mut current_generators)?;
    let mut naming: Handle<TNamingNaming> = Handle::default();
    label.find_attribute_as(&TNamingNaming::get_id(), &mut naming)?;
    let selection = naming
        .get()
        .map(|n| n.get_name().shape())
        .unwrap_or_default();
    let mut old: Handle<TNamingNamedShape> = Handle::default();
    label.find_attribute_as(&TNamingNamedShape::get_id(), &mut old)?;
    let version = old.get().map_or(-1, TNamingNamedShape::version);
    let builder = TNamingBuilder::new(label)?;
    let (list, by_generator) = generated_shapes(label, &generation_label, &current_generators)?;
    if version == -1 {
        return initial_selection(&builder, &naming, &selection, &list);
    }
    if list.len() == 1 {
        // The trivial case.
        builder.select(&list[0], &list[0])?;
        return Ok(true);
    }
    let regeneration = Regeneration {
        builder: &builder,
        naming: &naming,
        generators,
        current_generators: &current_generators,
    };
    regeneration.select(&list, &by_generator)
}

/// The shapes made by the generation from each current generator, in order, and per
/// generator (szy 21.10.03: all of them).
fn generated_shapes(
    label: &TdfLabel,
    generation_label: &TdfLabel,
    generators: &IndexedMapOfShape,
) -> Result<(Vec<Shape>, GeneratedMap), Failure> {
    let mut list = Vec::new();
    let mut by_generator = GeneratedMap::default();
    for index in 1..=generators.extent() {
        let generator = generators.find_key(index)?;
        let mut made = Map::<Shape>::default();
        let mut iterator = TNamingNewShapeIterator::from_label(generator, label)?;
        while iterator.more() {
            if iterator.label()? == *generation_label {
                made.add(iterator.shape()?);
                list.push(iterator.shape()?);
            }
            iterator.next()?;
        }
        if made.extent() > 0 {
            by_generator.bind(generator.clone(), made);
        }
    }
    Ok((list, by_generator))
}

/// The initial build of the name: the generated shape that is the selection (its 1-based
/// position recorded in the name for debug), or every generated shape without a selection.
/// Natively a label without naming crashes there; the port reports `Standard_NullObject`.
fn initial_selection(
    builder: &TNamingBuilder,
    naming: &Handle<TNamingNaming>,
    selection: &Shape,
    list: &[Shape],
) -> Result<bool, Failure> {
    let naming = live(naming, "TNaming_Name::Solve: => Generated")?;
    if selection.is_null() {
        for shape in list {
            builder.select(shape, shape)?;
        }
        return Ok(true);
    }
    if let Some(position) = list.iter().position(|shape| shape.is_same(selection)) {
        builder.select(&list[position], &list[position])?;
        naming
            .change_name()
            .set_index(i32::try_from(position + 1).unwrap_or(i32::MAX));
    }
    Ok(true)
}

/// The inputs of a regeneration.
struct Regeneration<'a> {
    builder: &'a TNamingBuilder,
    naming: &'a Handle<TNamingNaming>,
    generators: &'a NameArgs,
    current_generators: &'a IndexedMapOfShape,
}

impl Regeneration<'_> {
    /// The regeneration branch: the candidates of the name's type; the shape common to all
    /// generators when there is one and the generators are all kept (or one lost generator
    /// was a neighbour), else every candidate.
    fn select(&self, list: &[Shape], by_generator: &GeneratedMap) -> Result<bool, Failure> {
        let naming = live(self.naming, "TNaming_Name::Solve: => Generated")?;
        let shape_type = naming.get_name().shape_type();
        let mut candidates = Vec::new();
        for shape in list {
            if shape.shape_type()? == shape_type {
                candidates.push(shape.clone());
            }
        }
        if candidates.is_empty() {
            return Ok(false);
        }
        let common = find_shape(by_generator)?;
        if !common.is_null() && self.keeps_common()? {
            self.builder.select(&common, &common)?;
            return Ok(true);
        }
        for shape in &candidates {
            self.builder.select(shape, shape)?;
        }
        Ok(true)
    }
    /// All generators kept, or one lost generator sharing a vertex (edges) or an edge
    /// (faces) with a kept one.
    fn keeps_common(&self) -> Result<bool, Failure> {
        if self.generators.len() == self.current_generators.extent() {
            return Ok(true);
        }
        let mut lost = Vec::new();
        for generator in self.generators {
            let shape = live(generator, "TNaming_Name::Solve: => Generated")?.get()?;
            if !self.current_generators.contains(&shape) {
                lost.push(shape);
            }
        }
        if lost.len() != 1 {
            return Ok(false);
        }
        lost_neighbour(&lost[0], self.current_generators)
    }
}

/// The lost-generator test: a kept generator of the same type shares a vertex (edges) or an
/// edge (faces) with the lost one.
fn lost_neighbour(lost: &Shape, kept: &IndexedMapOfShape) -> Result<bool, Failure> {
    for index in 1..=kept.extent() {
        let shape = kept.find_key(index)?;
        if shape.shape_type()? != lost.shape_type()? {
            continue;
        }
        match shape.shape_type()? {
            ShapeEnum::Edge => {
                if topexp::common_vertex(&topods::edge(shape)?, &topods::edge(lost)?).is_some() {
                    return Ok(true);
                }
            }
            ShapeEnum::Face if shares_edge(shape, lost)? => return Ok(true),
            _ => {}
        }
    }
    Ok(false)
}

/// Two faces share an edge.
fn shares_edge(first: &Shape, second: &Shape) -> Result<bool, Failure> {
    let mut outer = Explorer::new(first, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while outer.more() {
        let mut inner = Explorer::new(second, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while inner.more() {
            if outer.current().is_same(inner.current()) {
                return Ok(true);
            }
            inner.next()?;
        }
        outer.next()?;
    }
    Ok(false)
}

/// The file-static `FindShape(DM)`: the shapes generated from the first generator (in map
/// order) that every other generator generated too; null, the one, or a compound.
fn find_shape(by_generator: &GeneratedMap) -> Result<Shape, Failure> {
    let Some((first_key, first_map)) = by_generator.items().next() else {
        return Ok(Shape::default());
    };
    let mut common = Vec::new();
    for shape in first_map.iter() {
        let in_all = by_generator
            .items()
            .filter(|(key, _)| *key != first_key)
            .all(|(_, map)| map.contains(shape));
        if in_all {
            common.push(shape.clone());
        }
    }
    match common.len() {
        0 => Ok(Shape::default()),
        1 => Ok(common[0].clone()),
        _ => compound_of(common.iter()),
    }
}

/// The file-static `SearchModifUntil(Valid, Target, theListOfGenerators, theMS)`: the new
/// shapes of each generator that are old shapes of the target, else their descendants the
/// target made.
fn search_modif_until(
    target: &Handle<TNamingNamedShape>,
    generators: &NameArgs,
    shapes: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    for generator in generators {
        let mut iterator = TNamingIterator::new(generator)?;
        while iterator.more() {
            let shape = iterator.new_shape()?;
            if !is_old_shape_of(target, &shape)? {
                let mut descendants = TNamingNewShapeIterator::from_iterator(&iterator)?;
                find_modif_until(&mut descendants, shapes, &shape, target)?;
            } else {
                shapes.add(shape);
            }
            iterator.next()?;
        }
    }
    Ok(())
}

/// The shape is an old shape of the target named shape.
fn is_old_shape_of(target: &Handle<TNamingNamedShape>, shape: &Shape) -> Result<bool, Failure> {
    let mut iterator = TNamingIterator::new(target)?;
    while iterator.more() {
        if iterator.old_shape()?.is_same(shape) {
            return Ok(true);
        }
        iterator.next()?;
    }
    Ok(false)
}

/// The file-static `FindModifUntil(it, MS, S, Context)` of TNaming_Name.cxx: `S` when the
/// context made a new shape of it, else the same search on its descendants.
fn find_modif_until(
    iterator: &mut TNamingNewShapeIterator,
    shapes: &mut IndexedMapOfShape,
    shape: &Shape,
    context: &Handle<TNamingNamedShape>,
) -> Result<bool, Failure> {
    let mut found = false;
    while iterator.more() {
        let new_shape = iterator.shape()?;
        if !new_shape.is_null() {
            if iterator.named_shape()? == *context {
                shapes.add(shape.clone());
                found = true;
            } else {
                let mut next = TNamingNewShapeIterator::from_new_shape_iterator(iterator)?;
                found = find_modif_until(&mut next, shapes, &new_shape, context)?;
            }
        }
        iterator.next()?;
    }
    Ok(found)
}
