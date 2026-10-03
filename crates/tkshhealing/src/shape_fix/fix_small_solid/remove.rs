use super::ShapeFixFixSmallSolid;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn remove(
    fixer: &ShapeFixFixSmallSolid,
    shape: &Shape,
    context: &Handle<ShapeBuildReShape>,
) -> Result<Shape, Failure> {
    let config = fixer.config();
    if !config.has_threshold() || !valid_input(shape)? {
        return Ok(shape.clone());
    }
    remove_small_solids(fixer, shape, context, &config)?;
    live_context(context)?.apply(shape, ShapeEnum::Shape)
}

fn remove_small_solids(
    fixer: &ShapeFixFixSmallSolid,
    shape: &Shape,
    context: &Handle<ShapeBuildReShape>,
    config: &super::SmallSolidConfig,
) -> Result<(), Failure> {
    let mut solids = Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)?;
    while solids.more() {
        remove_current(fixer, &solids.current().clone(), context, config)?;
        solids.next()?;
    }
    Ok(())
}

fn remove_current(
    fixer: &ShapeFixFixSmallSolid,
    solid: &Shape,
    context: &Handle<ShapeBuildReShape>,
    config: &super::SmallSolidConfig,
) -> Result<(), Failure> {
    if !config.is_small(solid)? {
        return Ok(());
    }
    live_context(context)?.remove(solid)?;
    let key = AsciiString::from("ShapeFix.FixSmallSolid.MSG0");
    fixer.send_warning(solid, &Msg::from_key(&key));
    Ok(())
}

fn live_context(context: &Handle<ShapeBuildReShape>) -> Result<&ShapeBuildReShape, Failure> {
    context.get().ok_or_else(|| {
        Failure::new("ShapeFix_FixSmallSolid requires a non-null ShapeBuild_ReShape context")
    })
}

pub(super) fn valid_input(shape: &Shape) -> Result<bool, Failure> {
    if shape.is_null() {
        return Ok(false);
    }
    Ok(matches!(
        shape.shape_type()?,
        ShapeEnum::Compound | ShapeEnum::CompSolid | ShapeEnum::Solid
    ))
}
