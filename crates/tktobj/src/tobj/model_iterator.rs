// Rust port of ApplicationFramework/TKTObj/TObj/TObj_ModelIterator.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TObj_ModelIterator`: every object of a model, depth first from the root.
use super::{TObjModel, TObjObject, TObjObjectIterator};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, SharedState, StandardTransient},
};

/// `TObj_ModelIterator`; `TObjModelIterator::new(&model)`.
pub struct TObjModelIterator {
    transient: StandardTransient,
    object: SharedState<Handle<dyn TObjObject>>,
    iterators: SharedState<Vec<Handle<dyn TObjObjectIterator>>>,
}
impl_transient!(TObjModelIterator, "TObj_ModelIterator", dyn TObjObjectIterator, transient,
    interfaces: [dyn TObjObjectIterator]);

impl TObjModelIterator {
    /// `TObj_ModelIterator(theModel)`: starts at the root of the model.
    pub fn new(model: &Handle<dyn TObjModel>) -> Result<Handle<TObjModelIterator>, Failure> {
        let root =
            rustycad_tkcdf::live(model, "TObj_ModelIterator::TObj_ModelIterator")?.get_root()?;
        let iterator = Handle::new(Self {
            transient: StandardTransient::default(),
            object: SharedState::new(root.clone()),
            iterators: SharedState::new(Vec::new()),
        });
        if !root.is_null() {
            iterator.add_iterator(&root)?;
        }
        Ok(iterator)
    }
    /// `myObject` (protected): the current object.
    pub fn object(&self) -> Handle<dyn TObjObject> {
        self.object.read().clone()
    }
    /// `myIterSeq` (protected): the stack of child iterators, innermost last.
    pub fn iterators(&self) -> Vec<Handle<dyn TObjObjectIterator>> {
        self.iterators.read().clone()
    }
    /// `addIterator(theObj)` (protected): pushes the iterator on the object's children.
    pub fn add_iterator(&self, object: &Handle<dyn TObjObject>) -> Result<(), Failure> {
        let children =
            rustycad_tkcdf::live(object, "TObj_ModelIterator::addIterator")?.get_children(None)?;
        if children.is_null() {
            return Ok(()); // object has no children
        }
        self.iterators.write().push(children);
        Ok(())
    }
}

impl TObjObjectIterator for TObjModelIterator {
    /// `More`: a current object.
    fn more(&self) -> bool {
        !self.object.read().is_null()
    }
    /// `Next`: the next object of the innermost iterator that has one (its children are
    /// visited next).
    fn next(&self) -> Result<(), Failure> {
        *self.object.write() = Handle::default();
        loop {
            let last = self.iterators.read().last().cloned();
            let Some(last) = last else {
                return Ok(());
            };
            let iterator = rustycad_tkcdf::live(&last, "TObj_ModelIterator::Next")?;
            if iterator.more() {
                let object = iterator.value();
                *self.object.write() = object.clone();
                iterator.next()?;
                return self.add_iterator(&object);
            }
            self.iterators.write().pop();
        }
    }
    /// `Value`: the current object.
    fn value(&self) -> Handle<dyn TObjObject> {
        self.object.read().clone()
    }
}
