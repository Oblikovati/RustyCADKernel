// Rust port of the file-static helpers of ApplicationFramework/TKCAF/TNaming/TNaming_Tool.cxx
// (LastModif, MakeShape, ApplyOrientation, FindModifUntil, Back) and the orientation
// lookup the OR-N branches of GetShape and CurrentShape repeat,
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shape-collecting helpers of `TNaming_Tool`.
use super::{TNamingNameType, TNamingNamedShape, TNamingNaming, TNamingNewShapeIterator};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::Map;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tklcaf::tdf::{TdfChildIterator, TdfLabel, TdfLabelMap};

/// The file-static `MakeShape(MS)`: nothing, the only shape, or a compound of the shapes in
/// map order.
pub(crate) fn make_shape(shapes: &IndexedMapOfShape) -> Result<Shape, Failure> {
    match shapes.extent() {
        0 => Ok(Shape::default()),
        1 => Ok(shapes.find_key(1)?.clone()),
        _ => {
            let builder = BRepBuilder::default();
            let mut compound = Compound::default();
            builder.make_compound(&mut compound);
            let mut compound = compound.into_shape();
            for index in 1..=shapes.extent() {
                builder.add(&mut compound, shapes.find_key(index)?)?;
            }
            Ok(compound)
        }
    }
}

/// The file-static `ApplyOrientation`: every shape of the map oriented.
pub(crate) fn apply_orientation(
    shapes: &mut IndexedMapOfShape,
    orientation: Orientation,
) -> Result<(), Failure> {
    for index in 1..=shapes.extent() {
        let oriented = shapes.find_key(index)?.oriented(orientation);
        shapes.substitute(index, oriented)?;
    }
    Ok(())
}

/// The naming of a selection label for the OR-N code of `GetShape` and `CurrentShape`.
pub(crate) struct SelectionNaming {
    /// The name of the label's naming.
    pub(crate) name: super::TNamingName,
    /// The orientation of the first child naming of type `ORIENTATION`, searched only when
    /// the name itself is of another type.
    pub(crate) child_orientation: Option<Orientation>,
}

impl SelectionNaming {
    /// The naming of `label`, `None` without one.
    pub(crate) fn find(label: &TdfLabel) -> Result<Option<Self>, Failure> {
        let mut naming: Handle<TNamingNaming> = Handle::default();
        label.find_attribute_as(&TNamingNaming::get_id(), &mut naming)?;
        let Some(naming) = naming.get() else {
            return Ok(None);
        };
        let name = naming.get_name();
        let child_orientation = if name.name_type() == TNamingNameType::Orientation {
            None
        } else {
            child_orientation(&rustycad_tklcaf::tdf::TdfAttribute::attribute_base(naming).label())?
        };
        Ok(Some(Self {
            name,
            child_orientation,
        }))
    }
    /// The orientation of an `ORIENTATION` name, else of its orientation child.
    pub(crate) fn orientation(&self) -> Option<Orientation> {
        if self.name.name_type() == TNamingNameType::Orientation {
            return Some(self.name.orientation());
        }
        self.child_orientation
    }
}

/// The child loop of the OR-N code: the first child naming of type `ORIENTATION`.
fn child_orientation(label: &TdfLabel) -> Result<Option<Orientation>, Failure> {
    let mut children = TdfChildIterator::new(label, false)?;
    while children.more() {
        let mut child: Handle<TNamingNaming> = Handle::default();
        children
            .value()
            .find_attribute_as(&TNamingNaming::get_id(), &mut child)?;
        if let Some(child) = child.get() {
            let child_name = child.get_name();
            if child_name.name_type() == TNamingNameType::Orientation {
                return Ok(Some(child_name.orientation()));
            }
        }
        children.next();
    }
    Ok(None)
}

/// `TopAbs_FORWARD` or `TopAbs_REVERSED`.
pub(crate) fn is_oriented(orientation: Orientation) -> bool {
    matches!(orientation, Orientation::Forward | Orientation::Reversed)
}

/// The two file-static `LastModif`: the last modifications of a shape, following the
/// modifications of each new shape (`updated` restricts them to updated labels).
pub(crate) fn last_modif(
    iterator: &mut TNamingNewShapeIterator,
    shape: &Shape,
    shapes: &mut IndexedMapOfShape,
    updated: Option<&TdfLabelMap>,
    deleted: &mut Vec<TdfLabel>,
) -> Result<(), Failure> {
    let mut modified = false;
    while iterator.more() {
        let label = iterator.label()?;
        if updated.is_some_and(|updated| !updated.contains(&label)) {
            iterator.next()?;
            continue;
        }
        if iterator.is_modification()? {
            modified = true;
            follow_modification(iterator, label, shapes, updated, deleted)?;
        }
        iterator.next()?;
    }
    if !modified {
        shapes.add(shape.clone());
    }
    Ok(())
}

/// One modification of `LastModif`: a final new shape is kept (a null one records a
/// deletion), else its own modifications are followed.
fn follow_modification(
    iterator: &TNamingNewShapeIterator,
    label: TdfLabel,
    shapes: &mut IndexedMapOfShape,
    updated: Option<&TdfLabelMap>,
    deleted: &mut Vec<TdfLabel>,
) -> Result<(), Failure> {
    let mut next = TNamingNewShapeIterator::from_new_shape_iterator(iterator)?;
    let new_shape = iterator.shape()?;
    if next.more() {
        return last_modif(&mut next, &new_shape, shapes, updated, deleted);
    }
    if new_shape.is_null() {
        deleted.push(label);
    } else {
        shapes.add(new_shape);
    }
    Ok(())
}

/// The file-static `FindModifUntil` of TNaming_Tool.cxx: the descendants made by `context`.
pub(crate) fn find_modif_until(
    iterator: &mut TNamingNewShapeIterator,
    shapes: &mut IndexedMapOfShape,
    context: &Handle<TNamingNamedShape>,
) -> Result<(), Failure> {
    while iterator.more() {
        let shape = iterator.shape()?;
        if !shape.is_null() {
            if iterator.named_shape()? == *context {
                shapes.add(shape);
            } else {
                let mut next = TNamingNewShapeIterator::from_new_shape_iterator(iterator)?;
                find_modif_until(&mut next, shapes, context)?;
            }
        }
        iterator.next()?;
    }
    Ok(())
}

/// `NCollection_Map<occ::handle<TNaming_NamedShape>>`.
pub type TNamingNamedShapeMap = Map<Handle<TNamingNamedShape>>;

/// `(TopAbs_ShapeEnum)(type + 1)`: the sub-shape type; `None` past `TopAbs_SHAPE`, which the
/// native code turns into an enumerator with no shape.
pub(crate) fn sub_shape_type(
    shape_type: rustycad_tkg3d::top_abs::ShapeEnum,
) -> Option<rustycad_tkg3d::top_abs::ShapeEnum> {
    rustycad_tkg3d::top_abs::ShapeEnum::ALL
        .get(shape_type as usize + 1)
        .copied()
}
