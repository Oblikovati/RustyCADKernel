//! Native out-parameter preservation, including nonzero initial values.
use super::common::{Reader, attempt, end, flag, pnt2d, real};
use super::healing_common::{flags, vec2d};
use super::shapes::{TpShapes, tp_shapes};
use super::wire_data::{sh_edge, sh_face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tkshhealing::shape_analysis::{EdgeTangent2d, ShapeAnalysisEdge};

fn tangent_face(
    r: &mut Reader<'_>,
    scene: &TpShapes,
    tool: &ShapeAnalysisEdge,
) -> Result<(), Failure> {
    let edge = sh_edge(scene, r.integer())?;
    let face = sh_face(scene, r.integer())?;
    let (at_end, fraction) = (r.flag(), r.real());
    let mut outputs = read_tangent_outputs(r);
    flag(tool.get_end_tangent_2d_on_face_into(&edge, &face, at_end, fraction, &mut outputs)?);
    pnt2d(outputs.point);
    vec2d(outputs.tangent);
    Ok(())
}

fn tangent_surface(
    r: &mut Reader<'_>,
    scene: &TpShapes,
    tool: &ShapeAnalysisEdge,
) -> Result<(), Failure> {
    let edge = sh_edge(scene, r.integer())?;
    let surface = &scene.surfaces[r.integer() as usize];
    let location = &scene.locations[r.integer() as usize];
    let (at_end, fraction) = (r.flag(), r.real());
    let mut outputs = read_tangent_outputs(r);
    flag(tool.get_end_tangent_2d_on_surface_into(
        &edge,
        surface,
        location,
        at_end,
        fraction,
        &mut outputs,
    )?);
    pnt2d(outputs.point);
    vec2d(outputs.tangent);
    Ok(())
}

fn read_tangent_outputs(r: &mut Reader<'_>) -> EdgeTangent2d {
    let point = Point2::new(r.real(), r.real());
    let tangent = Vector2::new(r.real(), r.real());
    EdgeTangent2d { point, tangent }
}

fn tolerance(
    r: &mut Reader<'_>,
    scene: &TpShapes,
    tool: &mut ShapeAnalysisEdge,
    op: i32,
) -> Result<(), Failure> {
    let edge = sh_edge(scene, r.integer())?;
    let face = if op == 2 {
        Some(sh_face(scene, r.integer())?)
    } else {
        None
    };
    let mut tolerances = (r.real(), r.real());
    let changed = match face {
        Some(face) => tool.check_vertex_tolerance_on_face_into(&edge, &face, &mut tolerances)?,
        None => tool.check_vertex_tolerance_into(&edge, &mut tolerances)?,
    };
    flag(changed);
    real(tolerances.0);
    real(tolerances.1);
    flags(|status| tool.status_flag(status));
    Ok(())
}

fn deviation(
    r: &mut Reader<'_>,
    scene: &TpShapes,
    tool: &mut ShapeAnalysisEdge,
    op: i32,
) -> Result<(), Failure> {
    let edge = sh_edge(scene, r.integer())?;
    let face = if op == 4 {
        Some(sh_face(scene, r.integer())?)
    } else {
        None
    };
    let samples = r.integer();
    let mut maximum = r.real();
    let changed = match face {
        Some(face) => {
            tool.check_same_parameter_on_face_into(&edge, &face, samples, &mut maximum)?
        }
        None => tool.check_same_parameter_into(&edge, samples, &mut maximum)?,
    };
    flag(changed);
    real(maximum);
    flags(|status| tool.status_flag(status));
    Ok(())
}

pub fn edgeoutputs_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let scene = tp_shapes(&mut reader).expect("probe shapes");
    let op = reader.integer();
    let mut tool = ShapeAnalysisEdge::new();
    match op {
        0 => attempt(|| tangent_face(&mut reader, &scene, &tool)),
        1 => attempt(|| tangent_surface(&mut reader, &scene, &tool)),
        2 | 3 => attempt(|| tolerance(&mut reader, &scene, &mut tool, op)),
        _ => attempt(|| deviation(&mut reader, &scene, &mut tool, op)),
    }
    end();
}
