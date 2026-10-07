// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming.hxx and TNaming.cxx (Substitute,
// Update, SubstituteSShape, MakeShape, IDList, Print) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming`: the package functions that rewrite the shapes of named-shape trees (the
//! editing functions are in `package_edit.rs`, the context queries in `package_context.rs`).
use super::TNamingRefShape;
use super::named_shape::used_shapes_of;
use super::{
    TNamingEvolution, TNamingIterator, TNamingNameType, TNamingNamedShape, TNamingShapeRefMap,
    TNamingShapeShapeMap, TNamingUsedShapes,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::{Failure, Guid};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttribute, TdfChildIterator, TdfLabel};
use std::io;
use std::sync::Arc;

/// `TNaming`: package functions.
///
/// ```
/// use rustycad_tkcaf::tnaming::{TNaming, TNamingEvolution};
/// let mut text = Vec::new();
/// TNaming::print_evolution(TNamingEvolution::Modify, &mut text).unwrap();
/// assert_eq!(text, b"MODIFY");
/// ```
pub struct TNaming;

impl TNaming {
    /// `Substitute(labelsource, labelcible, mapOldNew)`: maps the shapes of the target tree
    /// to those of the parallel source tree, then rebinds each target shape of the
    /// `TNaming_UsedShapes` map to its source shape.
    pub fn substitute(
        source: &TdfLabel,
        target: &TdfLabel,
        map: &mut TNamingShapeShapeMap,
    ) -> Result<(), Failure> {
        map_labels(target, source, map)?;
        let used = used_shapes_of(target)?;
        let mut used_map = live(&used, "TNaming::Substitute")?.map();
        let pairs: Vec<(Shape, Shape)> = map.items().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (old, new) in &pairs {
            substitute_shape(old, new, &mut used_map);
        }
        Ok(())
    }
    /// `Update(label, mapOldNew)`: rebuilds the shapes of the label tree that contain
    /// substituted sub-shapes, and rebinds them in the `TNaming_UsedShapes` map.
    pub fn update(label: &TdfLabel, map: &mut TNamingShapeShapeMap) -> Result<(), Failure> {
        let used = used_shapes_of(label)?;
        let used = live(&used, "TNaming::Update")?;
        let mut iterator = TNamingIterator::from_label(label)?;
        while let Some(node) = iterator.node.clone() {
            for reference in [node.old(), node.new_shape()] {
                if let Some(reference) = reference.filter(|r| !r.shape().is_null()) {
                    update_shape(&reference, map, used)?;
                }
            }
            iterator.next()?;
        }
        for child in children(label)? {
            Self::update(&child, map)?;
        }
        Ok(())
    }
    /// `MakeShape(MS)`: nothing, the only shape, or a compound of the shapes in map order.
    pub fn make_shape(shapes: &MapOfShape) -> Result<Shape, Failure> {
        match shapes.extent() {
            0 => Ok(Shape::default()),
            1 => Ok(shapes.iter().next().cloned().unwrap_or_default()),
            _ => compound_of(shapes.iter()),
        }
    }
    /// `SubstituteSShape(accesslabel, From, To)`: the ref shape of `To` takes the orientation
    /// of `From` (`To` is changed); false when `To` is not in the map.
    pub fn substitute_s_shape(
        access: &TdfLabel,
        from: &Shape,
        to: &mut Shape,
    ) -> Result<bool, Failure> {
        let used = used_shapes_of(access)?;
        let mut map = live(&used, "TNaming::SubstituteSShape")?.map();
        let Some(reference) = map.seek(to).cloned() else {
            return Ok(false);
        };
        if !map.unbind(to) {
            return Ok(false);
        }
        to.set_orientation(from.orientation());
        reference.set_shape(to);
        Ok(map.bind(to.clone(), reference))
    }
    /// `IDList(anIDList)`: appends the GUID of `TNaming_NamedShape`.
    pub fn id_list(ids: &mut Vec<Guid>) {
        ids.push(TNamingNamedShape::get_id());
    }
    /// `Print(EVOL, S)`: the name of an evolution (`UNKNOWN_Evolution` for `REPLACE`).
    pub fn print_evolution(
        evolution: TNamingEvolution,
        stream: &mut dyn io::Write,
    ) -> io::Result<()> {
        let text = match evolution {
            TNamingEvolution::Primitive => "PRIMITIVE",
            TNamingEvolution::Generated => "GENERATED",
            TNamingEvolution::Modify => "MODIFY",
            TNamingEvolution::Delete => "DELETE",
            TNamingEvolution::Selected => "SELECTED",
            TNamingEvolution::Replace => "UNKNOWN_Evolution",
        };
        write!(stream, "{text}")
    }
    /// `Print(NAME, S)`: the name of a name type.
    pub fn print_name_type(
        name_type: TNamingNameType,
        stream: &mut dyn io::Write,
    ) -> io::Result<()> {
        write!(stream, "{}", name_type_text(name_type))
    }
    /// `Print(ACCESS, S)`: the dump of the root's `TNaming_UsedShapes` (nothing without one).
    pub fn print_label(access: &TdfLabel, stream: &mut dyn io::Write) -> io::Result<()> {
        let used = used_shapes_of(access).map_err(io::Error::other)?;
        match used.get() {
            Some(used) => used.dump(stream),
            None => Ok(()),
        }
    }
}

/// The text of `Print(TNaming_NameType)`.
fn name_type_text(name_type: TNamingNameType) -> &'static str {
    match name_type {
        TNamingNameType::Unknown => "UNKNOWN",
        TNamingNameType::Identity => "IDENTITY",
        TNamingNameType::ModifUntil => "MODIFUNTIL",
        TNamingNameType::Generation => "GENERATION",
        TNamingNameType::Intersection => "INTERSECTION",
        TNamingNameType::Union => "UNION",
        TNamingNameType::Substraction => "SUBSTRACTION",
        TNamingNameType::ConstShape => "CONSTSHAPE",
        TNamingNameType::FilterByNeighbourgs => "FILTERBYNEIGHBOURGS",
        TNamingNameType::Orientation => "ORIENTATION",
        TNamingNameType::WireIn => "WIREIN",
        TNamingNameType::ShellIn => "SHELLIN",
    }
}

/// A compound of shapes, in order.
pub(crate) fn compound_of<'a>(shapes: impl Iterator<Item = &'a Shape>) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut compound = compound.into_shape();
    for shape in shapes {
        builder.add(&mut compound, shape)?;
    }
    Ok(compound)
}

/// The children of a label (`TDF_ChildIterator(L)`).
pub(crate) fn children(label: &TdfLabel) -> Result<Vec<TdfLabel>, Failure> {
    let mut labels = Vec::new();
    let mut iterator = TdfChildIterator::new(label, false)?;
    while iterator.more() {
        labels.push(iterator.value());
        iterator.next();
    }
    Ok(labels)
}

/// The file-static `MapShapes(SCible, SSource, M)`: the target shape and, in parallel, its
/// sub-shapes not mapped yet.
fn map_shapes(
    target: &Shape,
    source: &Shape,
    map: &mut TNamingShapeShapeMap,
) -> Result<(), Failure> {
    map.bind(target.clone(), source.clone());
    let mut targets = ShapeIterator::new(target, true, true);
    let mut sources = ShapeIterator::new(source, true, true);
    while targets.more() {
        if !map.is_bound(targets.value()?) {
            let (next_target, next_source) = (targets.value()?.clone(), sources.value()?.clone());
            map_shapes(&next_target, &next_source, map)?;
        }
        targets.next();
        sources.next();
    }
    Ok(())
}

/// The file-static `MapShapes(LCible, LSource, M)`: the shapes of the parallel label trees.
fn map_labels(
    target: &TdfLabel,
    source: &TdfLabel,
    map: &mut TNamingShapeShapeMap,
) -> Result<(), Failure> {
    let mut targets = TNamingIterator::from_label(target)?;
    let mut sources = TNamingIterator::from_label(source)?;
    while targets.more() {
        let old = targets.old_shape()?;
        if !old.is_null() && !map.is_bound(&old) {
            map_shapes(&old, &sources.old_shape()?, map)?;
        }
        let new = targets.new_shape()?;
        if !new.is_null() && !map.is_bound(&new) {
            map_shapes(&new, &sources.new_shape()?, map)?;
        }
        targets.next()?;
        sources.next()?;
    }
    let mut target_children = TdfChildIterator::new(target, false)?;
    let mut source_children = TdfChildIterator::new(source, false)?;
    while target_children.more() {
        map_labels(&target_children.value(), &source_children.value(), map)?;
        target_children.next();
        source_children.next();
    }
    Ok(())
}

/// The file-static `SubstituteShape(oldShape, newShape, amap)`: the ref shape of the old
/// shape takes the new one and is rebound under it. A same old and new shape is reported on
/// the standard output, as natively.
fn substitute_shape(old: &Shape, new: &Shape, map: &mut TNamingShapeRefMap) {
    if old.is_same(new) {
        println!("import_tool::Substitute : oldShape IsSame newShape");
    }
    let Some(reference) = map.seek(old).cloned() else {
        return;
    };
    reference.set_shape(new);
    map.unbind(old);
    map.bind(new.clone(), reference);
}

/// One shape of `Update`: rebuilt when it is not mapped yet, then substituted.
///
/// Natively the old shape `SubstituteShape` receives is a reference to the shape of the
/// iterator's ref shape: when that ref shape is the one bound to it, `pos->Shape(newShape)`
/// changes the reference too, so `UnBind(oldShape)` unbinds the new shape and the old key
/// stays bound. The port reads the key back from the ref shape to keep that.
fn update_shape(
    reference: &Arc<TNamingRefShape>,
    map: &mut TNamingShapeShapeMap,
    used: &TNamingUsedShapes,
) -> Result<(), Failure> {
    let shape = reference.shape();
    if !map.is_bound(&shape) {
        rebuild(&shape, map)?;
    }
    let image = map.find(&shape)?.clone();
    let mut used_map = used.map();
    if shape.is_same(&image) {
        println!("import_tool::Substitute : oldShape IsSame newShape");
    }
    let Some(bound) = used_map.seek(&shape).cloned() else {
        return Ok(());
    };
    bound.set_shape(&image);
    used_map.unbind(&reference.shape());
    used_map.bind(image, bound);
    Ok(())
}

/// The file-static `Rebuild(S, M)`: a shape whose sub-shapes changed is rebuilt from their
/// images. The native result is inverted: it is true when the shape is mapped to itself
/// (unchanged), so the parent of an unchanged shape is rebuilt too.
fn rebuild(shape: &Shape, map: &mut TNamingShapeShapeMap) -> Result<bool, Failure> {
    if map.is_bound(shape) {
        return Ok(false);
    }
    let forward = shape.oriented(Orientation::Forward);
    let mut modified = false;
    for sub_shape in ShapeIterator::new(&forward, true, true) {
        if rebuild(&sub_shape, map)? {
            modified = true;
        }
    }
    if !modified {
        map.bind(shape.clone(), shape.clone());
        return Ok(true);
    }
    let rebuilt = copy_from_images(shape, map)?;
    map.bind(shape.clone(), rebuilt.oriented(shape.orientation()));
    Ok(modified)
}

/// The rebuild of `Rebuild` and `ShapeCopy`: an empty copy of the forward shape (with the
/// range of an edge) holding the images of its sub-shapes in their orientations.
pub(crate) fn copy_from_images(
    shape: &Shape,
    map: &TNamingShapeShapeMap,
) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let forward = shape.oriented(Orientation::Forward);
    let mut copy = forward.clone();
    copy.empty_copy()?;
    if copy.shape_type()? == ShapeEnum::Edge {
        let (first, last) = BRepTool::range(&topods::edge(shape)?)?;
        builder.range(&topods::edge(&copy)?, first, last, false)?;
    }
    for sub_shape in ShapeIterator::new(&forward, true, true) {
        let image = map.find(&sub_shape)?;
        builder.add(&mut copy, &image.oriented(sub_shape.orientation()))?;
    }
    Ok(copy)
}
