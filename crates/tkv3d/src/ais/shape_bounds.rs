//! Native AIS_Shape lazy bounds are topology-derived, not driver presentation bounds.
use super::shape::AisShapeSource;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

// Mode2 consumes a snapshot of the actual topology cache; the native public const-reference getter stays partial.
pub(super) fn shape_presentation_bounds_snapshot<S: AisShapeSource + ?Sized>(
    object: &S,
) -> Result<BoundingBox3, Failure> {
    let base = object.shape_base();
    let shape = base.fields.read().shape.clone();
    if shape.shape_type()? == ShapeEnum::Compound && shape.nb_children() == 0 {
        base.fields.write().bounds.set_void();
        return Ok(base.fields.read().bounds.clone());
    }
    if base.fields.read().recompute_bounds {
        recompute_shape_topology_bounds(object)?;
    }
    Ok(base.fields.read().bounds.clone())
}
fn recompute_shape_topology_bounds<S: AisShapeSource + ?Sized>(object: &S) -> Result<(), Failure> {
    let shape = object.shape_base().fields.read().shape.clone();
    let mut bounds = BoundingBox3::default();
    let result = BRepBndLib::add(&shape, &mut bounds, false);
    let mut fields = object.shape_base().fields.write();
    fields.bounds = bounds;
    if result.is_ok() {
        fields.recompute_bounds = false;
    }
    result
}
