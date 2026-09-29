//! BRepGraph algorithms oracle, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_algo.cxx: the `br_bedit` actions of
//! BRepGraph_Validate and BRepGraph_Transform and the raw field corruptions written
//! through a mutation guard's `Internal()`. Every action reads its arguments in the
//! native order, so a raised exception leaves the reader where the native probe does.
use super::brep::{direction, point};
use super::brep_graph::node;
use super::brep_graph_edit::id;
use super::brep_graph_layers::read_node;
use super::brep_graph_layers_report::ascii;
use super::common::{Reader, flag, int, word};
use rustycad_tkbrep::brep_graph::transform;
use rustycad_tkbrep::brep_graph::validate::{self, Mode, Options, Severity, ValidationResult};
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Axis2, Transform3, Vector3};

/// `ba_trsf`: 0 translation, 1 rotation, 2 scale, 3 point mirror, 4 axis mirror, 5 plane
/// mirror, 6 `SetValues`.
pub fn trsf(r: &mut Reader<'_>) -> Result<Transform3, Failure> {
    let mut t = Transform3::default();
    match r.integer() {
        0 => t.set_translation(Vector3::from(point(r).xyz())),
        1 => {
            let p = point(r);
            let d = direction(r)?;
            t.set_rotation(Axis1::new(p, d), r.real())?;
        }
        2 => {
            let p = point(r);
            t.set_scale(p, r.real())?;
        }
        3 => t.set_mirror_point(point(r)),
        4 => {
            let p = point(r);
            t.set_mirror_axis(Axis1::new(p, direction(r)?));
        }
        5 => {
            let (p, n) = (point(r), direction(r)?);
            t.set_mirror_plane(Axis2::with_x_direction(p, n, direction(r)?)?);
        }
        _ => set_values(&mut t, r)?,
    }
    Ok(t)
}

fn set_values(t: &mut Transform3, r: &mut Reader<'_>) -> Result<(), Failure> {
    let mut rows = [[0.0; 4]; 3];
    for row in &mut rows {
        for value in row.iter_mut() {
            *value = r.real();
        }
    }
    t.set_values(rows)
}

/// `ba_report`: IsValid, the warning and error counts, then each issue.
fn report(v: &ValidationResult) {
    flag(v.is_valid());
    int(v.nb_issues(Severity::Warning));
    int(v.nb_issues(Severity::Error));
    int(v.issues.len() as i32);
    for issue in &v.issues {
        int(issue.sev as i32);
        node(issue.node_id);
        ascii(&issue.description);
    }
}

/// `ba_validate`: the five `Perform` overloads.
fn run_validate(g: &BRepGraph, how: i32) -> ValidationResult {
    match how {
        0 => validate::perform(g),
        1 => validate::perform_with_mode(g, Mode::Lightweight),
        2 => validate::perform_with_mode(g, Mode::Audit),
        3 => validate::perform_with_options(g, &Options::lightweight()),
        _ => validate::perform_with_options(g, &Options::audit()),
    }
}

// `ba_edge_field`.
fn edge_field(g: &mut BRepGraph, r: &mut Reader<'_>) -> Result<(), Failure> {
    let e: EdgeId = id(r);
    let (field, value) = (r.integer(), r.integer() as u32);
    let mut m = g.editor().edges().mutate(e)?;
    let d = m.internal();
    match field {
        0 => d.start_vertex_ref_id = VertexRefId::new(value),
        1 => d.end_vertex_ref_id = VertexRefId::new(value),
        2 => d.curve3d_rep_id = EdgeCurve3DRepId::new(value),
        _ => d.polygon3d_rep_id = EdgePolygon3DRepId::new(value),
    }
    Ok(())
}

// `ba_co_edge_field`.
fn co_edge_field(g: &mut BRepGraph, r: &mut Reader<'_>) -> Result<(), Failure> {
    let c: CoEdgeId = id(r);
    let (field, value) = (r.integer(), r.integer() as u32);
    let mut m = g.editor().co_edges().mutate(c)?;
    let d = m.internal();
    match field {
        0 => d.child_edge_id = EdgeId::new(value),
        1 => d.face_id = FaceId::new(value),
        2 => d.curve2d_rep_id = CoEdgeCurve2DRepId::new(value),
        3 => d.polygon2d_rep_id = CoEdgePolygon2DRepId::new(value),
        4 => d.polygon_on_tri_rep_id = CoEdgePolygonOnTriRepId::new(value),
        _ => d.parent_wire_id = WireId::new(value),
    }
    Ok(())
}

// `ba_face_field`.
fn face_field(g: &mut BRepGraph, r: &mut Reader<'_>) -> Result<(), Failure> {
    let x: FaceId = id(r);
    let (field, value) = (r.integer(), r.integer() as u32);
    let mut m = g.editor().faces().mutate(x)?;
    let d = m.internal();
    if field == 0 {
        d.surface_rep_id = FaceSurfaceRepId::new(value);
    } else {
        d.triangulation_rep_id = FaceTriangulationRepId::new(value);
    }
    Ok(())
}

// 247-249: the occurrence, vertex reference and child reference retargets.
fn child_field(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    match code {
        247 => {
            let o: OccurrenceId = id(r);
            let n = read_node(r);
            g.editor().occurrences().mutate(o)?.internal().child_node_id = n;
        }
        248 => {
            let (x, v): (VertexRefId, VertexId) = (id(r), id(r));
            g.editor()
                .vertices()
                .mutate_ref(x)?
                .internal()
                .child_vertex_id = v;
        }
        _ => {
            let x: ChildRefId = id(r);
            let n = read_node(r);
            g.editor()
                .generic()
                .mutate_child_ref(x)?
                .internal()
                .child_node_id = n;
        }
    }
    Ok(())
}

/// 244-249: raw writes through `Internal()` (`ba_corrupt_action`).
fn corrupt_action(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    match code {
        244 => edge_field(g, r)?,
        245 => co_edge_field(g, r)?,
        246 => face_field(g, r)?,
        247..=249 => child_field(g, r, code)?,
        _ => return Ok(false),
    }
    word("-");
    Ok(true)
}

/// 244-252 (`ba_algo_action`): the corruptions, 250 Validate, 251 and 252 MoveRef.
pub fn algo_action(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if corrupt_action(g, r, code)? {
        return Ok(true);
    }
    match code {
        250 => report(&run_validate(g, r.integer())),
        251 => {
            let x: ChildRefId = id(r);
            let t = trsf(r)?;
            flag(transform::move_child_ref(g, x, &t)?);
        }
        252 => {
            let x: OccurrenceRefId = id(r);
            let t = trsf(r)?;
            flag(transform::move_occurrence_ref(g, x, &t)?);
        }
        _ => return Ok(false),
    }
    Ok(true)
}
