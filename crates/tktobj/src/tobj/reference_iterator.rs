// Rust port of ApplicationFramework/TKTObj/TObj/TObj_ReferenceIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_ReferenceIterator`: the objects referred by the `TObj_TReference` attributes
//! under a label.
use super::{TObjLabelIterator, TObjLabelIteratorBase, TObjObjectIterator, TObjTReference};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, RuntimeType, StandardTransient},
};
use rustycad_tklcaf::tdf::TdfLabel;

/// `TObj_ReferenceIterator`; `TObjReferenceIterator::new(&label, None, true)`.
pub struct TObjReferenceIterator {
    transient: StandardTransient,
    base: TObjLabelIteratorBase,
    kind: Option<&'static RuntimeType>,
}
impl_transient!(TObjReferenceIterator, "TObj_ReferenceIterator", dyn TObjLabelIterator,
    transient, interfaces: [dyn TObjObjectIterator, dyn TObjLabelIterator]);

impl TObjReferenceIterator {
    /// `TObj_ReferenceIterator(theLabel, theType, theRecursive)`.
    pub fn new(
        label: &TdfLabel,
        kind: Option<&'static RuntimeType>,
        recursive: bool,
    ) -> Result<Handle<TObjReferenceIterator>, Failure> {
        let iterator = Handle::new(Self {
            transient: StandardTransient::default(),
            base: TObjLabelIteratorBase::with_label(label, recursive)?,
            kind,
        });
        iterator.make_step()?;
        Ok(iterator)
    }
    /// `myType` (protected): the type of the referred objects, `None` for any.
    pub fn kind(&self) -> Option<&'static RuntimeType> {
        self.kind
    }
}

impl TObjObjectIterator for TObjReferenceIterator {
    crate::tobj_label_iterator_members!();
}

impl TObjLabelIterator for TObjReferenceIterator {
    fn label_iterator_base(&self) -> &TObjLabelIteratorBase {
        &self.base
    }
    /// `MakeStep`: the next label with a reference to an object of the type (a null
    /// referred object is reported too, as natively).
    fn make_step(&self) -> Result<(), Failure> {
        let mut children = self.base.child_iterator();
        while children.more() && self.base.node().is_null() {
            let label = children.value();
            let mut reference: Handle<TObjTReference> = Handle::default();
            if label.find_attribute_as(&TObjTReference::get_id(), &mut reference)? {
                let object = reference.get_object()?;
                self.base.set_object(&object);
                let filtered = self
                    .kind
                    .is_some_and(|kind| object.is_some() && !object.is_kind(kind));
                if !filtered {
                    self.base.set_node(&label);
                }
            }
            children.next();
        }
        self.base.set_child_iterator(children);
        Ok(())
    }
}
