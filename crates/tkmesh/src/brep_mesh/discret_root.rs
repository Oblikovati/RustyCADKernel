// Rust translation of BRepMesh_DiscretRoot.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepMesh_DiscretRoot`: the interface of the meshing algorithms the factories make.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};

/// The protected data of `BRepMesh_DiscretRoot`: the shape and the done flag.
#[derive(Default)]
pub struct DiscretRootBase {
    shape: SharedState<Shape>,
    is_done: SharedState<bool>,
}

/// `BRepMesh_DiscretRoot`: a triangulation algorithm over a shape.
pub trait BRepMeshDiscretRoot: Transient {
    /// The shape and done flag; `algo.discret_root_base()`.
    fn discret_root_base(&self) -> &DiscretRootBase;

    /// `Perform(theRange)` (pure virtual): computes the triangulation of the shape;
    /// `algo.perform(&ProgressRange::default())?`.
    fn perform(&self, range: &ProgressRange<'_>) -> Result<(), Failure>;

    /// `SetShape(theShape)`; `algo.set_shape(&shape)`.
    fn set_shape(&self, shape: &Shape) {
        *self.discret_root_base().shape.write() = shape.clone();
    }
    /// `Shape()`; `algo.shape()`.
    fn shape(&self) -> Shape {
        self.discret_root_base().shape.snapshot()
    }
    /// `IsDone()`: whether the last Perform succeeded; `algo.is_done()`.
    fn is_done(&self) -> bool {
        *self.discret_root_base().is_done.read()
    }
    /// `setDone()` (protected); `algo.set_done()`.
    fn set_done(&self) {
        *self.discret_root_base().is_done.write() = true;
    }
    /// `setNotDone()` (protected); `algo.set_not_done()`.
    fn set_not_done(&self) {
        *self.discret_root_base().is_done.write() = false;
    }
    /// `init()` (protected virtual): nothing; `algo.init()`.
    fn init(&self) {}
}
impl_transient_interface!(
    dyn BRepMeshDiscretRoot,
    "BRepMesh_DiscretRoot",
    StandardTransient
);

#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkernel::impl_transient;

    #[derive(Default)]
    struct FakeDiscretRoot {
        transient: StandardTransient,
        base: DiscretRootBase,
    }
    impl_transient!(FakeDiscretRoot, "FakeDiscretRoot", dyn BRepMeshDiscretRoot, transient, interfaces: [dyn BRepMeshDiscretRoot]);
    impl BRepMeshDiscretRoot for FakeDiscretRoot {
        fn discret_root_base(&self) -> &DiscretRootBase {
            &self.base
        }
        fn perform(&self, _range: &ProgressRange<'_>) -> Result<(), Failure> {
            self.set_done();
            Ok(())
        }
    }

    #[test]
    fn done_flag_follows_perform_and_set_not_done() {
        let root = FakeDiscretRoot::default();
        assert!(!root.is_done());
        root.perform(&ProgressRange::default()).unwrap();
        assert!(root.is_done());
        root.set_not_done();
        assert!(!root.is_done());
        assert!(root.shape().is_null());
    }
}
