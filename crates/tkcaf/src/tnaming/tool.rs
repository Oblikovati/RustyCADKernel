// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Tool.hxx, TNaming_Tool.cxx and the
// TNaming_Tool members of TNaming_NamedShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Tool`: queries on the evolution graph (current shapes, the named shape of a
//! shape, its label and life span, its initial shapes).
use super::named_shape::used_shapes_of;
use super::node::same_pointer;
use super::tool_shapes::{
    SelectionNaming, TNamingNamedShapeMap, apply_orientation, find_modif_until, is_oriented,
    last_modif, make_shape,
};
use super::{
    TNamingEvolution, TNamingIterator, TNamingNamedShape, TNamingNewShapeIterator, TNamingNode,
    TNamingOldShapeIterator, TNamingUsedShapes,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfLabelMap};
use std::sync::Arc;

/// `TNaming_Tool`.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkcaf::tnaming::TNamingTool;
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// assert!(!TNamingTool::has_label(&data.root(), &Shape::default()).unwrap());
/// assert!(TNamingTool::named_shape(&Shape::default(), &data.root()).unwrap().is_null());
/// ```
pub struct TNamingTool;

impl TNamingTool {
    /// `CurrentShape(NS)`: the last modifications of the new shapes of `NS`.
    pub fn current_shape(named: &Handle<TNamingNamedShape>) -> Result<Shape, Failure> {
        current_shape_in(live(named, "TNaming_Tool::CurrentShape")?, None)
    }
    /// `CurrentShape(NS, Updated)`: the same, following only the updated labels; a null
    /// shape when the label of `NS` is not updated.
    pub fn current_shape_updated(
        named: &Handle<TNamingNamedShape>,
        updated: &TdfLabelMap,
    ) -> Result<Shape, Failure> {
        let named = live(named, "TNaming_Tool::CurrentShape")?;
        if !updated.contains(&named.label()) {
            return Ok(Shape::default());
        }
        current_shape_in(named, Some(updated))
    }
    /// `CurrentNamedShape(NS, Updated)`: the named shape of the current shape.
    pub fn current_named_shape_updated(
        named: &Handle<TNamingNamedShape>,
        updated: &TdfLabelMap,
    ) -> Result<Handle<TNamingNamedShape>, Failure> {
        let current = Self::current_shape_updated(named, updated)?;
        Self::named_shape_of_current(&current, named)
    }
    /// `CurrentNamedShape(NS)`.
    pub fn current_named_shape(
        named: &Handle<TNamingNamedShape>,
    ) -> Result<Handle<TNamingNamedShape>, Failure> {
        let current = Self::current_shape(named)?;
        Self::named_shape_of_current(&current, named)
    }
    fn named_shape_of_current(
        current: &Shape,
        named: &Handle<TNamingNamedShape>,
    ) -> Result<Handle<TNamingNamedShape>, Failure> {
        if current.is_null() {
            return Ok(Handle::default());
        }
        Self::named_shape(
            current,
            &live(named, "TNaming_Tool::CurrentNamedShape")?.label(),
        )
    }
    /// `NamedShape(aShape, anAcces)`: the named shape that made `aShape` (a modification is
    /// preferred to a generation), as currently on its label; null when none.
    pub fn named_shape(
        shape: &Shape,
        access: &TdfLabel,
    ) -> Result<Handle<TNamingNamedShape>, Failure> {
        let used = used_shapes_of(access)?;
        let Some(used) = used.get() else {
            return Ok(Handle::default());
        };
        let Some(reference) = used.map().seek(shape).cloned() else {
            return Ok(Handle::default());
        };
        let reference = Some(reference);
        let mut found: Option<Arc<TNamingNode>> = None;
        let mut current = reference.as_ref().and_then(|r| r.first_use());
        while let Some(node) = current {
            if same_pointer(&node.new_shape(), &reference) {
                let evolution = node_evolution(&node)?;
                if evolution != TNamingEvolution::Selected {
                    found = Some(node.clone());
                    // Modifications are preferred to generations (tangent faces).
                    if evolution != TNamingEvolution::Generated {
                        break;
                    }
                }
            }
            current = node.next_same_shape(&reference);
        }
        let Some(found) = found else {
            return Ok(Handle::default());
        };
        // Protection against returning a backed-up attribute: the label's current one.
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        found
            .label()?
            .find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?;
        Ok(named)
    }
    /// `GetShape(NS)`: the new shapes of `NS` (a compound when several); the selections
    /// take the orientation of their naming.
    pub fn get_shape(named: &Handle<TNamingNamedShape>) -> Result<Shape, Failure> {
        Self::shape_of(live(named, "TNaming_Tool::GetShape")?)
    }
    /// The body of `GetShape` on a named shape reached by reference.
    pub(crate) fn shape_of(named: &TNamingNamedShape) -> Result<Shape, Failure> {
        let mut shapes = IndexedMapOfShape::default();
        let mut iterator = TNamingIterator::of(named);
        let selected = named.evolution() == TNamingEvolution::Selected;
        while iterator.more() {
            let new_shape = iterator.new_shape()?;
            if !new_shape.is_null() {
                let shape = if selected && new_shape.shape_type()? != ShapeEnum::Vertex {
                    selected_shape(named, new_shape)?
                } else {
                    new_shape
                };
                shapes.add(shape);
            }
            iterator.next()?;
        }
        make_shape(&shapes)
    }
    /// `OriginalShape(NS)`: the old shapes of `NS`.
    pub fn original_shape(named: &Handle<TNamingNamedShape>) -> Result<Shape, Failure> {
        let mut shapes = IndexedMapOfShape::default();
        let mut iterator = TNamingIterator::new(named)?;
        while iterator.more() {
            shapes.add(iterator.old_shape()?);
            iterator.next()?;
        }
        make_shape(&shapes)
    }
    /// `GeneratedShape(S, Generation)`: the shapes `Generation` made from `S`, else the
    /// ones it made from the descendants of `S`.
    pub fn generated_shape(
        shape: &Shape,
        generation: &Handle<TNamingNamedShape>,
    ) -> Result<Shape, Failure> {
        let named = live(generation, "TNaming_Tool::GeneratedShape")?;
        let used = used_shapes_of(&named.label())?;
        let mut shapes = IndexedMapOfShape::default();
        let mut iterator = TNamingNewShapeIterator::from_used_shapes(shape, &used)?;
        while iterator.more() {
            let new_shape = iterator.shape()?;
            if !new_shape.is_null() && iterator.named_shape()? == *generation {
                shapes.add(new_shape);
            }
            iterator.next()?;
        }
        if shapes.is_empty() {
            let mut descendants = TNamingNewShapeIterator::from_used_shapes(shape, &used)?;
            find_modif_until(&mut descendants, &mut shapes, generation)?;
        }
        make_shape(&shapes)
    }
    /// `Collect(NS, Labels, OnlyModif)`: `NS`, the named shapes it comes from and,
    /// recursively, those that modify (or, without `OnlyModif`, use) its new shapes.
    pub fn collect(
        named: &Handle<TNamingNamedShape>,
        collected: &mut TNamingNamedShapeMap,
        only_modif: bool,
    ) -> Result<(), Failure> {
        collected.add(named.clone());
        back(named, collected)?;
        let mut iterator = TNamingIterator::new(named)?;
        while iterator.more() {
            if !iterator.new_shape()?.is_null() {
                let mut descendants = TNamingNewShapeIterator::from_iterator(&iterator)?;
                while descendants.more() {
                    if !only_modif || descendants.is_modification()? {
                        Self::collect(&descendants.named_shape()?, collected, only_modif)?;
                    }
                    descendants.next()?;
                }
            }
            iterator.next()?;
        }
        Ok(())
    }
    /// `HasLabel(access, aShape)`: the shape is in the root's `TNaming_UsedShapes`.
    pub fn has_label(access: &TdfLabel, shape: &Shape) -> Result<bool, Failure> {
        let used = used_shapes_of(access)?;
        Ok(used.get().is_some_and(|used| used.map().is_bound(shape)))
    }
    /// `Label(access, aShape, TransDef)`: the label that made the shape and the transaction
    /// of its named shape; `Standard_NoSuchObject` for a shape without label.
    pub fn label(
        access: &TdfLabel,
        shape: &Shape,
        transaction: &mut i32,
    ) -> Result<TdfLabel, Failure> {
        if !Self::has_label(access, shape)? {
            return Err(no_label("TNaming_Tool::Label"));
        }
        Self::label_in(&used_shapes_of(access)?, shape, transaction)
    }
    /// `InitialShape(aShape, anAcces, Labels)`: the first ancestors of the shape through
    /// modifications, with the labels that made them.
    pub fn initial_shape(
        shape: &Shape,
        access: &TdfLabel,
        labels: &mut Vec<TdfLabel>,
    ) -> Result<Shape, Failure> {
        let used = used_shapes_of(access)?;
        if !Self::has_label_in(&used, shape)? {
            return Ok(Shape::default());
        }
        let mut transaction = 0;
        Self::label_in(&used, shape, &mut transaction)?;
        let mut iterator =
            TNamingOldShapeIterator::from_used_shapes_in_transaction(shape, transaction, &used)?;
        if !iterator.more() {
            return Ok(shape.clone());
        }
        let mut shapes = IndexedMapOfShape::default();
        Self::first_olds(&used, shape, &mut iterator, &mut shapes, labels)?;
        make_shape(&shapes)
    }
    /// `ValidUntil(access, S)`: the last transaction in which a named shape made `S`.
    pub fn valid_until(access: &TdfLabel, shape: &Shape) -> Result<i32, Failure> {
        if !Self::has_label(access, shape)? {
            return Err(no_label("TNaming_Tool::ValidUntil"));
        }
        Self::valid_until_in(shape, &used_shapes_of(access)?)
    }
    /// The private `HasLabel(Shapes, aShape)`.
    pub(crate) fn has_label_in(
        shapes: &Handle<TNamingUsedShapes>,
        shape: &Shape,
    ) -> Result<bool, Failure> {
        Ok(live(shapes, "TNaming_Tool::HasLabel")?
            .map()
            .is_bound(shape))
    }
    /// The private `ValidUntil(S, US)`.
    pub(crate) fn valid_until_in(
        shape: &Shape,
        shapes: &Handle<TNamingUsedShapes>,
    ) -> Result<i32, Failure> {
        let reference = find_reference(shapes, shape, "TNaming_Tool::ValidUntil")?;
        let mut until = 0;
        let mut current = reference.as_ref().and_then(|r| r.first_use());
        while let Some(node) = current {
            if node.new_shape().is_some() && same_pointer(&node.new_shape(), &reference) {
                let named = node.live_attribute("TNaming_Tool::ValidUntil")?;
                until = until.max(live(&named, "TNaming_Tool::ValidUntil")?.until_transaction()?);
            }
            current = node.next_same_shape(&reference);
        }
        Ok(until)
    }
    /// The private `Label(Shapes, aShape, TransDef)`: the first non-selection node that
    /// made the shape, else its first use.
    pub(crate) fn label_in(
        shapes: &Handle<TNamingUsedShapes>,
        shape: &Shape,
        transaction: &mut i32,
    ) -> Result<TdfLabel, Failure> {
        let reference = find_reference(shapes, shape, "TNaming_Tool::Label")?;
        let first = reference.as_ref().and_then(|r| r.first_use());
        let mut current = first.clone();
        while let Some(node) = current.clone() {
            let creates = same_pointer(&node.new_shape(), &reference)
                && node_evolution(&node)? != TNamingEvolution::Selected;
            if creates {
                break;
            }
            current = node.next_same_shape(&reference);
        }
        let node = current.or(first).ok_or_else(|| {
            exceptions::null_object(
                "TNaming_Tool::Label: the shape has no use; expected a used shape",
            )
        })?;
        let named = node.live_attribute("TNaming_Tool::Label")?;
        *transaction = live(&named, "TNaming_Tool::Label")?.transaction();
        node.label()
    }
    /// The private `FirstOlds`: the modification ancestors of `S` reached by `it`.
    pub(crate) fn first_olds(
        shapes: &Handle<TNamingUsedShapes>,
        shape: &Shape,
        iterator: &mut TNamingOldShapeIterator,
        found: &mut IndexedMapOfShape,
        labels: &mut Vec<TdfLabel>,
    ) -> Result<(), Failure> {
        let mut modified = false;
        while iterator.more() {
            if iterator.is_modification()? {
                modified = true;
                let old_shape = iterator.shape()?;
                let mut older = TNamingOldShapeIterator::from_old_shape_iterator(iterator)?;
                if older.more() {
                    Self::first_olds(shapes, &old_shape, &mut older, found, labels)?;
                } else {
                    let mut transaction = 0;
                    labels.push(Self::label_in(shapes, &old_shape, &mut transaction)?);
                    found.add(old_shape);
                }
            }
            iterator.next()?;
        }
        if !modified {
            found.add(shape.clone());
        }
        Ok(())
    }
}

/// The shared body of both `CurrentShape`.
fn current_shape_in(
    named: &TNamingNamedShape,
    updated: Option<&TdfLabelMap>,
) -> Result<Shape, Failure> {
    let mut shapes = IndexedMapOfShape::default();
    let mut deleted: Vec<TdfLabel> = Vec::new();
    let mut iterator = TNamingIterator::of(named);
    while iterator.more() {
        let shape = iterator.new_shape()?;
        if !shape.is_null() {
            let orientation = orientation_to_apply(named, &shape)?;
            let mut descendants = TNamingNewShapeIterator::from_iterator(&iterator)?;
            if descendants.more() {
                let mut last = IndexedMapOfShape::default();
                last_modif(&mut descendants, &shape, &mut last, updated, &mut deleted)?;
                if let Some(orientation) = orientation {
                    apply_orientation(&mut last, orientation)?;
                }
                for index in 1..=last.extent() {
                    shapes.add(last.find_key(index)?.clone());
                }
            } else {
                shapes.add(shape);
            }
        }
        iterator.next()?;
    }
    make_shape(&shapes)
}

/// The OR-N block of `CurrentShape`: for a selection of a non-vertex with a naming, the
/// orientation of an `ORIENTATION` name (or child name, `FORWARD` without one), when it is
/// `FORWARD` or `REVERSED`.
fn orientation_to_apply(
    named: &TNamingNamedShape,
    shape: &Shape,
) -> Result<Option<Orientation>, Failure> {
    if named.evolution() != TNamingEvolution::Selected || shape.shape_type()? == ShapeEnum::Vertex {
        return Ok(None);
    }
    let Some(naming) = SelectionNaming::find(&named.label())? else {
        return Ok(None);
    };
    let orientation = naming.orientation().unwrap_or(Orientation::Forward);
    Ok(is_oriented(orientation).then_some(orientation))
}

/// The OR-N block of `GetShape`: a naming oriented `FORWARD` or `REVERSED` orients the
/// selected shape (its own orientation for an `ORIENTATION` name, else its orientation
/// child's, if any).
fn selected_shape(named: &TNamingNamedShape, shape: Shape) -> Result<Shape, Failure> {
    let Some(naming) = SelectionNaming::find(&named.label())? else {
        return Ok(shape);
    };
    if !is_oriented(naming.name.orientation()) {
        return Ok(shape);
    }
    Ok(match naming.orientation() {
        Some(orientation) => shape.oriented(orientation),
        None => shape,
    })
}

/// The file-static `Back`: the named shapes the new shapes of `NS` come from, recursively.
fn back(
    named: &Handle<TNamingNamedShape>,
    collected: &mut TNamingNamedShapeMap,
) -> Result<(), Failure> {
    let label = live(named, "TNaming_Tool::Collect")?.label();
    let mut iterator = TNamingIterator::new(named)?;
    while iterator.more() {
        if !iterator.new_shape()?.is_null() {
            let mut olds = TNamingOldShapeIterator::from_iterator(&iterator)?;
            while olds.more() {
                let old_shape = olds.shape()?;
                if !old_shape.is_null() {
                    let origin = TNamingTool::named_shape(&old_shape, &label)?;
                    if origin.is_some() && collected.add(origin.clone()) {
                        back(&origin, collected)?;
                    }
                }
                olds.next()?;
            }
        }
        iterator.next()?;
    }
    Ok(())
}

/// `myAtt->Evolution()` of a node.
fn node_evolution(node: &TNamingNode) -> Result<TNamingEvolution, Failure> {
    super::iterator::node_evolution(node, "TNaming_Tool")
}

/// The ref shape of a bound shape; `Standard_NoSuchObject` when it is not bound.
fn find_reference(
    shapes: &Handle<TNamingUsedShapes>,
    shape: &Shape,
    who: &str,
) -> Result<super::TNamingRefShapePtr, Failure> {
    let reference = live(shapes, who)?.map().seek(shape).cloned();
    if reference.is_none() {
        return Err(no_label(who));
    }
    Ok(reference)
}

/// The `Standard_NoSuchObject_Raise_if(!HasLabel(...))` of the label queries.
fn no_label(who: &str) -> Failure {
    exceptions::no_such_object(&format!(
        "{who}: the shape is not in the TNaming_UsedShapes map; expected a named shape"
    ))
}
