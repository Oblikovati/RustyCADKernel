// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming.cxx (Displace, Transform,
// Replicate, ChangeShapes and their file-static helpers)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TNaming` functions that rebuild the named shapes of a label tree with moved,
//! transformed or copied shapes.
use super::package::{children, compound_of, copy_from_images};
use super::{
    TNaming, TNamingBuilder, TNamingEvolution, TNamingIterator, TNamingNamedShape,
    TNamingShapeShapeMap, TNamingTool,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::gp::Transform3;
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeShape, BRepBuilderAPITransform};

impl TNaming {
    /// `Displace(label, aLocation, WithOld)`: every named shape of the label tree rebuilt with
    /// its new (and, with `WithOld`, old) shapes moved.
    pub fn displace(label: &TdfLabel, location: &Location, with_old: bool) -> Result<(), Failure> {
        let mut iterator = TNamingIterator::from_label(label)?;
        // dp: the children are visited even without a shape.
        if iterator.more() {
            let evolution = iterator.evolution()?;
            let mut pairs = Vec::new();
            while iterator.more() {
                pairs.push((iterator.old_shape()?, iterator.new_shape()?));
                iterator.next()?;
            }
            let builder = TNamingBuilder::new(label)?;
            for (old, new) in &pairs {
                let old = if with_old && !old.is_null() {
                    old.moved(location, false)?
                } else {
                    old.clone()
                };
                let new = if new.is_null() {
                    Shape::default()
                } else {
                    new.moved(location, false)?
                };
                load_named_shape(&builder, evolution, &old, &new)?;
            }
        }
        for child in children(label)? {
            Self::displace(&child, location, with_old)?;
        }
        Ok(())
    }
    /// `Transform(label, aTransformation)`: the shapes of the label tree transformed together
    /// (one compound), then the named shapes rebuilt with their images.
    pub fn transform(label: &TdfLabel, transformation: &Transform3) -> Result<(), Failure> {
        let mut shapes = Vec::new();
        collect_shapes(label, &mut shapes)?;
        let compound = compound_of(shapes.iter())?;
        let transformer =
            BRepBuilderAPITransform::with_shape(&compound, *transformation, false, false)?;
        let mut map = TNamingShapeShapeMap::default();
        build_transform_map(label, &transformer, &mut map)?;
        replace(label, &map)
    }
    /// `Replicate(NS, T, L)`: replicates the current shape of `NS`.
    pub fn replicate(
        named: &Handle<TNamingNamedShape>,
        transformation: &Transform3,
        label: &TdfLabel,
    ) -> Result<(), Failure> {
        let shape = TNamingTool::current_shape(named)?;
        Self::replicate_shape(&shape, transformation, label)
    }
    /// `Replicate(SH, T, L)`: the transformed shape generated from `SH` on `L`, and its faces
    /// (edges for a face or a wire, whose geometry is copied for the prism pattern)
    /// generated from those of `SH` on the first child of `L`.
    pub fn replicate_shape(
        shape: &Shape,
        transformation: &Transform3,
        label: &TdfLabel,
    ) -> Result<(), Failure> {
        let shape_type = shape.shape_type()?;
        let thin = matches!(shape_type, ShapeEnum::Face | ShapeEnum::Wire);
        let mut transformer = BRepBuilderAPITransform::new(*transformation);
        transformer.perform(shape, thin, false)?;
        let new_shape = transformer.shape()?.clone();
        TNamingBuilder::new(label)?.generated_from(shape, &new_shape)?;
        let sub_type = if thin {
            ShapeEnum::Edge
        } else {
            ShapeEnum::Face
        };
        let builder = TNamingBuilder::new(&label.find_child(1, true)?)?;
        let mut explorer = Explorer::new(shape, sub_type, ShapeEnum::Shape)?;
        while explorer.more() {
            let old_sub_shape = explorer.current().clone();
            builder.generated_from(&old_sub_shape, &transformer.modified_shape(&old_sub_shape)?)?;
            explorer.next()?;
        }
        Ok(())
    }
    /// `ChangeShapes(label, M)`: every named shape of the label tree rebuilt with copies of its
    /// shapes (new TShapes with the same flags, recorded in `M`).
    pub fn change_shapes(label: &TdfLabel, map: &mut TNamingShapeShapeMap) -> Result<(), Failure> {
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
        if let Some(named) = named.get() {
            let evolution = named.evolution();
            let mut pairs = Vec::new();
            let mut iterator = TNamingIterator::from_label(label)?;
            while iterator.more() {
                let old = shape_copy(&iterator.old_shape()?, map)?;
                let new = shape_copy(&iterator.new_shape()?, map)?;
                pairs.push((old, new));
                iterator.next()?;
            }
            let builder = TNamingBuilder::new(label)?;
            for (old, new) in &pairs {
                load_named_shape(&builder, evolution, old, new)?;
            }
        }
        for child in children(label)? {
            Self::change_shapes(&child, map)?;
        }
        Ok(())
    }
}

/// The file-static `LoadNamedShape` of TNaming.cxx (`REPLACE` loads nothing).
fn load_named_shape(
    builder: &TNamingBuilder,
    evolution: TNamingEvolution,
    old: &Shape,
    new: &Shape,
) -> Result<(), Failure> {
    match evolution {
        TNamingEvolution::Primitive => builder.generated(new),
        TNamingEvolution::Generated => builder.generated_from(old, new),
        TNamingEvolution::Modify => builder.modify(old, new),
        TNamingEvolution::Delete => builder.delete(old),
        TNamingEvolution::Selected => builder.select(new, old),
        TNamingEvolution::Replace => Ok(()),
    }
}

/// The file-static `BuildCompound`: the old and new shapes of the label tree, depth first.
fn collect_shapes(label: &TdfLabel, shapes: &mut Vec<Shape>) -> Result<(), Failure> {
    let mut iterator = TNamingIterator::from_label(label)?;
    while iterator.more() {
        for shape in [iterator.old_shape()?, iterator.new_shape()?] {
            if !shape.is_null() {
                shapes.push(shape);
            }
        }
        iterator.next()?;
    }
    for child in children(label)? {
        collect_shapes(&child, shapes)?;
    }
    Ok(())
}

/// The file-static `BuildMap(L, Transformer, M)`: each shape of the label tree mapped to its
/// image.
fn build_transform_map(
    label: &TdfLabel,
    transformer: &BRepBuilderAPITransform,
    map: &mut TNamingShapeShapeMap,
) -> Result<(), Failure> {
    let mut iterator = TNamingIterator::from_label(label)?;
    while iterator.more() {
        for shape in [iterator.old_shape()?, iterator.new_shape()?] {
            if !shape.is_null() {
                map.bind(shape.clone(), transformer.modified_shape(&shape)?);
            }
        }
        iterator.next()?;
    }
    for child in children(label)? {
        build_transform_map(&child, transformer, map)?;
    }
    Ok(())
}

/// The file-static `Replace(L, M)`: each named shape rebuilt with the images of its shapes.
/// The native old and new shapes live outside the loop, so a null shape reuses the previous
/// evolution's.
fn replace(label: &TdfLabel, map: &TNamingShapeShapeMap) -> Result<(), Failure> {
    let mut iterator = TNamingIterator::from_label(label)?;
    if !iterator.more() {
        return Ok(());
    }
    let evolution = iterator.evolution()?;
    let builder = TNamingBuilder::new(label)?;
    let (mut old, mut new) = (Shape::default(), Shape::default());
    while iterator.more() {
        let (old_shape, new_shape) = (iterator.old_shape()?, iterator.new_shape()?);
        if !old_shape.is_null() {
            old = map.seek(&old_shape).cloned().unwrap_or(old_shape);
        }
        if !new_shape.is_null() {
            new = map.seek(&new_shape).cloned().unwrap_or(new_shape);
        }
        load_named_shape(&builder, evolution, &old, &new)?;
        iterator.next()?;
    }
    for child in children(label)? {
        replace(&child, map)?;
    }
    Ok(())
}

/// The file-static `ShapeCopy(S, M)`: a copy of the shape with copies of its sub-shapes and
/// the flags of the source TShape.
fn shape_copy(shape: &Shape, map: &mut TNamingShapeShapeMap) -> Result<Shape, Failure> {
    if shape.is_null() {
        return Ok(shape.clone());
    }
    if let Some(copy) = map.seek(shape) {
        return Ok(copy.clone());
    }
    let forward = shape.oriented(Orientation::Forward);
    for sub_shape in rustycad_tkbrep::topods::ShapeIterator::new(&forward, true, true) {
        shape_copy(&sub_shape, map)?;
    }
    let mut copy = copy_from_images(shape, map)?;
    copy.set_orientation(shape.orientation());
    copy_flags(shape, &copy)?;
    map.bind(shape.clone(), copy.clone());
    Ok(copy)
}

/// The flags `ShapeCopy` copies, in the native order.
fn copy_flags(source: &Shape, copy: &Shape) -> Result<(), Failure> {
    copy.set_free(source.free()?)?;
    copy.set_modified(source.modified()?)?;
    copy.set_checked(source.checked()?)?;
    copy.set_orientable(source.orientable()?)?;
    copy.set_closed(source.closed()?)?;
    copy.set_infinite(source.infinite()?)?;
    copy.set_convex(source.convex()?)
}
