// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_NamingTool.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_NamingTool`: the current shapes of a name argument within valid and
//! forbidden labels, and the descendants of a named shape.
use super::tool::orientation_to_apply;
use super::tool_shapes::apply_orientation;
use super::{
    TNamingEvolution, TNamingIterator, TNamingNamedShape, TNamingNewShapeIterator,
    TNamingOldShapeIterator, TNamingTool,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfLabelMap};

/// `TNaming_NamingTool`.
///
/// ```
/// use rustycad_tkcaf::tnaming::{TNamingNamedShape, TNamingNamingTool};
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tklcaf::tdf::TdfLabelMap;
/// let mut labels = TdfLabelMap::default();
/// TNamingNamingTool::build_descendants(&Handle::<TNamingNamedShape>::default(), &mut labels).unwrap();
/// assert!(labels.is_empty());
/// ```
pub struct TNamingNamingTool;

impl TNamingNamingTool {
    /// `CurrentShape(Valid, Forbiden, NS, MS)`: the last modifications of the new shapes of
    /// `NS` through the valid (all when empty), not forbidden labels; nothing when the label
    /// of `NS` is not valid. A selection takes the orientation of its naming.
    pub fn current_shape(
        valid: &TdfLabelMap,
        forbiden: &TdfLabelMap,
        named: &Handle<TNamingNamedShape>,
        shapes: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        let named_ref = live(named, "TNaming_NamingTool::CurrentShape")?;
        let label = named_ref.label();
        if !valid.is_empty() && !valid.contains(&label) {
            return Ok(());
        }
        let mut iterator = TNamingIterator::new(named)?;
        while iterator.more() {
            let shape = iterator.new_shape()?;
            if !shape.is_null() {
                let orientation = orientation_to_apply(named_ref, &shape)?;
                add_current(&iterator, &shape, orientation, valid, forbiden, shapes)?;
            }
            iterator.next()?;
        }
        Ok(())
    }
    /// `CurrentShapeFromShape(Valid, Forbiden, Acces, S, MS)`: the shape of a selection that
    /// uses `S`, else the last modifications of `S`. The native code asks the iterator for
    /// its named shape before checking it, so a shape without descendant is
    /// `Standard_NoSuchObject`, as natively.
    pub fn current_shape_from_shape(
        valid: &TdfLabelMap,
        forbiden: &TdfLabelMap,
        access: &TdfLabel,
        shape: &Shape,
        shapes: &mut IndexedMapOfShape,
    ) -> Result<(), Failure> {
        let mut iterator = TNamingNewShapeIterator::from_label(shape, access)?;
        let named = iterator.named_shape()?;
        if named
            .get()
            .is_some_and(|named| named.evolution() == TNamingEvolution::Selected)
        {
            shapes.add(TNamingTool::get_shape(&named)?);
            return Ok(());
        }
        if !iterator.more() {
            shapes.add(shape.clone());
            return Ok(());
        }
        last_modif(&mut iterator, shape, shapes, valid, forbiden)
    }
    /// `BuildDescendants(NS, Labels)`: the label of `NS`, the labels of every descendant of
    /// its first new shape, and those of the named shapes that made its first new shape
    /// (but not `NS` itself), recursively.
    pub fn build_descendants(
        named: &Handle<TNamingNamedShape>,
        descendants: &mut TdfLabelMap,
    ) -> Result<(), Failure> {
        let Some(named_ref) = named.get() else {
            return Ok(());
        };
        let label = named_ref.label();
        descendants.add(label.clone());
        let first = TNamingIterator::new(named)?;
        let mut news = TNamingNewShapeIterator::from_iterator(&first)?;
        make_descendants(&mut news, descendants)?;
        let mut olds = TNamingOldShapeIterator::from_iterator(&first)?;
        while olds.more() {
            let old_shape = olds.shape()?;
            if !old_shape.is_null() {
                let origin = TNamingTool::named_shape(&old_shape, &label)?;
                if origin.is_some() {
                    build_descendants2(&origin, &label, descendants)?;
                }
            }
            olds.next()?;
        }
        Ok(())
    }
}

/// One new shape of `CurrentShape`: itself (oriented) without descendant, else its last
/// modifications (oriented).
fn add_current(
    iterator: &TNamingIterator,
    shape: &Shape,
    orientation: Option<rustycad_tkg3d::top_abs::Orientation>,
    valid: &TdfLabelMap,
    forbiden: &TdfLabelMap,
    shapes: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    let mut descendants = TNamingNewShapeIterator::from_iterator(iterator)?;
    if !descendants.more() {
        shapes.add(orientation.map_or_else(|| shape.clone(), |o| shape.oriented(o)));
        return Ok(());
    }
    let mut last = IndexedMapOfShape::default();
    last_modif(&mut descendants, shape, &mut last, valid, forbiden)?;
    if let Some(orientation) = orientation {
        // The solution to be refined (native comment).
        apply_orientation(&mut last, orientation)?;
    }
    for index in 1..=last.extent() {
        shapes.add(last.find_key(index)?.clone());
    }
    Ok(())
}

/// The file-static `IsForbiden`: the label or one of its fathers (not the root) is
/// forbidden.
fn is_forbiden(forbiden: &TdfLabelMap, label: &TdfLabel) -> bool {
    let mut current = label.clone();
    while !current.is_root() {
        if forbiden.contains(&current) {
            return true;
        }
        current = current.father();
    }
    false
}

/// The file-static `LastModif` of TNaming_NamingTool.cxx.
fn last_modif(
    iterator: &mut TNamingNewShapeIterator,
    shape: &Shape,
    shapes: &mut IndexedMapOfShape,
    updated: &TdfLabelMap,
    forbiden: &TdfLabelMap,
) -> Result<(), Failure> {
    let mut modified = false;
    while iterator.more() {
        let label = iterator.label()?;
        let skipped =
            (!updated.is_empty() && !updated.contains(&label)) || is_forbiden(forbiden, &label);
        if !skipped && iterator.is_modification()? {
            modified = true;
            let mut next = TNamingNewShapeIterator::from_new_shape_iterator(iterator)?;
            if next.more() {
                last_modif(&mut next, &iterator.shape()?, shapes, updated, forbiden)?;
            } else {
                shapes.add(iterator.shape()?);
            }
        }
        iterator.next()?;
    }
    if !modified {
        shapes.add(shape.clone());
    }
    Ok(())
}

/// The file-static `MakeDescendants`.
fn make_descendants(
    iterator: &mut TNamingNewShapeIterator,
    descendants: &mut TdfLabelMap,
) -> Result<(), Failure> {
    while iterator.more() {
        descendants.add(iterator.label()?);
        if !iterator.shape()?.is_null() {
            let mut next = TNamingNewShapeIterator::from_new_shape_iterator(iterator)?;
            make_descendants(&mut next, descendants)?;
        }
        iterator.next()?;
    }
    Ok(())
}

/// The global `BuildDescendants2` of TNaming_NamingTool.cxx: the descendants of the first new
/// shape of `NS`, except through the label `forbidden`.
fn build_descendants2(
    named: &Handle<TNamingNamedShape>,
    forbidden: &TdfLabel,
    descendants: &mut TdfLabelMap,
) -> Result<(), Failure> {
    if named.is_null() {
        return Ok(());
    }
    let first = TNamingIterator::new(named)?;
    let mut iterator = TNamingNewShapeIterator::from_iterator(&first)?;
    while iterator.more() {
        if iterator.named_shape()?.is_some() {
            let label = iterator.label()?;
            if label != *forbidden {
                descendants.add(label);
                let mut next = TNamingNewShapeIterator::from_new_shape_iterator(&iterator)?;
                make_descendants(&mut next, descendants)?;
            }
        }
        iterator.next()?;
    }
    Ok(())
}
