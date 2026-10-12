//! Bounded Martensite edge views avoid topology copies hidden by the owned convenience cast.
use super::{Edge, Shape, shape_reference_scope::scoped_reference_shape, typed::check_cast};
use rustycad_tkernel::standard::{Failure, catch_native_failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
impl Edge {
    /// Lend a checked edge for one read-only operation without owning another TShape.
    /// Example: `Edge::with_shape_reference(&shape, |edge| Ok(edge.nb_children()))?`.
    ///
    /// ```compile_fail
    /// use rustycad_tkbrep::topods::{Edge,Shape};
    /// let shape=Shape::default();
    /// let escaped=Edge::with_shape_reference(&shape,|edge|Ok(edge));
    /// ```
    pub fn with_shape_reference<R>(
        shape: &Shape,
        project: impl FnOnce(&Edge) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        with_typed_shape_reference(
            shape,
            ShapeEnum::Edge,
            "TopoDS::Edge",
            Edge::unchecked,
            project,
        )
    }
}

pub(super) fn with_typed_shape_reference<T, R>(
    shape: &Shape,
    kind: ShapeEnum,
    cast: &str,
    wrap: fn(Shape) -> T,
    project: impl FnOnce(&T) -> Result<R, Failure>,
) -> Result<R, Failure> {
    catch_native_failure(|| {
        check_cast(shape, kind, cast)?;
        let frame = scoped_reference_shape(shape.tshape(), shape.location(), shape.orientation());
        let typed = wrap(frame);
        project(&typed)
    })
}
