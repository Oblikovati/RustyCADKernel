// Rust translation of the static helper FindFromUEdge
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The first part of the new bound of a face at a corner: the section edges from the
//! cut end of the first u-edge.
use super::HistoryMap;
use super::ds_queries::{find_next_vertex, find_vertex, remove_edges};
use super::gap::{GapEnds, fill_gap, pcurve_on, pcurve_value};
use rustycad_tkbo::bop_ds::{DataStructure, Pave};
use rustycad_tkbrep::topods::{Compound, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

/// The fixed inputs of the searches of the new bound of a face.
pub(super) struct BoundSearch<'a> {
    /// The first u-edge before the trimming.
    pub(super) ue1_old: &'a Edge,
    /// The second u-edge before the trimming.
    pub(super) ue2_old: &'a Edge,
    /// The face (any orientation; it is used FORWARD).
    pub(super) face_f: Face,
    /// 0 for the first column of faces, 1 for the second.
    pub(super) rank: i32,
    /// The bound edge of the face.
    pub(super) bound_edge: &'a Edge,
    /// The index of the bound edge in the data structure.
    pub(super) bound_edge_index: i32,
    /// The data structure of the intersection.
    pub(super) ds: &'a DataStructure,
    /// The history of the trimming.
    pub(super) hist: &'a HistoryMap,
}

impl<'a> BoundSearch<'a> {
    /// The inputs with the face made FORWARD.
    pub(super) fn new(
        ue_old: (&'a Edge, &'a Edge),
        face: &Face,
        rank: i32,
        bound: (&'a Edge, i32),
        ds: &'a DataStructure,
        hist: &'a HistoryMap,
    ) -> Self {
        let face_f = Face::unchecked(face.oriented(Orientation::Forward));
        Self {
            ue1_old: ue_old.0,
            ue2_old: ue_old.1,
            face_f,
            rank,
            bound_edge: bound.0,
            bound_edge_index: bound.1,
            ds,
            hist,
        }
    }

    /// `FillGap` between two vertices of the face in the section edges.
    pub(super) fn fill(
        &self,
        (first, last): (&Vertex, &Vertex),
        (p1, p2): (Point2, Point2),
        sec_edges: &Compound,
    ) -> Result<Option<List<Shape>>, Failure> {
        let ends = GapEnds {
            first_vertex: first,
            last_vertex: last,
            first_point: p1,
            last_point: p2,
        };
        fill_gap(&ends, &self.face_f, sec_edges)
    }

    /// The point of the p-curve of an edge on the face at a parameter.
    pub(super) fn point_on(&self, e: &Edge, t: f64) -> Result<Point2, Failure> {
        let (c, _, _) = pcurve_on(e, &self.face_f)?;
        pcurve_value(&c, t)
    }
}

/// The outputs of FindFromUEdge.
pub(super) struct UEdgeFound {
    /// The section edges left (updated only when a chain is found).
    pub(super) sec_edges_new: Compound,
    /// The chain found (set only when one is found).
    pub(super) list_of_wire_edges: List<Shape>,
    /// The pave ending the chain (index 0 and parameter 0 when none is found).
    pub(super) found_pave: Pave,
    /// Whether the chain ends on the second u-edge (true when none is found).
    pub(super) is_on_u_edge: bool,
}

/// A chain of section edges taken out of the compound.
fn take_chain(
    sec_edges: &mut Compound,
    mut ordered: List<Shape>,
    list: &mut List<Shape>,
) -> Result<(), Failure> {
    *sec_edges = remove_edges(sec_edges, &ordered)?;
    list.append_list(&mut ordered);
    Ok(())
}

/// `FindFromUEdge(theUE1Old, theUE2Old, theUE1New, theUE2New, theFace, theSecEdges,
/// theRank, theBoundEdge, theBoundEdgeIndex, theDS, theHistMap, theSecEdgesNew,
/// theListOfWireEdges, theFoundPave, isOnUEdge)`: the chain of section edges from the
/// cut end of the first u-edge to a new vertex of the bound edge, else to the cut end
/// of the second u-edge; false when the cut end has no vertex.
pub(super) fn find_from_u_edge(
    search: &BoundSearch,
    (ue1_new, ue2_new): (&Edge, &Edge),
    the_sec_edges: &Compound,
    out: &mut UEdgeFound,
) -> Result<bool, Failure> {
    out.found_pave = Pave::new(0, 0.);
    out.is_on_u_edge = true;
    let mut atmp_pave = Pave::default();
    let Some(a_first_v) = find_vertex(
        search.ue1_old,
        search.rank,
        search.ds,
        search.hist,
        &mut atmp_pave,
    )?
    else {
        return Ok(true);
    };
    if a_first_v.is_null() {
        return Ok(false);
    }
    let (a_c1, f, l) = pcurve_on(ue1_new, &search.face_f)?;
    let p1: Point2 = pcurve_value(&a_c1, if search.rank == 0 { l } else { f })?;
    let mut a_comp_of_sec_edges: Compound = the_sec_edges.clone();
    let mut a_list_of_wire_edges: List<Shape> = List::default();
    let mut found: Option<(Pave, bool)> = None;
    let nbpave: i32 = search.ds.paves(search.bound_edge_index)?.len() as i32;
    let (mut a_pave1, mut pit) = (Pave::default(), 0);
    while let Some((a_next_vertex, a_pave2)) =
        find_next_vertex(search.bound_edge_index, &a_pave1, search.ds)?
    {
        if pit >= nbpave {
            break;
        }
        let p2: Point2 = search.point_on(search.bound_edge, a_pave2.parameter())?;
        if let Some(ordered) =
            search.fill((&a_first_v, &a_next_vertex), (p1, p2), &a_comp_of_sec_edges)?
        {
            take_chain(&mut a_comp_of_sec_edges, ordered, &mut a_list_of_wire_edges)?;
            found = Some((a_pave2, false));
            break;
        }
        a_pave1 = a_pave2;
        pit += 1;
    }
    if found.is_none() {
        let mut a_pave2 = Pave::default();
        if let Some(a_last_v) = find_vertex(
            search.ue2_old,
            search.rank,
            search.ds,
            search.hist,
            &mut a_pave2,
        )? {
            let p2: Point2 = search.point_on(ue2_new, a_pave2.parameter())?;
            if let Some(ordered) =
                search.fill((&a_first_v, &a_last_v), (p1, p2), &a_comp_of_sec_edges)?
            {
                take_chain(&mut a_comp_of_sec_edges, ordered, &mut a_list_of_wire_edges)?;
                found = Some((a_pave2, true));
            }
        }
    }
    if let Some((pave, on_u_edge)) = found {
        out.found_pave = pave;
        out.is_on_u_edge = on_u_edge;
        out.list_of_wire_edges = a_list_of_wire_edges;
        out.sec_edges_new = a_comp_of_sec_edges;
    }
    Ok(true)
}
