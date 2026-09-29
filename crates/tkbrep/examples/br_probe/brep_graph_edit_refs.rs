//! BRepGraph editor oracle, part 2, the Rust twin of the wire, face, reference and
//! generic actions of tools/oracle/tkbrep_probe_brepgraph_edit_rest.cxx.
use super::brep::{Scene, at};
use super::brep_graph_edit::{id, ids, parity};
use super::brep_graph_layers::{read_node, read_ref};
use super::common::{Reader, flag, int, word};
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ChildRefId, CoEdgeId, EdgeId, FaceId, FaceRefId, OccurrenceRefId, ShellRefId,
    SolidRefId, WireId, WireRefId,
};
use rustycad_tkernel::standard::Failure;

/// 182-198: wires and faces (`be_wire_face_action`).
pub fn wire_face_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    match code {
        182..=185 | 187 | 189 => wire_query(g, r, code)?,
        186 | 188 | 190 | 191 => wire_edit(g, r, code)?,
        192..=198 => face_edit(g, s, r, code)?,
        _ => return Ok(false),
    }
    Ok(true)
}

// The checks, RemoveCoEdge and SetCoEdgeOrder print their answer instead of `-`.
fn wire_query(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let mut w = g.editor().wires();
    match code {
        182 => {
            let co_edges: Vec<CoEdgeId> = ids(r);
            int(w.check_co_edge_order(&co_edges) as i32);
        }
        183 => {
            let (wire, co_edges): (WireId, Vec<CoEdgeId>) = (id(r), ids(r));
            int(w.check_wire_co_edge_order(wire, &co_edges) as i32);
        }
        184 => int(w.check_append_co_edge(id(r), id(r)) as i32),
        185 => {
            let (wire, old, new): (WireId, EdgeId, EdgeId) = (id(r), id(r), id(r));
            int(w.check_replace_edge(wire, old, new, r.flag()) as i32);
        }
        187 => {
            let wire: WireId = id(r);
            flag(w.remove_co_edge(wire, id(r))?);
        }
        _ => {
            let (wire, co_edges): (WireId, Vec<CoEdgeId>) = (id(r), ids(r));
            flag(w.set_co_edge_order(wire, &co_edges));
        }
    }
    Ok(())
}

fn wire_edit(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let mut w = g.editor().wires();
    match code {
        186 => {
            let (wire, old, new): (WireId, EdgeId, EdgeId) = (id(r), id(r), id(r));
            w.replace_edge(wire, old, new, r.flag())?;
        }
        188 => w.reverse(id(r)),
        190 => {
            let reference: WireRefId = id(r);
            w.set_ref_orientation(reference, parity(r)?)?;
        }
        _ => {
            let reference: WireRefId = id(r);
            w.set_ref_child_wire_id(reference, id(r))?;
        }
    }
    word("-");
    Ok(())
}

fn face_edit(g: &mut BRepGraph, s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let geometry = &s.geometry;
    let mut f = g.editor().faces();
    match code {
        192 => {
            let face: FaceId = id(r);
            f.set_tolerance(face, r.real())?;
        }
        193 => {
            let face: FaceId = id(r);
            f.set_surface(face, &at(&geometry.surfaces, r.integer()))?;
        }
        194 => f.clear_surface(id(r))?,
        195 => {
            let face: FaceId = id(r);
            f.set_persistent_triangulation(face, &at(&geometry.triangulations, r.integer()))?;
        }
        196 => f.clear_persistent_triangulation(id(r))?,
        197 => {
            let reference: FaceRefId = id(r);
            f.set_ref_orientation(reference, parity(r)?)?;
        }
        _ => {
            let reference: FaceRefId = id(r);
            f.set_ref_face_id(reference, id(r))?;
        }
    }
    word("-");
    Ok(())
}

/// 199-216: references and generic operations (`be_ref_action`).
pub fn ref_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    match code {
        199..=207 => container_ref(g, s, r, code)?,
        208..=216 => generic(g, s, r, code)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn container_ref(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<(), Failure> {
    let e = g.editor();
    match code {
        199 => {
            let reference: ShellRefId = id(r);
            e.shells().set_ref_orientation(reference, parity(r)?)?;
        }
        200 => e.shells().set_ref_child_shell_id(id(r), id(r))?,
        201 => {
            let reference: SolidRefId = id(r);
            e.solids().set_ref_orientation(reference, parity(r)?)?;
        }
        202 => e.solids().set_ref_child_solid_id(id(r), id(r))?,
        203 => {
            let reference: ChildRefId = id(r);
            e.compounds().replace_child(reference, read_node(r))?;
        }
        204 => e.comp_solids().replace_solid(id(r), id(r))?,
        205 => {
            let reference: OccurrenceRefId = id(r);
            e.occurrences()
                .set_ref_local_location(reference, &s.graph.loc(r.integer()))?;
        }
        206 => e.occurrences().set_child_node_id(id(r), read_node(r))?,
        _ => e.occurrences().set_ref_child_occurrence_id(id(r), id(r))?,
    }
    word("-");
    Ok(())
}

fn generic(g: &mut BRepGraph, s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let mut x = g.editor().generic();
    match code {
        208 => x.remove_node(read_node(r))?,
        209 => x.replace_node(read_node(r), read_node(r))?,
        210 => x.remove_subgraph(read_node(r))?,
        211 => {
            flag(x.remove_ref(read_ref(r))?);
            return Ok(());
        }
        212 => {
            let (parent, reference) = (read_node(r), read_ref(r));
            flag(x.remove_ref_of_parent(parent, reference, r.flag())?);
            return Ok(());
        }
        213 => {
            let reference: ChildRefId = id(r);
            x.set_child_ref_orientation(reference, parity(r)?)?;
        }
        214 => x.set_child_ref_child_node_id(id(r), read_node(r))?,
        215 => {
            let reference: ChildRefId = id(r);
            x.set_child_ref_local_location(reference, &s.graph.loc(r.integer()))?;
        }
        _ => x.cleanup_removed_references(),
    }
    word("-");
    Ok(())
}
