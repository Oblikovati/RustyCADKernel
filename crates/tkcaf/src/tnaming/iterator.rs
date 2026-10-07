// Rust port of ApplicationFramework/TKCAF/TNaming/TNaming_Iterator.hxx, .lxx and the
// TNaming_Iterator members of TNaming_NamedShape.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TNaming_Iterator`: the evolutions of one named shape, newest first.
use super::{TNamingEvolution, TNamingNamedShape, TNamingNode, TNamingNodePtr};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle, exceptions};
use rustycad_tklcaf::live;
use rustycad_tklcaf::tdf::{TdfAttribute, TdfLabel};
use std::sync::Arc;

/// `TNaming_Iterator`: walks the (old shape, new shape) pairs of a named shape.
///
/// ```
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingIterator};
/// use rustycad_tklcaf::tdf::TdfData;
/// let data = TdfData::new();
/// let label = data.root().find_child(1, true).unwrap();
/// let mut iterator = TNamingIterator::from_label(&label).unwrap();
/// assert!(!iterator.more());
/// assert!(iterator.next().is_err());
/// ```
pub struct TNamingIterator {
    pub(crate) node: TNamingNodePtr,
    pub(crate) transaction: i32,
}

impl TNamingIterator {
    /// `TNaming_Iterator(anAtt)`: the evolutions of the attribute (`Standard_NullObject`
    /// for a null handle, a crash natively).
    pub fn new(attribute: &Handle<TNamingNamedShape>) -> Result<Self, Failure> {
        Ok(Self::of(live(attribute, "TNaming_Iterator")?))
    }
    /// The iterator over a named shape reached by reference.
    pub(crate) fn of(attribute: &TNamingNamedShape) -> Self {
        Self {
            node: attribute.first_node(),
            transaction: -1,
        }
    }
    /// `TNaming_Iterator(aLabel)`: the evolutions of the label's named shape, none when it
    /// has none.
    pub fn from_label(label: &TdfLabel) -> Result<Self, Failure> {
        let mut attribute: Handle<TNamingNamedShape> = Handle::default();
        label.find_attribute_as(&TNamingNamedShape::get_id(), &mut attribute)?;
        Ok(Self {
            node: attribute.get().and_then(TNamingNamedShape::first_node),
            transaction: -1,
        })
    }
    /// `TNaming_Iterator(aLabel, aTrans)`: the evolutions of the version of the label's
    /// named shape that is valid in the transaction.
    pub fn from_label_in_transaction(label: &TdfLabel, transaction: i32) -> Result<Self, Failure> {
        let mut found: Handle<dyn TdfAttribute> = Handle::default();
        label.find_attribute_in_transaction(
            &TNamingNamedShape::get_id(),
            transaction,
            &mut found,
        )?;
        let attribute = found.cast::<TNamingNamedShape>();
        Ok(Self {
            node: attribute.get().and_then(TNamingNamedShape::first_node),
            transaction,
        })
    }
    /// `More`.
    pub fn more(&self) -> bool {
        self.node.is_some()
    }
    /// `Next`: `Standard_NoMoreObject` at the end.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        let node = self.current("TNaming_Iterator::Next", exceptions::no_more_object)?;
        self.node = node.next_same_attribute();
        Ok(())
    }
    /// `OldShape`: a null shape for a primitive; `Standard_NoSuchObject` at the end.
    pub fn old_shape(&self) -> Result<Shape, Failure> {
        let node = self.current("TNaming_Iterator::OldShape", exceptions::no_such_object)?;
        Ok(node.old().map(|shape| shape.shape()).unwrap_or_default())
    }
    /// `NewShape`: a null shape for a deletion; `Standard_NoSuchObject` at the end.
    pub fn new_shape(&self) -> Result<Shape, Failure> {
        let node = self.current("TNaming_Iterator::NewShape", exceptions::no_such_object)?;
        Ok(node
            .new_shape()
            .map(|shape| shape.shape())
            .unwrap_or_default())
    }
    /// `IsModification`: the evolution is `MODIFY` or `DELETE`.
    pub fn is_modification(&self) -> Result<bool, Failure> {
        let evolution = self.evolution()?;
        Ok(matches!(
            evolution,
            TNamingEvolution::Modify | TNamingEvolution::Delete
        ))
    }
    /// `Evolution`: the kind of the attribute; `Standard_NoSuchObject` at the end.
    pub fn evolution(&self) -> Result<TNamingEvolution, Failure> {
        let node = self.current(
            "TNaming_Iterator::IsModification",
            exceptions::no_such_object,
        )?;
        node_evolution(&node, "TNaming_Iterator::Evolution")
    }
    fn current(&self, who: &str, error: fn(&str) -> Failure) -> Result<Arc<TNamingNode>, Failure> {
        self.node.clone().ok_or_else(|| {
            error(&format!(
                "{who}: the iterator is at its end; expected More()"
            ))
        })
    }
}

/// `myNode->myAtt->myEvolution`.
pub(crate) fn node_evolution(node: &TNamingNode, who: &str) -> Result<TNamingEvolution, Failure> {
    let attribute = node.live_attribute(who)?;
    Ok(live(&attribute, who)?.evolution())
}
