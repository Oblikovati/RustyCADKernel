// Rust port of ApplicationFramework/TKTObj/TObj/TObj_LabelIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_LabelIterator`: the base of the iterators that walk OCAF labels.
use super::{TObjObject, TObjObjectIterator};
use rustycad_tkernel::{
    impl_transient_interface,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tklcaf::tdf::{TdfChildIterator, TdfLabel};

/// The protected fields `myNode`, `myIterator` and `myObject`.
#[derive(Default)]
pub struct TObjLabelIteratorBase {
    node: SharedState<TdfLabel>,
    iterator: SharedState<TdfChildIterator>,
    object: SharedState<Handle<dyn TObjObject>>,
}

impl TObjLabelIteratorBase {
    /// `TObj_LabelIterator()`: an empty iterator.
    pub fn new() -> Self {
        Self::default()
    }
    /// `TObj_LabelIterator(theLabel, isRecursive)`; `Standard_NullObject` for a null label.
    pub fn with_label(label: &TdfLabel, is_recursive: bool) -> Result<Self, Failure> {
        let base = Self::new();
        base.init(label, is_recursive)?;
        Ok(base)
    }
    /// `Init(theLabel, isRecursive)` (protected).
    pub fn init(&self, label: &TdfLabel, is_recursive: bool) -> Result<(), Failure> {
        self.iterator.write().initialize(label, is_recursive)
    }
    /// `myNode`.
    pub fn node(&self) -> TdfLabel {
        self.node.read().clone()
    }
    /// `myObject`.
    pub fn object(&self) -> Handle<dyn TObjObject> {
        self.object.read().clone()
    }
    /// Sets the current item (`myObject` and `myNode`).
    pub fn set_current(&self, object: &Handle<dyn TObjObject>, node: &TdfLabel) {
        *self.object.write() = object.clone();
        *self.node.write() = node.clone();
    }
    /// Sets `myObject` alone.
    pub fn set_object(&self, object: &Handle<dyn TObjObject>) {
        *self.object.write() = object.clone();
    }
    /// Sets `myNode` alone.
    pub fn set_node(&self, node: &TdfLabel) {
        *self.node.write() = node.clone();
    }
    /// A copy of `myIterator`; a step works on the copy and stores it back.
    pub fn child_iterator(&self) -> TdfChildIterator {
        self.iterator.read().clone()
    }
    /// Stores `myIterator` back.
    pub fn set_child_iterator(&self, iterator: TdfChildIterator) {
        *self.iterator.write() = iterator;
    }
}

/// `TObj_LabelIterator`'s members; the `TObj_ObjectIterator` members of a label iterator
/// are filled by [`crate::tobj_label_iterator_members`].
pub trait TObjLabelIterator: TObjObjectIterator {
    /// The embedded base with the native protected fields.
    fn label_iterator_base(&self) -> &TObjLabelIteratorBase;
    /// `MakeStep` (pure virtual): shifts the iterator to the next object.
    fn make_step(&self) -> Result<(), Failure>;
    /// `LabelValue`: the label of the current item.
    fn label_value(&self) -> TdfLabel {
        self.label_iterator_base().node()
    }
}
impl_transient_interface!(
    dyn TObjLabelIterator,
    "TObj_LabelIterator",
    dyn TObjObjectIterator
);

/// `TObj_LabelIterator::More`: a current node.
pub fn label_iterator_more<T: TObjLabelIterator + ?Sized>(iterator: &T) -> bool {
    !iterator.label_iterator_base().node().is_null()
}

/// `TObj_LabelIterator::Next`: clears the current item, then steps.
pub fn label_iterator_next<T: TObjLabelIterator + ?Sized>(iterator: &T) -> Result<(), Failure> {
    iterator
        .label_iterator_base()
        .set_current(&Handle::default(), &TdfLabel::default());
    iterator.make_step()
}

/// Fills the `TObj_ObjectIterator` impl of a `TObj_LabelIterator` with its `More`, `Next`
/// and `Value`.
#[macro_export]
macro_rules! tobj_label_iterator_members {
    () => {
        fn more(&self) -> bool {
            $crate::tobj::label_iterator_more(self)
        }
        fn next(&self) -> Result<(), rustycad_tkernel::standard::Failure> {
            $crate::tobj::label_iterator_next(self)
        }
        fn value(&self) -> rustycad_tkernel::standard::Handle<dyn $crate::tobj::TObjObject> {
            $crate::tobj::TObjLabelIterator::label_iterator_base(self).object()
        }
    };
}
