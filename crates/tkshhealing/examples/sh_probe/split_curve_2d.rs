//! Native 2D curve-split geometry, shared arrays and status histories.
use super::{
    brep_lib_common::geom2d_curve, sh_common::curve2d as curve_digest,
    shape_construct::construct_spline_2d as spline_digest,
};
use rustycad_tkshhealing::shape_upgrade::{
    ShapeUpgradeSplitCurve2d as ProbeSplitter, SplitCurves2d as ProbeCurves,
};
include!("split_curve_geometry.inc.rs");
pub use curve_geometry_case as split_curve_2d_case;

fn read_curve(
    reader: &mut Reader<'_>,
) -> Result<Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>, Failure> {
    let offset = reader.flag();
    let distance = reader.real();
    let basis = geom2d_curve(reader)?;
    if !offset {
        return Ok(basis);
    }
    Ok(Handle::new(rustycad_tkg2d::geom2d::Geom2dOffsetCurve::new(
        &basis, distance, false,
    )?)
    .cast())
}
