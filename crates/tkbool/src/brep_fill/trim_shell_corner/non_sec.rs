// Rust translation of BRepFill_TrimShellCorner::MakeFacesNonSec
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_TrimShellCorner.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The trimming of two faces at a corner when they do not intersect: the bounds are
//! cut at the common vertices of the u-edges and joined by a new edge.
use super::BRepFillTrimShellCorner;
use super::ds_queries::find_common_vertex_ds;
use super::gap::find_free_vertices;
use crate::brep_fill::shape_table::cell;
use rustycad_tkbo::bop_ds::DataStructure;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkshhealing::shape_fix::ShapeFixEdge;
use rustycad_tktopalgo::brep_lib::BRepLibMakeEdge;

/// The common vertices of the consecutive u-edges of the two columns and which pairs
/// share one (0 none, 1 the first pair, 2 the second, 3 both).
struct UEdgeCommons {
    vertices: Vec<Vertex>,
    flag: i32,
}

impl BRepFillTrimShellCorner {
    /// `MakeFacesNonSec(theIndex, theDS, theFaceIndex1, theFaceIndex2)`: the history of
    /// the two faces of row `the_index` when they do not intersect; false when the
    /// bounds cannot be joined.
    pub(super) fn make_faces_non_sec(
        &mut self,
        the_index: i32,
        the_ds: &DataStructure,
        the_face_index1: i32,
        the_face_index2: i32,
    ) -> Result<bool, Failure> {
        let a_e1: Shape = cell(&self.my_bounds, the_index, 1)?.clone();
        let a_e2: Shape = cell(&self.my_bounds, the_index, 2)?.clone();
        let (an_index1, an_index2) = (the_ds.index(&a_e1), the_ds.index(&a_e2));
        let common =
            find_common_vertex_ds(the_ds, an_index1, an_index2, &self.my_int_point_cross_dir)?;
        let a_substitutor = BRepToolsReShape::new();
        let commons = self.u_edge_commons(the_index, &a_substitutor)?;
        let mut a_new_edge: Option<Edge> = None;
        if common.found {
            if commons.vertices.len() != 1 {
                return Ok(false);
            }
            let (v1, v2) = if commons.flag == 1 {
                (&commons.vertices[0], &common.vertex)
            } else {
                (&common.vertex, &commons.vertices[0])
            };
            a_new_edge = Some(BRepLibMakeEdge::from_vertices(v1, v2)?.edge()?);
        }
        if commons.vertices.len() == 2 {
            a_new_edge = Some(
                BRepLibMakeEdge::from_vertices(&commons.vertices[0], &commons.vertices[1])?
                    .edge()?,
            );
        }
        for fit in 1..=2 {
            let side = BoundSide {
                fit,
                bound: if fit == 1 { &a_e1 } else { &a_e2 },
                bound_index: if fit == 1 { an_index1 } else { an_index2 },
            };
            let face_index: i32 = if fit == 1 {
                the_face_index1
            } else {
                the_face_index2
            };
            if !self.trim_non_sec_face(
                the_index,
                the_ds,
                face_index,
                &side,
                &mut a_new_edge,
                &a_substitutor,
            )? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The u-edges of both pairs replaced by their first new edges in the substitutor,
    /// and the vertices shared by the end of the first and the start of the second.
    fn u_edge_commons(
        &self,
        the_index: i32,
        a_substitutor: &BRepToolsReShape,
    ) -> Result<UEdgeCommons, Failure> {
        let mut commons = UEdgeCommons {
            vertices: Vec::new(),
            flag: 0,
        };
        let (low, up) = (self.my_u_edges.lower_col(), self.my_u_edges.upper_col());
        for (ueit, eindex) in (1..=2).zip(the_index..) {
            let (a_shape1, a_shape2) = (
                cell(&self.my_u_edges, eindex, low)?,
                cell(&self.my_u_edges, eindex, up)?,
            );
            let a_ue1: Edge = topods::edge(&self.current_u_edge(eindex, low)?)?;
            let a_ue2: Edge = topods::edge(&self.current_u_edge(eindex, up)?)?;
            for (old, new) in [(a_shape1, &a_ue1), (a_shape2, &a_ue2)] {
                if !old.is_same(new) {
                    a_substitutor.replace(
                        &old.oriented(Orientation::Forward),
                        &new.oriented(Orientation::Forward),
                    )?;
                }
            }
            let v1: Vertex = topexp::last_vertex(&a_ue1, false);
            if v1.is_same(&topexp::first_vertex(&a_ue2, false)) {
                commons.flag = if commons.flag == 0 { ueit } else { 3 };
                commons.vertices.push(v1);
            }
        }
        Ok(commons)
    }

    /// The compound of the current u-edges of column `fit` (with the new edge, stored
    /// in the history of the v-edge each time) and the vertices of its bound side.
    fn non_sec_compound(
        &mut self,
        the_index: i32,
        fit: i32,
        a_new_edge: Option<&Edge>,
    ) -> Result<(Compound, MapOfShape), Failure> {
        let builder = BRepBuilder::default();
        let mut a_comp = Compound::default();
        builder.make_compound(&mut a_comp);
        let mut a_map_v = MapOfShape::default();
        for eindex in the_index..=the_index + 1 {
            let a_ue: Edge =
                topods::edge(&self.current_u_edge(eindex, self.my_u_edges.lower_col() + fit - 1)?)?;
            let a_v: Vertex = if fit == 1 {
                topexp::first_vertex(&a_ue, false)
            } else {
                topexp::last_vertex(&a_ue, false)
            };
            a_map_v.add(a_v.into_shape());
            builder.add(&mut a_comp, &a_ue)?;
        }
        if let Some(new_edge) = a_new_edge {
            builder.add(&mut a_comp, new_edge)?;
            self.store_v_edge_in_hist_map(the_index, new_edge)?;
        }
        Ok((a_comp, a_map_v))
    }

    /// The face of column `fit` with its bound replaced by the cut bound and the new
    /// edge; false when the free vertices cannot be joined.
    fn trim_non_sec_face(
        &mut self,
        the_index: i32,
        the_ds: &DataStructure,
        face_index: i32,
        side: &BoundSide,
        a_new_edge: &mut Option<Edge>,
        a_substitutor: &BRepToolsReShape,
    ) -> Result<bool, Failure> {
        let (a_comp, a_map_v) = self.non_sec_compound(the_index, side.fit, a_new_edge.as_ref())?;
        let alonevertices: List<Shape> = find_free_vertices(&a_comp, &a_map_v)?;
        if !alonevertices.is_empty() && alonevertices.extent() != 2 {
            return Ok(false);
        }
        let mut a_face: Shape = the_ds.shape(face_index)?.clone();
        let a_face_ori: Orientation = a_face.orientation();
        a_face.set_orientation(Orientation::Forward);
        if let Some(new_edge) = a_new_edge.as_mut() {
            orient_new_edge(new_edge, side.bound, &topods::face(&a_face)?)?;
        }
        let Some(a_ordered_list) =
            ordered_bound_edges(the_ds, side, &alonevertices, a_new_edge.as_ref())?
        else {
            return Ok(false);
        };
        if !a_ordered_list.is_empty() {
            let builder = BRepBuilder::default();
            let mut a_w = Wire::default();
            builder.make_wire(&mut a_w);
            for e in a_ordered_list.iter() {
                builder.add(&mut a_w, e)?;
            }
            a_substitutor.replace(&side.bound.oriented(Orientation::Forward), &a_w)?;
        }
        self.record_face_history(a_substitutor, &a_face, a_face_ori)?;
        Ok(true)
    }
}

/// The bound of a column: its rank (1 or 2), edge and index in the data structure.
struct BoundSide<'a> {
    fit: i32,
    bound: &'a Shape,
    bound_index: i32,
}

/// The new edge made FORWARD, reversed when its ends are nearer to the opposite ends of
/// the bound, with a p-curve added on the face for a non-planar surface.
fn orient_new_edge(a_new_edge: &mut Edge, bound: &Shape, face: &Face) -> Result<(), Failure> {
    a_new_edge.set_orientation(Orientation::Forward);
    let p1 = BRepTool::pnt(&topexp::first_vertex(a_new_edge, false))?;
    let p2 = BRepTool::pnt(&topexp::last_vertex(a_new_edge, false))?;
    let a_e_ref: Edge = topods::edge(bound)?;
    let (ref_first, ref_last) = (
        BRepTool::pnt(&topexp::first_vertex(&a_e_ref, false))?,
        BRepTool::pnt(&topexp::last_vertex(&a_e_ref, false))?,
    );
    let (p11, p22, p12, p21) = (
        p1.distance(&ref_first),
        p2.distance(&ref_last),
        p1.distance(&ref_last),
        p2.distance(&ref_first),
    );
    if p11 > p12 && p22 > p21 {
        a_new_edge.reverse();
    }
    ShapeFixEdge::new().fix_add_pcurve(a_new_edge, face, false, 0.)?;
    Ok(())
}

/// The parameters on the bound of the first paves at the two free vertices.
fn free_vertex_params(
    the_ds: &DataStructure,
    bound_index: i32,
    alonevertices: &List<Shape>,
) -> Result<(Option<f64>, Option<f64>), Failure> {
    let (first, last) = (alonevertices.first(), alonevertices.last());
    let (mut aparam1, mut aparam2): (Option<f64>, Option<f64>) = (None, None);
    for a_pave in the_ds.paves(bound_index)? {
        let a_v: &Shape = the_ds.shape(a_pave.index())?;
        if aparam1.is_none() && first.is_some_and(|v| a_v.is_same(v)) {
            aparam1 = Some(a_pave.parameter());
        }
        if aparam2.is_none() && last.is_some_and(|v| a_v.is_same(v)) {
            aparam2 = Some(a_pave.parameter());
        }
    }
    Ok((aparam1, aparam2))
}

/// The edges replacing the bound: the bound cut between the free vertices (with the new
/// edge after or before it) or the new edge alone; `None` when a free vertex is not on
/// the bound.
fn ordered_bound_edges(
    the_ds: &DataStructure,
    side: &BoundSide,
    alonevertices: &List<Shape>,
    a_new_edge: Option<&Edge>,
) -> Result<Option<List<Shape>>, Failure> {
    let mut a_ordered_list: List<Shape> = List::default();
    if alonevertices.is_empty() {
        if let Some(new_edge) = a_new_edge {
            a_ordered_list.append(new_edge.as_ref().clone());
        }
        return Ok(Some(a_ordered_list));
    }
    let (Some(aparam1), Some(aparam2)) =
        free_vertex_params(the_ds, side.bound_index, alonevertices)?
    else {
        return Ok(None);
    };
    let (first, last) = (
        alonevertices.first().cloned().unwrap_or_default(),
        alonevertices.last().cloned().unwrap_or_default(),
    );
    let (a_v1, a_v2, range) = if aparam1 < aparam2 {
        (first, last, (aparam1, aparam2))
    } else {
        (last, first, (aparam2, aparam1))
    };
    let a_new_bound_e = cut_bound(side.bound, &a_v1, &a_v2, range)?;
    a_ordered_list.append(a_new_bound_e.into_shape());
    if let Some(new_edge) = a_new_edge {
        let bfoundv: bool =
            Explorer::new(new_edge, ShapeEnum::Vertex, ShapeEnum::Shape)?.any(|v| a_v2.is_same(&v));
        if bfoundv {
            a_ordered_list.append(new_edge.as_ref().clone());
        } else {
            a_ordered_list.prepend(new_edge.as_ref().clone());
        }
    }
    Ok(Some(a_ordered_list))
}

/// An empty copy of the bound between the two vertices on the range.
fn cut_bound(
    bound: &Shape,
    a_v1: &Shape,
    a_v2: &Shape,
    (aparam1, aparam2): (f64, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut a_new_bound_e: Edge = topods::edge(&bound.empty_copied()?)?;
    builder.add(
        &mut a_new_bound_e,
        &topods::vertex(a_v1)?.oriented(Orientation::Forward),
    )?;
    builder.add(
        &mut a_new_bound_e,
        &topods::vertex(a_v2)?.oriented(Orientation::Reversed),
    )?;
    builder.range(&a_new_bound_e, aparam1, aparam2, false)?;
    a_new_bound_e.set_orientation(Orientation::Forward);
    Ok(a_new_bound_e)
}
