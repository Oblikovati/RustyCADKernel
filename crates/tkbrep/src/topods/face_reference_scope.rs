//! Martensite scoped face views preserve native read-only topology ownership.
use super::{Face, Shape, edge_reference_scope::with_typed_shape_reference};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
impl Face {
    /// Lend a checked face without another topology owner; `Face::with_shape_reference(&shape,|face|Ok(face.nb_children()))?`.
    ///
    /// ```compile_fail
    /// use rustycad_tkbrep::topods::{Face,Shape};
    /// let shape=Shape::default();
    /// let escaped=Face::with_shape_reference(&shape,|face|Ok(face));
    /// ```
    pub fn with_shape_reference<R>(
        shape: &Shape,
        project: impl FnOnce(&Face) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        with_typed_shape_reference(
            shape,
            ShapeEnum::Face,
            "TopoDS::Face",
            Face::unchecked,
            project,
        )
    }
}
