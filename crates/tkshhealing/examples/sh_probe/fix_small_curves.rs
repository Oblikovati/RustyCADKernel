//! Base small-curve tool preserves geometry, parameter bits and status across Init/Approx.
use super::{
    common::{Reader, attempt, end, flag, int, real},
    healing_common::flags,
    sh_common::{curve, curve2d},
    shapes::{TpShapes, tp_same_index, tp_shapes},
    wire_data::{sh_edge, sh_face},
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkshhealing::shape_upgrade::{
    FixSmallCurves, ShapeUpgradeFixSmallCurves, ShapeUpgradeSplitCurve2d, ShapeUpgradeSplitCurve3d,
};
struct SmallCurveOutputs {
    spatial: Handle<dyn GeomCurve>,
    planar: Handle<dyn Geom2dCurve>,
    reversed: Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}
fn small_outputs(shapes: &TpShapes, reader: &mut Reader<'_>) -> SmallCurveOutputs {
    SmallCurveOutputs {
        spatial: nullable_geometry(&shapes.curves, reader.integer()),
        planar: nullable_geometry(&shapes.pcurves, reader.integer()),
        reversed: nullable_geometry(&shapes.pcurves, reader.integer()),
        first: reader.real(),
        last: reader.real(),
    }
}
fn nullable_geometry<T: ?Sized + RuntimeTyped>(curves: &[Handle<T>], index: i32) -> Handle<T> {
    if index < 0 {
        return Handle::default();
    }
    curves[index as usize].clone()
}
/// Emit unchanged approximation outputs, e.g. `fix_small_curves_case(&request)`.
pub fn fix_small_curves_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| small_curve_request(&mut reader));
    end();
}
fn small_curve_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let tool = ShapeUpgradeFixSmallCurves::new();
    let mut outputs = small_outputs(&shapes, reader);
    let originals = (
        outputs.spatial.clone(),
        outputs.planar.clone(),
        outputs.reversed.clone(),
    );
    small_state(&tool, &shapes, &outputs);
    for _ in 0..reader.integer() {
        attempt(|| small_operation(&tool, &shapes, &mut outputs, reader));
        flag(outputs.spatial == originals.0);
        flag(outputs.planar == originals.1);
        flag(outputs.reversed == originals.2);
        small_state(&tool, &shapes, &outputs);
    }
    Ok(())
}
fn small_operation(
    tool: &ShapeUpgradeFixSmallCurves,
    shapes: &TpShapes,
    outputs: &mut SmallCurveOutputs,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    match reader.integer() {
        0 => tool.init(
            &sh_edge(shapes, reader.integer())?,
            &sh_face(shapes, reader.integer())?,
        ),
        1 => set_small_tools(tool, reader),
        2 => flag(tool.approx(
            &mut outputs.spatial,
            &mut outputs.planar,
            &mut outputs.reversed,
            &mut outputs.first,
            &mut outputs.last,
        )?),
        _ => {
            let status = reader.integer();
            tool.fix_small_curves_data().with_mut(|f| f.status = status);
        }
    }
    Ok(())
}
fn set_small_tools(tool: &ShapeUpgradeFixSmallCurves, reader: &mut Reader<'_>) {
    let spatial = if reader.flag() {
        Handle::new(ShapeUpgradeSplitCurve3d::new())
    } else {
        Handle::default()
    };
    let planar = if reader.flag() {
        Handle::new(ShapeUpgradeSplitCurve2d::new())
    } else {
        Handle::default()
    };
    tool.set_split_curve_3d_tool(&spatial);
    tool.set_split_curve_2d_tool(&planar);
    flag(tool.split_curve_3d_tool() == spatial);
    flag(tool.split_curve_2d_tool() == planar);
}
fn small_state(tool: &ShapeUpgradeFixSmallCurves, shapes: &TpShapes, outputs: &SmallCurveOutputs) {
    tool.fix_small_curves_data().with(|f| {
        int(tp_same_index(shapes, &f.edge));
        int(tp_same_index(shapes, &f.face));
    });
    flag(tool.split_curve_3d_tool().is_null());
    flag(tool.split_curve_2d_tool().is_null());
    flags(|status| tool.status_flag(status));
    real(outputs.first);
    real(outputs.last);
    curve(&outputs.spatial);
    curve2d(&outputs.planar);
    curve2d(&outputs.reversed);
}
