//! Wire splitting uses the production pipeline and real midpoint criteria.
use super::edge_divide::criteria;
use super::{
    brep_lib_common::shape,
    common::{Reader, attempt, end},
    healing_common::flags,
    shapes::tp_shapes,
    wire_data::{sh_edge, sh_face},
};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_upgrade::{
        ShapeUpgradeSplitCurve2d, ShapeUpgradeSplitCurve3d, ShapeUpgradeWireDivide, WireDivider,
    },
};
/// Compare repeated wire division; `wire_divide_case(&request)`.
pub fn wire_divide_case(values: &[f64]) {
    attempt(|| wire_request(&mut Reader::new(values)));
    end();
}
fn wire_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let divider = ShapeUpgradeWireDivide::new();
    configure_tools(&divider, reader);
    let context = Handle::new(ShapeBuildReShape::new());
    divider.set_context(&context);
    let edge = sh_edge(&shapes, reader.integer())?;
    let face = sh_face(&shapes, reader.integer())?;
    divider.load_edge(&edge)?;
    divider.set_face(&face);
    configure_modes(&divider, reader);
    for _ in 0..reader.integer() {
        attempt(|| divider.perform());
        flags(|status| divider.status_flag(status));
        shape(&divider.wire())?;
        shape(&edge)?;
        shape(&context.apply(&edge, ShapeEnum::Shape)?)?;
    }
    Ok(())
}
pub(super) fn configure_tools(
    divider: &ShapeUpgradeWireDivide,
    reader: &mut Reader<'_>,
) -> (bool, bool) {
    let spatial = match reader.integer() {
        -1 => Handle::default(),
        0 => Handle::new(ShapeUpgradeSplitCurve3d::new()),
        _ => Handle::new(criteria::Midpoint3d::default()).cast(),
    };
    let planar = match reader.integer() {
        -1 => Handle::default(),
        0 => Handle::new(ShapeUpgradeSplitCurve2d::new()),
        _ => Handle::new(criteria::Midpoint2d::default()).cast(),
    };
    divider.set_split_curve_3d_tool(&spatial);
    divider.set_split_curve_2d_tool(&planar);
    (
        divider.split_curve_3d_tool() == spatial,
        divider.split_curve_2d_tool() == planar,
    )
}

fn configure_modes(divider: &ShapeUpgradeWireDivide, reader: &mut Reader<'_>) {
    divider.set_edge_mode(reader.integer());
    divider.set_min_tolerance(reader.real());
    if !reader.flag() {
        divider.set_context(&Handle::default());
    }
}
