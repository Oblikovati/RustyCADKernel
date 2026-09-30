// Rust translation of BRepBuilderAPI_Sewing::SameParameterEdge(edge1, edge2,
// listFaces1, listFaces2, secForward, whichSec, firstCall) (BRepBuilderAPI_Sewing.cxx,
// except its p-curve loop) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The merge of two sections into one new edge: the 3d curve and range of the longer
//! (or chosen) section, vertices covering both, the p-curves of both, then
//! SameParameter, with a second try on the other section and a measured tolerance.
use super::BRepBuilderAPISewing;
use super::pair_pcurves::PcurveTransfer;
use super::vertex_tolerance::{
    compute_tolerance_vertex2, compute_tolerance_vertex3, find_nm_vertices,
};
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::Point3;

/// The two sections and their faces, as the caller passes them.
pub(super) type SectionPair<'a> = (&'a Edge, &'a Edge, &'a List<Shape>, &'a List<Shape>);

/// The number of samples of the tolerance measure.
const NB_SAMPLES: i32 = 23;

/// The length of an edge's 3d curve over its range (`GCPnts_AbscissaPoint::Length`).
fn curve_length(edge: &Edge) -> Result<f64, Failure> {
    let (c3d, f, l) = BRepTool::curve(edge)?;
    let adaptor = GeomAdaptorCurve::with_curve(&c3d)?;
    AbscissaPoint::length_between(&adaptor, f, l)
}

/// The reference (whose 3d curve is kept) and the other section: the longer first on
/// the first call, the swap of the previous choice on the second.
fn sort_sections(
    pair: SectionPair<'_>,
    which_sec: &mut i32,
    first_call: bool,
) -> Result<(Edge, Edge), Failure> {
    let (edge_first, edge_last, _, _) = pair;
    let swap = if first_call {
        curve_length(edge_first)? < curve_length(edge_last)?
    } else {
        *which_sec == 1
    };
    *which_sec = if swap { 2 } else { 1 };
    if swap {
        return Ok((edge_last.clone(), edge_first.clone()));
    }
    Ok((edge_first.clone(), edge_last.clone()))
}

/// The new vertices of the merged edge, or None when the sections share their
/// vertices the wrong way (open sections only).
fn merged_vertices(
    edge1: &Edge,
    edge2: &Edge,
    sec_forward: bool,
) -> Result<Option<(Vertex, Vertex)>, Failure> {
    let (v11, v12) = topexp::vertices(edge1, false);
    let (v21, v22) = topexp::vertices(edge2, false);
    let (closed1, closed2) = (v11.is_same(&v12), v21.is_same(&v22));
    if !closed1 && !closed2 {
        let wrong = if sec_forward {
            v11.is_same(&v22) || v12.is_same(&v21)
        } else {
            v11.is_same(&v21) || v12.is_same(&v22)
        };
        if wrong {
            return Ok(None);
        }
    }
    if closed1 || closed2 {
        let v1_new = match (closed1, closed2) {
            (true, true) => compute_tolerance_vertex2(&v11, &v21)?,
            (true, false) => compute_tolerance_vertex3([&v22, &v21, &v11])?,
            _ => compute_tolerance_vertex3([&v11, &v12, &v21])?,
        };
        return Ok(Some((v1_new.clone(), v1_new)));
    }
    let (w1, w2) = if sec_forward {
        (&v21, &v22)
    } else {
        (&v22, &v21)
    };
    let v1_new = if v11.is_same(w1) {
        v11.clone()
    } else {
        compute_tolerance_vertex2(&v11, w1)?
    };
    let v2_new = if v12.is_same(w2) {
        v12.clone()
    } else {
        compute_tolerance_vertex2(&v12, w2)?
    };
    Ok(Some((v1_new, v2_new)))
}

/// The 3d curve of the reference section in global coordinates, with its range.
fn global_curve3d(edge1: &Edge) -> Result<(Handle<dyn GeomCurve>, f64, f64), Failure> {
    let found = BRepTool::curve_with_location(edge1)?;
    let mut c3d = found.curve.clone();
    if !found.location.is_identity()
        && let Some(curve) = found.curve.get()
    {
        c3d = curve.copy()?.cast::<dyn GeomCurve>();
        if let Some(copy) = c3d.get() {
            copy.transform(found.location.transformation())?;
        }
    }
    Ok((c3d, found.first, found.last))
}

/// The new edge with the 3d curve, tolerance and range of the reference section,
/// neither SameRange nor SameParameter.
fn new_edge_on_curve(
    edge1: &Edge,
    c3d: &Handle<dyn GeomCurve>,
    (first, last): (f64, f64),
) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge)?;
    edge.set_orientation(edge1.orientation());
    builder.update_edge_curve(&edge, c3d, BRepTool::edge_tolerance(edge1)?)?;
    builder.range(&edge, first, last, false)?;
    builder.same_range(&edge, false)?;
    builder.same_parameter(&edge, false)?;
    Ok(edge)
}

/// The vertices of the new edge: FORWARD, REVERSED, then the non-manifold ones.
fn add_vertices(
    edge: &Edge,
    (v1, v2): (Vertex, Vertex),
    nm_vertices: &Sequence<Shape>,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut an_edge = edge.oriented(Orientation::Forward);
    builder.add(&mut an_edge, &v1.oriented(Orientation::Forward))?;
    builder.add(&mut an_edge, &v2.oriented(Orientation::Reversed))?;
    for vertex in nm_vertices.iter() {
        builder.add(&mut an_edge, vertex)?;
    }
    Ok(())
}

/// The new edge of the merge with its vertices (None when the sections share their
/// vertices the wrong way), and the 3d curve of the reference in global coordinates.
#[allow(clippy::type_complexity)]
fn merged_edge(
    edge1: &Edge,
    edge2: &Edge,
    sec_forward: bool,
    range: (f64, f64),
) -> Result<Option<(Edge, (Handle<dyn GeomCurve>, f64, f64))>, Failure> {
    // To keep NM vertices on edge.
    let (mut nm_vertices, mut nm_pars) = (Sequence::default(), Sequence::default());
    find_nm_vertices(edge1, &mut nm_vertices, &mut nm_pars)?;
    find_nm_vertices(edge2, &mut nm_vertices, &mut nm_pars)?;
    let curve = global_curve3d(edge1)?;
    let edge = new_edge_on_curve(edge1, &curve.0, range)?;
    let Some(vertices) = merged_vertices(edge1, edge2, sec_forward)? else {
        return Ok(None);
    };
    add_vertices(&edge, vertices, &nm_vertices)?;
    Ok(Some((edge, curve)))
}

impl BRepBuilderAPISewing {
    /// `SameParameterEdge(edge1, edge2, listFaces1, listFaces2, secForward, whichSec,
    /// firstCall)`: a new edge merging two sections (null for a floating section, for
    /// sections sharing their vertices the wrong way, for a seam in manifold mode, or
    /// when its tolerance exceeds the maximal one); `which_sec` tells which section
    /// gave the 3d curve.
    pub(crate) fn same_parameter_edge_of_pair(
        &self,
        pair: SectionPair<'_>,
        sec_forward: bool,
        which_sec: &mut i32,
        first_call: bool,
    ) -> Result<Edge, Failure> {
        let (_, _, faces_first, faces_last) = pair;
        if faces_first.is_empty() || faces_last.is_empty() {
            return Ok(Edge::default());
        }
        let (edge1, edge2) = sort_sections(pair, which_sec, first_call)?;
        let range = BRepTool::range(&edge1)?;
        let Some((mut edge, curve)) = merged_edge(&edge1, &edge2, sec_forward, range)? else {
            return Ok(Edge::default());
        };
        let (faces1, faces2) = if *which_sec == 1 {
            (faces_first, faces_last)
        } else {
            (faces_last, faces_first)
        };
        let sections = (&edge1, &edge2, faces1, faces2);
        let is_res_edge = match self.transfer_pcurves(sections, &edge, sec_forward, range)? {
            PcurveTransfer::Rejected => return Ok(Edge::default()),
            PcurveTransfer::Done(is_res_edge) => is_res_edge,
        };
        let (is_same_par, tol_reached) = self.same_parameter_reached(&edge, is_res_edge)?;
        if first_call && (!is_res_edge || !is_same_par || tol_reached > self.my_tolerance) {
            let retry = (pair, sec_forward, which_sec);
            self.second_try(retry, &mut edge, tol_reached, (&curve.0, curve.1, curve.2))?;
        }
        if BRepTool::edge_tolerance(&edge)? > self.max_tolerance() {
            edge.nullify();
        }
        Ok(edge)
    }

    /// The merge on the other section (the second call); it replaces the edge when it
    /// is SameParameter with a smaller tolerance, otherwise the edge takes the
    /// tolerance measured against the 3d curve.
    fn second_try(
        &self,
        (pair, sec_forward, which_sec): (SectionPair<'_>, bool, &mut i32),
        edge: &mut Edge,
        tol_reached: f64,
        curve: (&Handle<dyn GeomCurve>, f64, f64),
    ) -> Result<(), Failure> {
        let mut which_secn = *which_sec;
        let s_edge = self.same_parameter_edge_of_pair(pair, sec_forward, &mut which_secn, false)?;
        if !s_edge.is_null() {
            let tol_reached2 = BRepTool::edge_tolerance(&s_edge)?;
            if BRepTool::same_parameter(&s_edge)? && tol_reached2 < tol_reached {
                *edge = s_edge;
                *which_sec = which_secn;
                return Ok(());
            }
        }
        if edge.is_null() {
            return Ok(());
        }
        self.set_measured_tolerance(edge, curve, tol_reached)
    }

    /// The guarded SameParameter of the merged edge: whether the edge is SameParameter
    /// afterwards, with its tolerance (infinite when not).
    fn same_parameter_reached(
        &self,
        edge: &Edge,
        is_res_edge: bool,
    ) -> Result<(bool, f64), Failure> {
        if is_res_edge {
            self.same_parameter_guarded(edge);
        }
        // The native catch would reset the flag; BRep_Tool raises only for a null edge.
        match BRepTool::same_parameter(edge) {
            Ok(true) => Ok((true, BRepTool::edge_tolerance(edge)?)),
            _ => Ok((false, precision::infinite())),
        }
    }

    /// The fallback of a failed merge: the deviation of the last p-curve from the
    /// reference 3d curve at 23 samples gives the tolerance when it is smaller than the
    /// reached one (set directly when the reached one exceeds the maximal tolerance);
    /// the edge is marked SameParameter.
    fn set_measured_tolerance(
        &self,
        edge: &Edge,
        (c3d, first3d, last3d): (&Handle<dyn GeomCurve>, f64, f64),
        tol_reached: f64,
    ) -> Result<(), Failure> {
        let adaptor = GeomAdaptorCurve::with_curve(c3d)?;
        let delta_t: f64 = (last3d - first3d) / f64::from(NB_SAMPLES - 1);
        let mut c3d_points: Vec<Point3> = Vec::new();
        for i in 1..=NB_SAMPLES {
            c3d_points.push(adaptor.value(first3d + f64::from(i - 1) * delta_t)?);
        }
        let mut max_tol: f64 = -1.0;
        let mut j = 1;
        loop {
            let found = BRepTool::curve_on_surface_by_index(edge, j)?;
            if found.pcurve.is_null() {
                break;
            }
            max_tol = pcurve_deviation(&found, &c3d_points)?;
            j += 1;
        }
        let builder = BRepBuilder::default();
        if max_tol >= 0.0 && max_tol < tol_reached {
            if tol_reached > self.max_tolerance() {
                // Set tolerance directly to overwrite too large tolerance.
                edge_tshape(edge, "BRep_TEdge::Tolerance")?.set_tolerance(max_tol);
            } else {
                builder.update_edge_tolerance(edge, max_tol)?;
            }
        }
        builder.same_parameter(edge, true)
    }
}

/// The largest distance between the p-curve's points on its surface and the samples
/// of the 3d curve, times `1 + 1e-7`, at least Precision::Confusion().
fn pcurve_deviation(
    found: &rustycad_tkbrep::brep::EdgeCurveOnSurface,
    c3d_points: &[Point3],
) -> Result<f64, Failure> {
    let mut surface: Handle<dyn GeomSurface> = found.surface.clone();
    if !found.location.is_identity()
        && let Some(s) = found.surface.get()
    {
        surface = s
            .transformed(found.location.transformation())?
            .cast::<dyn GeomSurface>();
    }
    let s = surface
        .get()
        .ok_or_else(|| super::failures::null_object("SameParameterEdge: null surface"))?;
    let c2d = found
        .pcurve
        .get()
        .ok_or_else(|| super::failures::null_object("SameParameterEdge: null p-curve"))?;
    let (first, last) = (found.first, found.last);
    let delta_t: f64 = (last - first) / f64::from(NB_SAMPLES - 1);
    let mut dist2: f64 = 0.0;
    for (i, point) in c3d_points.iter().enumerate() {
        let p2d = c2d.value(first + i as f64 * delta_t)?;
        let p2 = s.d0(p2d.x(), p2d.y())?;
        let dist = p2.square_distance(point);
        if dist > dist2 {
            dist2 = dist;
        }
    }
    Ok(std_max(dist2.sqrt() * (1. + 1e-7), precision::confusion()))
}
