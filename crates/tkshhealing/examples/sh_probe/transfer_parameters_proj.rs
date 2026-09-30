//! Projected parameter transfer histories share base geometry digests.
use super::{
    common::{Reader, attempt, end, flag, int, pnt, real},
    sh_common::{curve, curve2d, location, surface},
    shapes::{TpShapes, tp_shapes},
    wire_data::{sh_edge, sh_face, sh_shape},
};
use rustycad_tkbrep::{
    brep::{BRepPointRepresentation, BRepTool, access::vertex_tshape},
    topods::{self, Edge, Face, Vertex},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkshhealing::shape_analysis::{
    ShapeAnalysisTransferParametersProj, TransferParameters,
};
/// Emit projected transfer state; `transfer_parameters_proj_case(&request)`.
pub fn transfer_parameters_proj_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| projection_request(&mut reader));
    end();
}
fn projection_request(reader: &mut Reader<'_>) -> Result<(), Failure> {
    let shapes = tp_shapes(reader)?;
    let edge = sh_edge(&shapes, reader.integer())?;
    let face = sh_face(&shapes, reader.integer())?;
    let tool = create_tool(&edge, &face, reader.flag())?;
    flag(tool.is_same_range());
    flag(tool.force_projection());
    for _ in 0..reader.integer() {
        attempt(|| projection_operation(&tool, &shapes, &edge, &face, reader));
        flag(tool.is_same_range());
        flag(tool.force_projection());
    }
    Ok(())
}
fn create_tool(
    edge: &Edge,
    face: &Face,
    initialized: bool,
) -> Result<ShapeAnalysisTransferParametersProj, Failure> {
    if initialized {
        return ShapeAnalysisTransferParametersProj::from_edge(edge, face);
    }
    let tool = ShapeAnalysisTransferParametersProj::new();
    flag(tool.is_same_range());
    flag(tool.force_projection());
    tool.init(edge, face)?;
    Ok(tool)
}
fn projection_operation(
    tool: &ShapeAnalysisTransferParametersProj,
    shapes: &TpShapes,
    edge: &Edge,
    face: &Face,
    reader: &mut Reader<'_>,
) -> Result<(), Failure> {
    let operation = reader.integer();
    if operation == 5 {
        tool.set_force_projection(reader.flag());
        return Ok(());
    }
    if operation == 6 || operation == 7 {
        return copy_vertex(shapes, reader, operation == 7);
    }
    super::transfer_parameters::transfer_operation_code(operation, tool, shapes, edge, face, reader)
}
fn copy_vertex(shapes: &TpShapes, reader: &mut Reader<'_>, on_face: bool) -> Result<(), Failure> {
    let vertex = topods::vertex(&sh_shape(shapes, reader.integer()))?;
    let target = reader.integer();
    let source = reader.integer();
    let copy = if on_face {
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_face(
            &vertex,
            &sh_face(shapes, target)?,
            &sh_face(shapes, source)?,
        )?
    } else {
        ShapeAnalysisTransferParametersProj::copy_nm_vertex_edge(
            &vertex,
            &sh_edge(shapes, target)?,
            &sh_edge(shapes, source)?,
        )?
    };
    flag(copy.is_same(&vertex));
    vertex_digest(&copy)?;
    vertex_digest(&vertex)?;
    if !copy.is_null() && !on_face {
        real(BRepTool::parameter(&copy, &sh_edge(shapes, target)?)?);
    }
    Ok(())
}
fn vertex_digest(vertex: &Vertex) -> Result<(), Failure> {
    flag(vertex.is_null());
    if vertex.is_null() {
        return Ok(());
    }
    int(vertex.orientation() as i32);
    pnt(BRepTool::pnt(vertex)?);
    real(BRepTool::vertex_tolerance(vertex)?);
    let points = vertex_tshape(vertex, "projection probe")?.points();
    int(points.len() as i32);
    for point in points {
        representation_digest(&point)?;
    }
    Ok(())
}
fn representation_digest(point: &Handle<dyn BRepPointRepresentation>) -> Result<(), Failure> {
    flag(point.is_null());
    if point.is_null() {
        return Ok(());
    }
    flag(point.is_point_on_curve());
    flag(point.is_point_on_curve_on_surface());
    flag(point.is_point_on_surface());
    real(point.parameter());
    location(&point.location());
    if point.is_point_on_curve() {
        curve(&point.curve()?);
    }
    if point.is_point_on_curve_on_surface() {
        curve2d(&point.pcurve()?);
        surface(&point.surface()?);
    }
    if point.is_point_on_surface() {
        real(point.parameter2()?);
        surface(&point.surface()?);
    }
    Ok(())
}
