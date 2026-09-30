//! Public transfer operations with native geometry and range outputs.
use super::{
    common::{Reader, attempt, end, flag, int, real},
    healing_common::{curve2d, curve3d},
    shapes::{TpShapes, tp_shapes},
    wire_data::{sh_edge, sh_face},
};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::{
    ncollection::HSequence,
    standard::{Failure, Handle},
};
use rustycad_tkshhealing::{
    shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisTransferParameters, TransferParameters},
    shape_build::ShapeBuildEdge,
};
/// Emit a transfer history; `transfer_parameters_case(&request)`.
pub fn transfer_parameters_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| transfer_request(&mut reader));
    end();
}
fn transfer_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let edge = sh_edge(&shapes, reader.integer())?;
    let face = sh_face(&shapes, reader.integer())?;
    let tool = if reader.flag() {
        ShapeAnalysisTransferParameters::from_edge(&edge, &face)?
    } else {
        let tool = ShapeAnalysisTransferParameters::new();
        flag(tool.is_same_range());
        tool.init(&edge, &face)?;
        tool
    };
    flag(tool.is_same_range());
    for _ in 0..reader.integer() {
        attempt(|| transfer_operation(&tool, &shapes, &edge, &face, reader));
        flag(tool.is_same_range());
    }
    Ok(())
}
fn transfer_operation(
    tool: &ShapeAnalysisTransferParameters,
    shapes: &TpShapes,
    edge: &Edge,
    face: &Face,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    match reader.integer() {
        0 => {
            let parameter = reader.real();
            real(tool.perform_parameter(parameter, reader.flag())?);
        }
        1 => transfer_sequence(tool, reader)?,
        2 => transfer_edge_range(tool, edge, face, reader)?,
        3 => tool.init(
            &sh_edge(shapes, reader.integer())?,
            &sh_face(shapes, reader.integer())?,
        )?,
        _ => tool.set_max_tolerance(reader.real()),
    }
    Ok(())
}
fn transfer_sequence(
    tool: &ShapeAnalysisTransferParameters,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    let to_2d = reader.flag();
    let parameters = Handle::new(HSequence::<f64>::default());
    for _ in 0..reader.integer() {
        let value = reader.real();
        parameters.change_sequence(|s| {
            s.append(value);
        });
    }
    let result = tool.perform(&parameters, to_2d)?;
    flag(result == parameters);
    let values = result.sequence(|s| s.iter().copied().collect::<Vec<_>>());
    int(values.len() as i32);
    values.into_iter().for_each(real);
    Ok(())
}
fn transfer_edge_range(
    tool: &ShapeAnalysisTransferParameters,
    edge: &Edge,
    face: &Face,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    let previous = reader.real();
    let current = reader.real();
    let is_2d = reader.flag();
    let target = ShapeBuildEdge.copy(edge, true)?;
    attempt(|| tool.transfer_range(&target, previous, current, is_2d));
    transfer_geometry(&target, face)?;
    transfer_geometry(edge, face)
}
fn transfer_geometry(edge: &Edge, face: &Face) -> Result<(), Failure> {
    let (curve, first, last) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
    curve3d(&curve, first, last);
    if !face.is_null() {
        let (curve, first, last) = ShapeAnalysisEdge::new().pcurve_on_face(edge, face, false)?;
        curve2d(&curve, first, last);
    }
    flag(BRepTool::same_range(edge)?);
    flag(BRepTool::same_parameter(edge)?);
    Ok(())
}
