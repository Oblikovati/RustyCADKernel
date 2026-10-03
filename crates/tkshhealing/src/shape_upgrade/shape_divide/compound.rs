//! Assembly recursion preserves native orientation and optional location removal (Bug21).
use super::ShapeDivider;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topods::{Builder, Compound, Shape, ShapeIterator},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum};
use rustycad_tkmath::toploc::Location;

pub(super) fn perform_compound(
    tool: &(impl ShapeDivider + ?Sized),
    source: &Shape,
) -> Result<bool, Failure> {
    let base = tool.shape_divide_base();
    let mut status = base.with(|f| f.status);
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    for child in ShapeIterator::new(source, false, true) {
        let result = divide_child(tool, child, source)?;
        Builder.add(&mut compound, &result)?;
        status |= base.with(|f| f.status);
    }
    base.with_mut(|f| {
        f.shape = source.clone();
        f.status = status;
    });
    finish_compound(base, source, compound.into_shape())
}
fn finish_compound(
    base: &super::ShapeUpgradeShapeDivide,
    source: &Shape,
    compound: Shape,
) -> Result<bool, Failure> {
    if !base.status_flag(ShapeExtendStatus::Done) {
        base.with_mut(|f| f.result = source.clone());
        return Ok(false);
    }
    let result = base.context().apply(&compound, ShapeEnum::Shape)?;
    base.with_mut(|f| f.result = result.clone());
    base.context().replace(source, &result)?;
    Ok(true)
}
fn divide_child(
    tool: &(impl ShapeDivider + ?Sized),
    mut child: Shape,
    source: &Shape,
) -> Result<Shape, Failure> {
    let base = tool.shape_divide_base();
    let location = child.location().clone();
    if base.context().mode_consider_location() {
        child.set_location(&Location::default(), false)?;
    }
    let applied = base.context().apply(&child, ShapeEnum::Shape)?;
    base.init(&applied);
    tool.perform(false)?;
    let mut result = base.result();
    if base.context().mode_consider_location() {
        result.set_location(&location, false)?;
    }
    result.set_orientation(top_abs::compose(result.orientation(), source.orientation()));
    base.with_mut(|f| f.result = result.clone());
    Ok(result)
}
