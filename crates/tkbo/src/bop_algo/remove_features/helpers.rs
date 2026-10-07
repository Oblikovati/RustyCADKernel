//! The static helpers of BOPAlgo_RemoveFeatures.cxx on histories, internal shapes and the
//! images of a shape in a Boolean builder.
use crate::bop_algo::BuilderShape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The sub-shapes of a type in exploration order (`TopExp_Explorer`).
pub(super) fn explore(shape: &Shape, kind: ShapeEnum) -> Result<Vec<Shape>, Failure> {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape)?;
    let mut found = Vec::new();
    while explorer.more() {
        found.push(explorer.current().clone());
        explorer.next()?;
    }
    Ok(found)
}

/// A compound of the shapes.
pub(super) fn compound_of(shapes: &[Shape]) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut result = compound.into_shape();
    for shape in shapes {
        builder.add(&mut result, shape)?;
    }
    Ok(result)
}

/// The direct children of a shape (`TopoDS_Iterator`).
pub(super) fn children(shape: &Shape) -> Vec<Shape> {
    ShapeIterator::new(shape, true, true).collect()
}

/// Every supported sub-shape of the shapes, except those of `keep`, removed in the history
/// (`MakeRemoved`).
pub(super) fn make_removed(
    shapes: &[Shape],
    history: &History,
    keep: &IndexedMapOfShape,
) -> Result<(), Failure> {
    let mut sub_shapes = IndexedMapOfShape::default();
    for shape in shapes {
        topexp::map_shapes(shape, &mut sub_shapes, true, true);
    }
    for shape in sub_shapes.iter() {
        if !keep.contains(shape) && History::is_supported_type(shape)? {
            history.remove(shape)?;
        }
    }
    Ok(())
}

/// The internal children of a face or solid, and the children holding an internal
/// sub-shape (`FindInternals`).
pub(super) fn find_internals(shape: &Shape, internals: &mut Vec<Shape>) {
    for child in children(shape) {
        if child.orientation() == Orientation::Internal {
            internals.push(child);
            continue;
        }
        if children(&child)
            .iter()
            .any(|grandchild| grandchild.orientation() == Orientation::Internal)
        {
            internals.push(child);
        }
    }
}

/// Removes the internal wires from the faces of the shapes, in place; the removed wires
/// go to `removed` when given (`RemoveInternalWires`).
pub(super) fn remove_internal_wires(
    shapes: &[Shape],
    mut removed: Option<&mut Vec<Shape>>,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for shape in shapes {
        for mut face in explore(shape, ShapeEnum::Face)? {
            let mut internals = Vec::new();
            find_internals(&face, &mut internals);
            if internals.is_empty() {
                continue;
            }
            // The face shares its TShape with the result, which is edited in place.
            face.set_free(true)?;
            for internal in &internals {
                if let Some(removed) = removed.as_deref_mut() {
                    removed.push(internal.clone());
                }
                builder.remove(&mut face, internal)?;
            }
            face.set_free(false)?;
        }
    }
    Ok(())
}

/// The sub-shape of `place` that is the same as `shape`; null when there is none
/// (`FindShape`).
pub(super) fn find_shape(shape: &Shape, place: &Shape) -> Result<Shape, Failure> {
    for candidate in explore(place, shape.shape_type()?)? {
        if candidate.is_same(shape) {
            return Ok(candidate);
        }
    }
    Ok(Shape::default())
}

/// The splits of `shape` in the builder result, or the shape itself when it was kept
/// unchanged (`TakeModified`).
pub(super) fn take_modified(shape: &Shape, builder: &BuilderShape) -> Result<Vec<Shape>, Failure> {
    let modified = builder.modified(shape)?;
    if modified.is_empty() && !builder.is_deleted(shape)? {
        return Ok(vec![shape.clone()]);
    }
    Ok(modified)
}

/// `TakeModified` into a map.
pub(super) fn take_modified_into(
    shape: &Shape,
    builder: &BuilderShape,
    map: &mut MapOfShape,
) -> Result<(), Failure> {
    for image in take_modified(shape, builder)? {
        map.add(image);
    }
    Ok(())
}
