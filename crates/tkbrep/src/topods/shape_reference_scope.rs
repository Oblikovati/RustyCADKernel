//! Martensite's bounded read-only topology frame lends actual observed handles; it never exports an uncounted owning value.
use super::{Shape, TShape, shape_topology_storage::ShapeTopologyStorage};
use rustycad_tkernel::standard::{Failure, HandleReference, catch_native_failure};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;
impl Shape {
    /// Lend a read-only shape for one callback without retaining its TShape; `Shape::with_reference_parts(slot,&location,orientation,|shape|Ok(shape.nb_children()))?`.
    /// The callback cannot return the borrowed frame; cloning it explicitly acquires native ownership.
    ///
    /// ```compile_fail
    /// use rustycad_tkbrep::topods::{Shape,TCompound,TShape};
    /// use rustycad_tkernel::standard::Handle;
    /// use rustycad_tkg3d::top_abs::Orientation;
    /// use rustycad_tkmath::toploc::Location;
    /// let topology:Handle<dyn TShape>=Handle::new(TCompound::new()).cast();
    /// let escaped=Shape::with_reference_parts(&topology,&Location::default(),Orientation::Forward,|shape|Ok(shape));
    /// ```
    pub fn with_reference_parts<R>(
        topology: &dyn HandleReference<dyn TShape>,
        location: &Location,
        orientation: Orientation,
        project: impl FnOnce(&Shape) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        catch_native_failure(|| {
            let shape = Shape {
                tshape: ShapeTopologyStorage::Observed(topology.observe()),
                location: location.clone(),
                orient: orientation,
            };
            project(&shape)
        })
    }
}
