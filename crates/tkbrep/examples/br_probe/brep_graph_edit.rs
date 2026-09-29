//! BRepGraph editor oracle, part 1, the Rust twin of
//! tools/oracle/tkbrep_probe_brepgraph_edit.cxx: the readers and the creation, container,
//! edge, vertex and coedge actions of `br_bedit`. Every action reads its arguments in the
//! native order, so a raised exception leaves the reader where the native probe leaves it.
use super::brep::{Scene, at, point};
use super::brep_graph::{index, list};
use super::brep_graph_layers::{read_list, read_node};
use super::common::{Reader, flag, word};
use super::graph::orientation;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, ChildRefId, CoEdgeId, CompSolidId, CompoundId, EdgeId, FaceId, FaceRefId, KindTag,
    OccurrenceId, OccurrenceRefId, ProductId, ShellId, ShellRefId, SolidId, SolidRefId, TypedId,
    VertexId, VertexRefId, WireId,
};
use rustycad_tkbrep::brep_graph_inc::ParityOrientation;
use rustycad_tkernel::standard::Failure;

/// `be_id`: a typed id as `index` (-1 is the invalid id).
pub fn id<T: KindTag>(r: &mut Reader<'_>) -> TypedId<T> {
    TypedId::new(r.integer() as u32)
}
/// `be_parity`: a TopAbs value as a parity; INTERNAL and EXTERNAL raise
/// Standard_ProgramError.
pub fn parity_of(o: i32) -> Result<ParityOrientation, Failure> {
    ParityOrientation::new(orientation(o))
}
/// `be_o`: a TopAbs value read as a parity.
pub fn parity(r: &mut Reader<'_>) -> Result<ParityOrientation, Failure> {
    parity_of(r.integer())
}
/// `be_ids`: a count, then the ids.
pub fn ids<T: KindTag>(r: &mut Reader<'_>) -> Vec<TypedId<T>> {
    read_list(r, id::<T>)
}
/// `be_os`: a count, then the orientations; reading stops at the first raise.
pub fn parities(r: &mut Reader<'_>) -> Result<Vec<ParityOrientation>, Failure> {
    let count = r.integer();
    (0..count).map(|_| parity(r)).collect()
}

/// 120-134: node creation (`be_add_action`).
pub fn add_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    if (120..=125).contains(&code) {
        return add_topology(g, s, r, code);
    }
    add_container(g, s, r, code)
}

fn add_topology(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    let geometry = &s.geometry;
    match code {
        120 => {
            let p = point(r);
            index(g.editor().vertices().add(p, r.real()));
        }
        121 => {
            let (a, b) = (id::<_>(r), id::<_>(r));
            let curve = at(&geometry.curves, r.integer());
            let (first, last) = (r.real(), r.real());
            index(g.editor().edges().add(a, b, &curve, first, last, r.real()));
        }
        122 => {
            let edge = id(r);
            index(g.editor().co_edges().add(edge, parity(r)?));
        }
        123 => {
            let (edge, face) = (id(r), id(r));
            let curve = at(&geometry.pcurves, r.integer());
            let (first, last) = (r.real(), r.real());
            let o = parity(r)?;
            index(
                g.editor()
                    .co_edges()
                    .add_with_pcurve(edge, face, &curve, first, last, o),
            );
        }
        124 => {
            let co_edges: Vec<CoEdgeId> = ids(r);
            index(g.editor().wires().add(&co_edges));
        }
        _ => {
            let surface = at(&geometry.surfaces, r.integer());
            let outer: WireId = id(r);
            let inner: Vec<WireId> = ids(r);
            index(g.editor().faces().add(&surface, outer, &inner, r.real()));
        }
    }
    Ok(true)
}

fn add_container(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    match code {
        126 => index(g.editor().shells().add()),
        127 => index(g.editor().solids().add()),
        128 => index(g.editor().compounds().add(&read_list(r, read_node))),
        129 => {
            let solids: Vec<SolidId> = ids(r);
            index(g.editor().comp_solids().add(&solids));
        }
        130 => {
            let root = read_node(r);
            index(g.editor().products().add(root, &s.graph.loc(r.integer())));
        }
        131 => index(g.editor().products().add_empty()),
        132 | 133 => append_products(g, s, r, code)?,
        134 => {
            g.editor().products().append_document_root(id(r))?;
            word("-");
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn append_products(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<(), Failure> {
    let parent: ProductId = id(r);
    if code == 133 {
        let children: Vec<ProductId> = ids(r);
        let locations = read_list(r, |x| s.graph.loc(x.integer()));
        list(
            &g.editor()
                .products()
                .append_many(parent, &children, &locations)?,
        );
        return Ok(());
    }
    let child: ProductId = id(r);
    let location = s.graph.loc(r.integer());
    let occurrence: OccurrenceId = id(r);
    let (made, reference) = g
        .editor()
        .products()
        .append(parent, child, &location, occurrence)?;
    index(made);
    index(reference);
    Ok(())
}

/// 135-155: container appends and removals (`be_container_action`).
pub fn container_action(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    if (135..=143).contains(&code) {
        return append_action(g, r, code);
    }
    remove_action(g, r, code)
}

fn append_action(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let e = g.editor();
    match code {
        135 => {
            let (face, wire): (FaceId, WireId) = (id(r), id(r));
            index(e.faces().append(face, wire, parity(r)?)?);
        }
        136 => {
            let (shell, face): (ShellId, FaceId) = (id(r), id(r));
            index(e.shells().append(shell, face, parity(r)?)?);
        }
        137 => {
            let (shell, faces): (ShellId, Vec<FaceId>) = (id(r), ids(r));
            list(&e.shells().append_many(shell, &faces, &parities(r)?)?);
        }
        138 => {
            let (solid, shell): (SolidId, ShellId) = (id(r), id(r));
            index(e.solids().append(solid, shell, parity(r)?)?);
        }
        139 => {
            let (solid, shells): (SolidId, Vec<ShellId>) = (id(r), ids(r));
            list(&e.solids().append_many(solid, &shells, &parities(r)?)?);
        }
        140 => {
            let (compound, child): (CompoundId, _) = (id(r), read_node(r));
            index(e.compounds().append(compound, child, parity(r)?)?);
        }
        141 => {
            let (compound, children): (CompoundId, _) = (id(r), read_list(r, read_node));
            list(
                &e.compounds()
                    .append_many(compound, &children, &parities(r)?)?,
            );
        }
        142 => {
            let (comp_solid, solid): (CompSolidId, SolidId) = (id(r), id(r));
            index(e.comp_solids().append(comp_solid, solid, parity(r)?)?);
        }
        _ => {
            let (comp_solid, solids): (CompSolidId, Vec<SolidId>) = (id(r), ids(r));
            list(
                &e.comp_solids()
                    .append_many(comp_solid, &solids, &parities(r)?)?,
            );
        }
    }
    Ok(true)
}

fn remove_action(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<bool, Failure> {
    let e = g.editor();
    let removed = match code {
        144 => e.faces().remove_wire(id(r), id(r))?,
        145 => e.shells().remove_face(id(r), id(r))?,
        146 => {
            let (owner, refs): (_, Vec<FaceRefId>) = (id(r), ids(r));
            e.shells().remove_faces(owner, &refs)?
        }
        147 => e.solids().remove_shell(id(r), id(r))?,
        148 => {
            let (owner, refs): (_, Vec<ShellRefId>) = (id(r), ids(r));
            e.solids().remove_shells(owner, &refs)?
        }
        149 => e.compounds().remove_child(id(r), id(r))?,
        150 => {
            let (owner, refs): (_, Vec<ChildRefId>) = (id(r), ids(r));
            e.compounds().remove_children(owner, &refs)?
        }
        151 => e.comp_solids().remove_solid(id(r), id(r))?,
        152 => {
            let (owner, refs): (_, Vec<SolidRefId>) = (id(r), ids(r));
            e.comp_solids().remove_solids(owner, &refs)?
        }
        153 => e.products().remove_occurrence(id(r), id(r))?,
        154 => {
            let (owner, refs): (_, Vec<OccurrenceRefId>) = (id(r), ids(r));
            e.products().remove_occurrences(owner, &refs)?
        }
        155 => e.products().remove_shape_root(id(r))?,
        _ => return Ok(false),
    };
    flag(removed);
    Ok(true)
}

/// 156-171: edges and vertices (`be_edge_action`).
pub fn edge_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    match code {
        156..=158 => edge_topology(g, r, code)?,
        159..=167 => edge_setter(g, s, r, code)?,
        168..=171 => vertex_setter(g, r, code)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn edge_topology(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let edge: EdgeId = id(r);
    let mut e = g.editor().edges();
    match code {
        156 => {
            let vertex: VertexId = id(r);
            let (a, b) = e.split(edge, vertex, r.real())?;
            index(a);
            index(b);
        }
        157 => flag(e.remove_vertex(edge, id(r))?),
        _ => {
            let reference: VertexRefId = id(r);
            index(e.replace_vertex(edge, reference, id(r))?);
        }
    }
    Ok(())
}

fn edge_setter(g: &mut BRepGraph, s: &Scene, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let edge: EdgeId = id(r);
    let mut e = g.editor().edges();
    match code {
        159 => e.reverse(edge),
        160 => e.set_tolerance(edge, r.real())?,
        161 => {
            let first = r.real();
            e.set_param_range(edge, first, r.real())?;
        }
        162 => {
            let curve = at(&s.geometry.curves, r.integer());
            let first = r.real();
            e.set_curve(edge, &curve, first, r.real())?;
        }
        163 => e.clear_curve(edge)?,
        164 => e.set_persistent_polygon3d(edge, &at(&s.geometry.polygons3d, r.integer()))?,
        165 => e.clear_persistent_polygon3d(edge)?,
        166 => e.set_start_vertex_ref_id(edge, id(r))?,
        _ => e.set_end_vertex_ref_id(edge, id(r))?,
    }
    word("-");
    Ok(())
}

fn vertex_setter(g: &mut BRepGraph, r: &mut Reader<'_>, code: i32) -> Result<(), Failure> {
    let mut v = g.editor().vertices();
    match code {
        168 => {
            let vertex: VertexId = id(r);
            v.set_point(vertex, point(r))?;
        }
        169 => {
            let vertex: VertexId = id(r);
            v.set_tolerance(vertex, r.real())?;
        }
        170 => {
            let reference: VertexRefId = id(r);
            v.set_ref_orientation(reference, parity(r)?)?;
        }
        _ => {
            let reference: VertexRefId = id(r);
            v.set_ref_child_vertex_id(reference, id(r))?;
        }
    }
    word("-");
    Ok(())
}

/// 172-181: coedges (`be_co_edge_action`).
pub fn co_edge_action(
    g: &mut BRepGraph,
    s: &Scene,
    r: &mut Reader<'_>,
    code: i32,
) -> Result<bool, Failure> {
    if !(172..=181).contains(&code) {
        return Ok(false);
    }
    let geometry = &s.geometry;
    let co_edge: CoEdgeId = id(r);
    let mut e = g.editor().co_edges();
    match code {
        172 => {
            let first = r.real();
            e.set_param_range(co_edge, first, r.real())?;
        }
        173 => e.set_orientation(co_edge, parity(r)?)?,
        174 => e.set_pcurve_keep_range(co_edge, &at(&geometry.pcurves, r.integer()))?,
        175 => {
            let curve = at(&geometry.pcurves, r.integer());
            let first = r.real();
            e.set_pcurve(co_edge, &curve, first, r.real())?;
        }
        176 => e.clear_pcurve(co_edge)?,
        177 => e.set_persistent_polygon2d(co_edge, &at(&geometry.polygons2d, r.integer()))?,
        178 => e.set_persistent_polygon_on_tri(co_edge, &at(&geometry.nodes, r.integer()))?,
        179 => e.reset_pcurve_binding(co_edge)?,
        180 => e.set_child_edge_id(co_edge, id(r))?,
        _ => e.set_face_id(co_edge, id(r))?,
    }
    word("-");
    Ok(true)
}
