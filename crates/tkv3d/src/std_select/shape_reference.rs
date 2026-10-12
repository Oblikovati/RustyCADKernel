//! Live shape fields retain native ownership once; reference adapters only lend their current contents.
use rustycad_tkbrep::topods::{Shape, TopoDSShapeReadSource};
use rustycad_tkernel::standard::{Failure, SharedState};
/// Actual shape member storage; `let fields=StdSelectShapeFields{shape:shape.clone()};`.
pub struct StdSelectShapeFields {
    /// Native shape field, exposed for genuine application subclass state.
    pub shape: Shape,
}
/// Const reference to the actual field, including later replacement; `reference.clone_shape()`.
pub struct StdSelectShapeRef<'a> {
    pub(super) fields: &'a SharedState<StdSelectShapeFields>,
}
impl StdSelectShapeRef<'_> {
    /// Check stable field reference identity without copying geometry; `reference.is_same_reference(&other)`.
    pub fn is_same_reference(&self, other: &Self) -> bool {
        let fields = self.fields;
        std::ptr::eq(fields, other.fields)
    }
    /// Acquire an explicit native owning copy; `let copy=reference.clone_shape();`.
    pub fn clone_shape(&self) -> Shape {
        let fields = self.fields.read();
        fields.shape.clone()
    }
    /// Test current field nullness without acquiring TShape ownership; `reference.is_null()`.
    pub fn is_null(&self) -> bool {
        let fields = self.fields.read();
        fields.shape.is_null()
    }
}
impl TopoDSShapeReadSource for StdSelectShapeRef<'_> {
    fn read_current_shape<R>(
        &self,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let fields = self.fields.read();
        let topology = fields.shape.tshape().observe();
        let location = fields.shape.location().clone();
        let orientation = fields.shape.orientation();
        drop(fields);
        Shape::with_reference_parts(&*topology, &location, orientation, project)
    }
}
