// SPDX-License-Identifier: GPL-2.0-only
//! Shared named fault source retains actual ShapeIterator More/Value race semantics.
use rustycad_tkbrep::topods::{Shape, TShape, TShapeBase};
use rustycad_tkernel::{
    impl_transient,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::atomic::{AtomicU32, Ordering};
struct FakeChangingChildCount {
    transient: StandardTransient,
    topology: TShapeBase,
    reads: AtomicU32,
}
impl_transient!(FakeChangingChildCount, "Martensite_FakeChangingChildCount", dyn TShape, transient, interfaces: [dyn TShape]);

impl FakeChangingChildCount {
    fn new(kind: ShapeEnum) -> Self {
        Self {
            transient: StandardTransient::default(),
            topology: TShapeBase::new(kind),
            reads: AtomicU32::new(0),
        }
    }
}

impl TShape for FakeChangingChildCount {
    fn tshape_base(&self) -> &TShapeBase {
        // Use the real iterator and its stored topology; no alternate traversal.
        &self.topology
    }

    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        let copied = Handle::new(Self::new(self.shape_type()));
        Ok(copied.cast())
    }

    fn nb_children(&self) -> i32 {
        let previous = self.reads.fetch_add(1, Ordering::Relaxed);
        i32::from(previous == 0)
    }
}

pub(crate) fn changing_shape(kind: ShapeEnum) -> Shape {
    let mut shape = Shape::default();
    let original = FakeChangingChildCount::new(kind);
    shape.set_tshape(&original.empty_copy().unwrap());
    shape
}
