use super::super::ShapeUpgradeShapeConvertToBezier as BezierConverter;
use crate::shape_extend::ShapeExtendWireData;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::Failure;

mod pcurves;
mod spatial;

pub(super) fn process_wire_edges(
    tool: &BezierConverter,
    face: &Face,
    wire_data: &ShapeExtendWireData,
) -> Result<(), Failure> {
    let count = wire_data.nb_edges();
    for index in 1..=count {
        let edge = wire_data.edge(index)?;
        let next = wire_data.edge(next_index(index, count))?;
        spatial::process_edge(tool, &edge)?;
        pcurves::process_edge_pair(face, &edge, &next)?;
    }
    Ok(())
}

fn next_index(index: i32, count: i32) -> i32 {
    if index == count { 1 } else { index + 1 }
}
