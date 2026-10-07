// Rust translation of BRepFill_TrimShellCorner::MakeFacesSec and the static helper
// GetUEdges (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of two faces at a corner when they intersect: each bound is replaced by
//! the section edges (and the splits of the bound) between the cut u-edges.
use super::BRepFillTrimShellCorner;
use super::ds_queries::{filter_section_edges, find_common_vertex_ds};
use super::from_u_edge::{BoundSearch, UEdgeFound, find_from_u_edge};
use super::from_v_edge::find_from_v_edge;
use super::gap::pcurve_value;
use crate::brep_fill::shape_table::{ShapeTable, cell};
use rustycad_tkbo::bop_ds::{DataStructure, Pave};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;

impl BRepFillTrimShellCorner {
    /// `MakeFacesSec(theIndex, theDS, theFaceIndex1, theFaceIndex2, theSSInterfIndex)`:
    /// the history of the two faces of row `the_index` cut by their section; false when
    /// a new bound cannot be built.
    pub(super) fn make_faces_sec(
        &mut self,
        the_index: i32,
        the_ds: &DataStructure,
        the_face_index1: i32,
        the_face_index2: i32,
        the_ss_interf_index: i32,
    ) -> Result<bool, Failure> {
        let a_ffi = the_ds
            .interf_ff_records()
            .value(the_ss_interf_index)
            .map_err(crate::brep_fill::shape_table::table_error)?;
        let mut a_sec_edges: Compound =
            filter_section_edges(a_ffi.curves(), &Face::default(), the_ds)?;
        let first_vertex: Vertex = self.u_edges_crossing(the_index, the_ds)?;
        let last_vertex: Vertex = self.u_edges_crossing(the_index + 1, the_ds)?;
        if let Some(choice) = self.choose_section(&a_sec_edges, &first_vertex, &last_vertex)? {
            let builder = BRepBuilder::default();
            let mut a_comp = Compound::default();
            builder.make_compound(&mut a_comp);
            for e in Explorer::new(&choice.wire, ShapeEnum::Edge, ShapeEnum::Shape)? {
                builder.add(&mut a_comp, &e)?;
            }
            a_sec_edges = a_comp;
            self.store_v_edge_in_hist_map(the_index, &choice.wire)?;
        }
        let a_substitutor = BRepToolsReShape::new();
        for (fit, face_index) in [(0, the_face_index1), (1, the_face_index2)] {
            if !self.trim_sec_face(
                the_index,
                the_ds,
                (fit, face_index),
                &a_sec_edges,
                &a_substitutor,
            )? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The common vertex of the two u-edges of a row (the last vertex examined, found
    /// or not, as natively: the found flag is not read).
    fn u_edges_crossing(&self, row: i32, the_ds: &DataStructure) -> Result<Vertex, Failure> {
        let e1 = the_ds.index(cell(&self.my_u_edges, row, 1)?);
        let e2 = the_ds.index(cell(&self.my_u_edges, row, 2)?);
        Ok(find_common_vertex_ds(the_ds, e1, e2, &self.my_int_point_cross_dir)?.vertex)
    }

    /// The u-edge replaced by its first new edge (with its orientation) in the
    /// substitutor.
    fn substituted_u_edge(
        &self,
        a_ue: &Edge,
        a_substitutor: &BRepToolsReShape,
    ) -> Result<Edge, Failure> {
        let Some(first) = super::ds_queries::first_new(&self.my_hist_map, a_ue) else {
            return Ok(a_ue.clone());
        };
        let an_edge: Shape = first.oriented(a_ue.orientation());
        if !a_ue.is_same(&an_edge) {
            a_substitutor.replace(
                &a_ue.oriented(Orientation::Forward),
                &an_edge.oriented(Orientation::Forward),
            )?;
        }
        topods::edge(&an_edge)
    }

    /// The face of column `fit` with its bound replaced by the wire of the section
    /// chains and splits; false when they cannot be found.
    fn trim_sec_face(
        &mut self,
        the_index: i32,
        the_ds: &DataStructure,
        (fit, face_index): (i32, i32),
        a_sec_edges: &Compound,
        a_substitutor: &BRepToolsReShape,
    ) -> Result<bool, Failure> {
        let a_face: Face = topods::face(the_ds.shape(face_index)?)?;
        let a_face_ori: Orientation = a_face.orientation();
        let a_face_f = Face::unchecked(a_face.oriented(Orientation::Forward));
        let a_bound_edge: Edge = topods::edge(cell(
            &self.my_bounds,
            the_index,
            self.my_bounds.lower_col() + fit,
        )?)?;
        let a_bound_edge_index: i32 = the_ds.index(&a_bound_edge);
        let Some((ue1_old, ue2_old)) =
            get_u_edges(the_index, fit, &self.my_u_edges, &a_bound_edge, &a_face_f)?
        else {
            return Ok(false);
        };
        let ue1 = self.substituted_u_edge(&ue1_old, a_substitutor)?;
        let ue2 = self.substituted_u_edge(&ue2_old, a_substitutor)?;
        let Some(a_list_of_wire_edges) = self.bound_wire_edges(
            the_ds,
            (&ue1_old, &ue2_old),
            (&ue1, &ue2),
            &a_face,
            fit,
            (&a_bound_edge, a_bound_edge_index),
            a_sec_edges,
        )?
        else {
            return Ok(false);
        };
        if !a_list_of_wire_edges.is_empty() {
            let builder = BRepBuilder::default();
            let mut a_w = Wire::default();
            builder.make_wire(&mut a_w);
            for e in a_list_of_wire_edges
                .iter()
                .filter(|e| !a_bound_edge.is_same(e))
            {
                builder.add(&mut a_w, e)?;
            }
            a_substitutor.replace(&a_bound_edge.oriented(Orientation::Forward), &a_w)?;
        }
        self.record_face_history(a_substitutor, &a_face, a_face_ori)?;
        Ok(true)
    }

    /// The edges of the new bound: the chain from the first u-edge, then the chains and
    /// splits along the bound edge; `None` when either search fails or nothing is
    /// found.
    #[allow(clippy::too_many_arguments)]
    fn bound_wire_edges(
        &self,
        the_ds: &DataStructure,
        old: (&Edge, &Edge),
        new: (&Edge, &Edge),
        a_face: &Face,
        fit: i32,
        bound: (&Edge, i32),
        a_sec_edges: &Compound,
    ) -> Result<Option<List<Shape>>, Failure> {
        let search = BoundSearch::new(old, a_face, fit, bound, the_ds, &self.my_hist_map);
        let mut found = UEdgeFound {
            sec_edges_new: a_sec_edges.clone(),
            list_of_wire_edges: List::default(),
            found_pave: Pave::default(),
            is_on_u_edge: true,
        };
        if !find_from_u_edge(&search, new, a_sec_edges, &mut found)? {
            return Ok(None);
        }
        let Some((mut a_second_list_of_edges, bis_section_found)) = find_from_v_edge(
            &search,
            &found.found_pave,
            found.is_on_u_edge,
            &found.sec_edges_new,
        )?
        else {
            return Ok(None);
        };
        if !bis_section_found && found.list_of_wire_edges.is_empty() {
            return Ok(None);
        }
        found
            .list_of_wire_edges
            .append_list(&mut a_second_list_of_edges);
        Ok(Some(found.list_of_wire_edges))
    }
}

/// `GetUEdges(theIndex, theRank, theUEdges, theBoundEdge, theFace, theFirstUEdge,
/// theSecondUEdge)`: the u-edges of rows `the_index` and `the_index + 1` of column
/// `the_rank` as met in the face, swapped when the first one does not start (rank 1) or
/// end (rank 0) at the start of the bound edge.
fn get_u_edges(
    the_index: i32,
    the_rank: i32,
    the_u_edges: &ShapeTable,
    the_bound_edge: &Edge,
    the_face: &Face,
) -> Result<Option<(Edge, Edge)>, Failure> {
    let a_ue1: &Shape = cell(the_u_edges, the_index, the_u_edges.lower_col() + the_rank)?;
    let a_ue2: &Shape = cell(
        the_u_edges,
        the_index + 1,
        the_u_edges.lower_col() + the_rank,
    )?;
    let a_face = Face::unchecked(the_face.oriented(Orientation::Forward));
    let (mut e1, mut e2): (Option<Edge>, Option<Edge>) = (None, None);
    for e in Explorer::new(&a_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if e1.is_none() && a_ue1.is_same(&e) {
            e1 = Some(topods::edge(&e)?);
        } else if e2.is_none() && a_ue2.is_same(&e) {
            e2 = Some(topods::edge(&e)?);
        }
    }
    let (Some(e1), Some(e2)) = (e1, e2) else {
        return Ok(None);
    };
    let c1 = BRepTool::curve_on_face(&e1, &a_face)?;
    if c1.pcurve.is_null() {
        return Ok(None);
    }
    let pu1: Point2 = pcurve_value(&c1.pcurve, if the_rank == 0 { c1.last } else { c1.first })?;
    let c2 = BRepTool::curve_on_face(the_bound_edge, &a_face)?;
    if c2.pcurve.is_null() {
        return Ok(None);
    }
    let (f, _) = BRepTool::range(the_bound_edge)?;
    let pf: Point2 = pcurve_value(&c2.pcurve, f)?;
    let a_v: Vertex = if the_rank == 0 {
        topexp::last_vertex(&e1, false)
    } else {
        topexp::first_vertex(&e1, false)
    };
    let a_bas = BRepAdaptorSurface::with_face(&a_face, false)?;
    if pf.distance(pu1) > a_bas.u_resolution(BRepTool::vertex_tolerance(&a_v)?)? {
        return Ok(Some((e2, e1)));
    }
    Ok(Some((e1, e2)))
}
