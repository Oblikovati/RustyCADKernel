// Rust port of ApplicationFramework/TKTObj/TObj/TObj_OcafObjectIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_OcafObjectIterator`: the TObj objects on the sub-labels of a label.
use super::TObjObjectBase;
use super::{TObjLabelIterator, TObjLabelIteratorBase, TObjObjectIterator};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, RuntimeType, StandardTransient},
};
use rustycad_tklcaf::tdf::TdfLabel;

/// `TObj_OcafObjectIterator`; `TObjOcafObjectIterator::new(&label, None, false, false)`.
pub struct TObjOcafObjectIterator {
    transient: StandardTransient,
    base: TObjLabelIteratorBase,
    kind: Option<&'static RuntimeType>,
    all_sub_children: bool,
}
impl_transient!(TObjOcafObjectIterator, "TObj_OcafObjectIterator", dyn TObjLabelIterator,
    transient, interfaces: [dyn TObjObjectIterator, dyn TObjLabelIterator]);

impl TObjOcafObjectIterator {
    /// `TObj_OcafObjectIterator(theLabel, theType, theRecursive, theAllSubChildren)`: the
    /// objects of that type (any when `None`) on the sub-labels (at all depths when
    /// recursive), stopping at the first object level unless `all_sub_children`.
    pub fn new(
        label: &TdfLabel,
        kind: Option<&'static RuntimeType>,
        recursive: bool,
        all_sub_children: bool,
    ) -> Result<Handle<TObjOcafObjectIterator>, Failure> {
        let iterator = Handle::new(Self {
            transient: StandardTransient::default(),
            base: TObjLabelIteratorBase::with_label(label, recursive)?,
            kind,
            all_sub_children,
        });
        iterator.make_step()?;
        Ok(iterator)
    }
    /// `myType` (protected): the type of the iterated objects, `None` for any.
    pub fn kind(&self) -> Option<&'static RuntimeType> {
        self.kind
    }
    /// `myAllSubChildren` (protected): the walk does not stop at the first object level.
    pub fn all_sub_children(&self) -> bool {
        self.all_sub_children
    }
}

impl TObjObjectIterator for TObjOcafObjectIterator {
    crate::tobj_label_iterator_members!();
}

impl TObjLabelIterator for TObjOcafObjectIterator {
    fn label_iterator_base(&self) -> &TObjLabelIteratorBase {
        &self.base
    }
    /// `MakeStep`: an object label stops the walk below it (unless all sub-children are
    /// wanted); any other label is entered.
    fn make_step(&self) -> Result<(), Failure> {
        let mut children = self.base.child_iterator();
        while children.more() && self.base.node().is_null() {
            let label = children.value();
            let mut object = Handle::default();
            if !TObjObjectBase::get_obj(&label, &mut object, false)? {
                children.next();
                continue;
            }
            if self.kind.is_none_or(|kind| object.is_kind(kind)) {
                self.base.set_current(&object, &label);
            }
            if self.all_sub_children {
                children.next();
            } else {
                children.next_brother();
            }
        }
        self.base.set_child_iterator(children);
        Ok(())
    }
}
