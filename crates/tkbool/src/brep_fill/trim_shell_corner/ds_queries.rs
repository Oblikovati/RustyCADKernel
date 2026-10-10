// Rust translation of the static helpers SplitUEdges, FindCommonVertex (on the data
// structure), FindVertex, FindNextVertex, GetPave and FilterSectionEdges, and of the
// interference search of Perform (ModelingAlgorithms/TKBool/BRepFill/
// BRepFill_TrimShellCorner.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The queries of BRepFill_TrimShellCorner on the data structure of the pave filler.
use super::HistoryMap;
use crate::brep_fill::shape_table::{ShapeTable, cell};
use rustycad_tkbo::bop_ds::{BopdsCurve, DataStructure, Pave, PaveBlock, PaveBlockHandle};
use rustycad_tkbo::int_tools::{BeanFaceIntersector, Context, Tools};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::{DynamicArray, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction3, Vector3};
use std::sync::{MutexGuard, PoisonError};

/// The locked pave block.
pub(super) fn lock_pb(block: &PaveBlockHandle) -> MutexGuard<'_, PaveBlock> {
    block.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The first face/face interference of the pair `(f1, f2)` (in this order) and whether
/// one of its curves has section edges; the sub-shape interference test runs on each
/// interference without point and curve before the pair test, as natively.
pub(super) fn face_pair_interference(
    the_ds: &DataStructure,
    an_index1: i32,
    an_index2: i32,
) -> Result<Option<(i32, bool)>, Failure> {
    for (i, a_ffi) in the_ds.interf_ff_records().iter().enumerate() {
        let (n_f1, n_f2) = a_ffi.indices();
        let (a_nb_p, a_nb_c) = (a_ffi.points().iter().len(), a_ffi.curves().iter().len());
        if a_nb_p == 0 && a_nb_c == 0 && !the_ds.has_interference_sub_shapes(n_f1, n_f2)? {
            continue;
        }
        if n_f1 == an_index1 && n_f2 == an_index2 {
            let bhassec: bool = a_ffi
                .curves()
                .iter()
                .any(|c: &BopdsCurve| !c.pave_blocks().is_empty());
            return Ok(Some((i as i32, bhassec)));
        }
    }
    Ok(None)
}

/// The result of `FindCommonVertex` on the data structure: whether a vertex was found,
/// and the last vertex examined with its parameters (set even when the direction test
/// fails, as the native output arguments are).
pub(super) struct CommonVertexDs {
    pub(super) found: bool,
    pub(super) vertex: Vertex,
    pub(super) param_on_e1: f64,
    pub(super) param_on_e2: f64,
}

/// `FindCommonVertex(theDS, theEIndex1, theEIndex2, theCrossDirection, ...)`: the
/// vertex of the first edge/edge interference of the edges whose tangent cross product
/// goes along `the_cross_direction` (both normalized, as the native `gp_Dir`
/// conversions do), with its parameters on the edges.
pub(super) fn find_common_vertex_ds(
    the_ds: &DataStructure,
    the_e_index1: i32,
    the_e_index2: i32,
    the_cross_direction: &Vector3,
) -> Result<CommonVertexDs, Failure> {
    let mut result = CommonVertexDs {
        found: false,
        vertex: Vertex::default(),
        param_on_e1: 0.,
        param_on_e2: 0.,
    };
    let the_e1: Edge = topods::edge(the_ds.shape(the_e_index1)?)?;
    let the_e2: Edge = topods::edge(the_ds.shape(the_e_index2)?)?;
    let (a_bc1, a_bc2) = (
        BRepAdaptorCurve::with_edge(&the_e1)?,
        BRepAdaptorCurve::with_edge(&the_e2)?,
    );
    for a_ee in the_ds.interf_ee_records().iter() {
        let same_order: bool = the_e_index1 == a_ee.index1() && the_e_index2 == a_ee.index2();
        if !(same_order || (the_e_index1 == a_ee.index2() && the_e_index2 == a_ee.index1()))
            || !a_ee.has_index_new()
        {
            continue;
        }
        let a_cp = a_ee.common_part();
        if a_cp.shape_type() != ShapeEnum::Vertex {
            continue;
        }
        result.vertex = Vertex::unchecked(the_ds.shape(a_ee.index_new())?.clone());
        if the_e_index1 == a_ee.index1() {
            Tools::vertex_parameters(a_cp, &mut result.param_on_e1, &mut result.param_on_e2)?;
        } else {
            Tools::vertex_parameters(a_cp, &mut result.param_on_e2, &mut result.param_on_e1)?;
        }
        let (_, a_dir_on_e1) = a_bc1.d1(result.param_on_e1)?;
        let (_, a_dir_on_e2) = a_bc2.d1(result.param_on_e2)?;
        let cross = Direction3::from_vector(a_dir_on_e1.crossed(a_dir_on_e2))?;
        if cross.dot(Direction3::from_vector(*the_cross_direction)?) > precision::square_confusion()
        {
            result.found = true;
            break;
        }
    }
    Ok(result)
}

/// The new vertex of the vertex/vertex interference of the end of `a_e1` and the start
/// of `a_e2`, with their parameters on the edges.
fn vertex_vertex_common(
    the_ds: &DataStructure,
    a_e1: &Edge,
    a_e2: &Edge,
) -> Result<Option<(Vertex, f64, f64)>, Failure> {
    let v1: Vertex = topexp::last_vertex(a_e1, false);
    let v2: Vertex = topexp::first_vertex(a_e2, false);
    let (vindex1, vindex2) = (the_ds.index(&v1), the_ds.index(&v2));
    for a_vv in the_ds.interf_vv_records().iter() {
        let pair: bool = (vindex1 == a_vv.index1() && vindex2 == a_vv.index2())
            || (vindex1 == a_vv.index2() && vindex2 == a_vv.index1());
        if !pair || !a_vv.has_index_new() {
            continue;
        }
        let common: Vertex = topods::vertex(the_ds.shape(a_vv.index_new())?)?;
        return Ok(Some((
            common,
            BRepTool::parameter(&v1, a_e1)?,
            BRepTool::parameter(&v2, a_e2)?,
        )));
    }
    Ok(None)
}

/// `SplitUEdges(theUEdges, theDS, theCrossDirection, theHistMap)`: each pair of
/// u-edges not yet split is cut at its common vertex (from an edge/edge or a
/// vertex/vertex interference) and the halves stored in the history.
pub(super) fn split_u_edges(
    the_u_edges: &ShapeTable,
    the_ds: &DataStructure,
    the_cross_direction: &Vector3,
    the_hist_map: &mut HistoryMap,
) -> Result<bool, Failure> {
    let (low_col, up_col) = (the_u_edges.lower_col(), the_u_edges.upper_col());
    for ueit in the_u_edges.lower_row()..=the_u_edges.upper_row() {
        let a_e1: Shape = cell(the_u_edges, ueit, low_col)?.clone();
        let a_e2: Shape = cell(the_u_edges, ueit, up_col)?.clone();
        if the_hist_map.is_bound(&a_e1) || the_hist_map.is_bound(&a_e2) {
            continue;
        }
        let (e1, e2) = (topods::edge(&a_e1)?, topods::edge(&a_e2)?);
        let common = find_common_vertex_ds(
            the_ds,
            the_ds.index(&a_e1),
            the_ds.index(&a_e2),
            the_cross_direction,
        )?;
        let mut found =
            common
                .found
                .then_some((common.vertex, common.param_on_e1, common.param_on_e2));
        if found.is_none() {
            found = vertex_vertex_common(the_ds, &e1, &e2)?;
        }
        if let Some((common, apar1, apar2)) = found {
            split_u_edge_pair(&e1, &e2, common, (apar1, apar2), the_hist_map)?;
        }
    }
    Ok(true)
}

/// The two u-edges cut at the common vertex: the first one ends at it, the second one
/// starts at it.
fn split_u_edge_pair(
    e1: &Edge,
    e2: &Edge,
    mut common: Vertex,
    (apar1, apar2): (f64, f64),
    the_hist_map: &mut HistoryMap,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut new_e1: Edge = topods::edge(&e1.empty_copied()?)?;
    let (mut a_v1, _) = topexp::vertices(e1, false);
    new_e1.set_orientation(Orientation::Forward);
    a_v1.set_orientation(Orientation::Forward);
    builder.add(&mut new_e1, &a_v1)?;
    common.set_orientation(Orientation::Reversed);
    builder.add(&mut new_e1, &common)?;
    let (f, _) = BRepTool::range(e1)?;
    builder.range(&new_e1, f, apar1, false)?;
    let mut new_e2: Edge = topods::edge(&e2.empty_copied()?)?;
    let (_, a_v2) = topexp::vertices(e2, false);
    new_e2.set_orientation(Orientation::Forward);
    common.set_orientation(Orientation::Forward);
    builder.add(&mut new_e2, &common)?;
    builder.add(&mut new_e2, &a_v2)?;
    let (_, l) = BRepTool::range(e2)?;
    builder.range(&new_e2, apar2, l, false)?;
    the_hist_map.bind(e1.as_ref().clone(), single(new_e1.into_shape()));
    the_hist_map.bind(e2.as_ref().clone(), single(new_e2.into_shape()));
    Ok(())
}

/// A list of one shape.
pub(super) fn single(shape: Shape) -> List<Shape> {
    let mut list: List<Shape> = List::default();
    list.append(shape);
    list
}

/// The first new shape recorded for `shape`, if any.
pub(super) fn first_new(the_hist_map: &HistoryMap, shape: &Shape) -> Option<Shape> {
    the_hist_map
        .seek(shape)
        .and_then(|lst| lst.first().cloned())
}

/// `FindVertex(theEdge, theRank, theDS, theHistMap, theVertex, thePave)`: the end
/// (rank 0) or start vertex of the new edge of `the_edge` and its pave (the index in
/// the data structure, searched among the new shapes when it is not a source one).
pub(super) fn find_vertex(
    the_edge: &Shape,
    the_rank: i32,
    the_ds: &DataStructure,
    the_hist_map: &HistoryMap,
    the_pave: &mut Pave,
) -> Result<Option<Vertex>, Failure> {
    let Some(new_edge) = first_new(the_hist_map, the_edge) else {
        return Ok(None);
    };
    let a_new_edge: Edge = topods::edge(&new_edge)?;
    let (f, l) = BRepTool::range(&a_new_edge)?;
    let the_vertex: Vertex = if the_rank == 0 {
        the_pave.set_parameter(l);
        topexp::last_vertex(&a_new_edge, false)
    } else {
        the_pave.set_parameter(f);
        topexp::first_vertex(&a_new_edge, false)
    };
    let mut an_index: i32 = the_ds.index(&the_vertex);
    if an_index == -1 {
        for i in the_ds.nb_source_shapes()..the_ds.nb_shapes() {
            if the_ds.shape(i)?.is_same(&the_vertex) {
                an_index = i;
                break;
            }
        }
    }
    the_pave.set_index(an_index);
    Ok(Some(the_vertex))
}

/// `FindNextVertex(theEdgeIndex, thePrevPave, theDS, theNextVertex, thePave)`: the
/// first new vertex among the paves of the edge after `the_prev_pave` (from the start
/// for the default pave).
pub(super) fn find_next_vertex(
    the_edge_index: i32,
    the_prev_pave: &Pave,
    the_ds: &DataStructure,
) -> Result<Option<(Vertex, Pave)>, Failure> {
    let mut b_take_pave: bool = the_prev_pave.is_equal(&Pave::default());
    for a_tmp_pave in the_ds.paves(the_edge_index)? {
        if b_take_pave {
            if the_ds.is_new_shape(a_tmp_pave.index()) {
                return Ok(Some((
                    Vertex::unchecked(the_ds.shape(a_tmp_pave.index())?.clone()),
                    a_tmp_pave,
                )));
            }
        } else if a_tmp_pave.is_equal(the_prev_pave) {
            b_take_pave = true;
        }
    }
    Ok(None)
}

/// `GetPave(theEdgeIndex, isFirst, theDS, thePave)`: the first or last pave of the
/// edge.
pub(super) fn get_pave(
    the_edge_index: i32,
    is_first: bool,
    the_ds: &DataStructure,
) -> Result<Option<Pave>, Failure> {
    let a_lp: Vec<Pave> = the_ds.paves(the_edge_index)?;
    Ok(if is_first {
        a_lp.first().copied()
    } else {
        a_lp.last().copied()
    })
}

/// The split edge of the pave block of the bound edge between the two paves.
pub(super) fn split_between(
    the_ds: &DataStructure,
    the_bound_edge_index: i32,
    (a_pave1, a_pave2): (&Pave, &Pave),
) -> Result<Option<Shape>, Failure> {
    for a_pb1 in the_ds.pave_blocks(the_bound_edge_index)? {
        let pb = lock_pb(a_pb1);
        if pb.original_edge() == the_bound_edge_index
            && pb.pave1().is_equal(a_pave1)
            && pb.pave2().is_equal(a_pave2)
            && pb.edge() > 0
        {
            return Ok(Some(the_ds.shape(pb.edge())?.clone()));
        }
    }
    Ok(None)
}

/// `RemoveEdges(theSourceComp, theListToRemove, theResultComp)`: a new compound of the
/// edges of the source that are not in the list.
pub(super) fn remove_edges(
    the_source_comp: &Compound,
    the_list_to_remove: &List<Shape>,
) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut a_comp = Compound::default();
    builder.make_compound(&mut a_comp);
    for e in
        rustycad_tkbrep::topexp::Explorer::new(the_source_comp, ShapeEnum::Edge, ShapeEnum::Shape)?
    {
        if !the_list_to_remove.iter().any(|r| e.is_same(r)) {
            builder.add(&mut a_comp, &e)?;
        }
    }
    Ok(a_comp)
}

/// `FilterSectionEdges(theBCurves, theSecPlane, theDS, theResult)`: the section edges
/// of the curves, keeping (when a plane is given) those lying wholly on the plane.
pub(super) fn filter_section_edges(
    the_b_curves: &DynamicArray<BopdsCurve>,
    the_sec_plane: &Face,
    the_ds: &DataStructure,
) -> Result<Compound, Failure> {
    let builder = BRepBuilder::default();
    let mut the_result = Compound::default();
    builder.make_compound(&mut the_result);
    for a_b_curve in the_b_curves.iter() {
        for a_pb in a_b_curve.pave_blocks() {
            let n_sect: i32 = lock_pb(a_pb).edge();
            let a_s: Shape = the_ds.shape(n_sect)?.clone();
            let an_edge: Edge = topods::edge(&a_s)?;
            if the_sec_plane.is_null() || edge_on_plane(&an_edge, the_sec_plane)? {
                builder.add(&mut the_result, &a_s)?;
            }
        }
    }
    Ok(the_result)
}

/// Whether the bean/face intersection of the edge with the plane covers the whole edge
/// (true when the intersector fails, as natively).
fn edge_on_plane(an_edge: &Edge, the_sec_plane: &Face) -> Result<bool, Failure> {
    let mut an_intersector = BeanFaceIntersector::from_shapes(an_edge, the_sec_plane)?;
    let (f, l) = BRepTool::range(an_edge)?;
    an_intersector.set_bean_parameters(f, l);
    an_intersector.set_context(&Handle::new(Context::default()));
    an_intersector.perform()?;
    if !an_intersector.is_done() {
        return Ok(true);
    }
    Ok(an_intersector.result().iter().any(|a_range| {
        (a_range.first() - f) < precision::p_confusion()
            && (l - a_range.last()) < precision::p_confusion()
    }))
}
