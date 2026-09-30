// Rust translation of BRepLib_FuseEdges::BuildListResultEdges (BRepLib_FuseEdges.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The fused edge of each chain: the curve of its first edge between the end vertices
//! of the chain (a concatenated B-spline, a closed B-spline with its origin moved, or
//! a curve extended to the vertices when needed).
use super::BRepLibFuseEdges;
use super::smoothing::make_closed_curve;
use crate::brep_lib::make_edge::BRepLibMakeEdge;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBoundedCurve, GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::convert::ParameterisationType;

/// A Standard_ConstructionError of the fusion.
fn fusion_error(message: &str) -> Failure {
    Failure::with_message("Standard_ConstructionError", message)
}

impl BRepLibFuseEdges {
    /// `BuildListResultEdges`: nothing (and not marked built) without chains; else the
    /// fused edge of each chain, kept when its p-curves are updated.
    pub(super) fn build_list_result_edges(&mut self) -> Result<(), Failure> {
        if self.my_map_lst_edg.extent() == 0 {
            return Ok(());
        }
        self.my_map_edg.clear(false);
        let chains: Vec<(i32, List<Shape>)> = self
            .my_map_lst_edg
            .items()
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        for (i_lst, chain) in chains {
            let old_edge = Edge::unchecked(chain.first().expect("a chain has edges").clone());
            let mut new_edge = self.fused_edge(&old_edge, &chain)?;
            if self.update_pcurve(&old_edge, &mut new_edge, &chain)? {
                self.my_map_edg.bind(i_lst, new_edge.into_shape());
            }
        }
        self.my_result_edges_done = true;
        Ok(())
    }

    /// The edge of one chain.
    fn fused_edge(&self, old_edge: &Edge, chain: &List<Shape>) -> Result<Edge, Failure> {
        let (vf, vl) = chain_ends(old_edge, chain);
        let found = BRepTool::curve_with_location(old_edge)?;
        let (mut f, mut l) = (found.first, found.last);
        let mut c: Handle<dyn GeomCurve> = found.curve.clone();
        let curve = c.get().ok_or_else(|| {
            crate::brep_lib::errors::null_object("FuseEdges: an edge without 3d curve")
        })?;
        if !found.location.is_identity() {
            c = curve.transformed(found.location.transformation())?.cast();
        }
        // if the curve is trimmed we get the basis curve to fit the new vertices
        // otherwise the makeedge will fail.
        if let Some(trimmed) = c.cast::<GeomTrimmedCurve>().get() {
            c = trimmed.basis_curve();
        }
        if self.my_concat_bspl && c.cast::<GeomBSplineCurve>().get().is_some() {
            c = concatenated(&c, (&mut f, &mut l), chain)?;
        }
        let maker = make_fused_edge(&mut c, &vf, &vl, &mut f, &mut l)?;
        maker.edge()
    }
}

/// The end vertices of a chain, from its first and last edges (swapped when the first
/// edge is REVERSED).
fn chain_ends(old_edge: &Edge, chain: &List<Shape>) -> (Vertex, Vertex) {
    let first = Edge::unchecked(chain.first().expect("a chain").clone());
    let last = Edge::unchecked(chain.last().expect("a chain").clone());
    if old_edge.orientation() == Orientation::Reversed {
        (
            topexp::last_vertex(&last, true),
            topexp::first_vertex(&first, true),
        )
    } else {
        (
            topexp::first_vertex(&first, true),
            topexp::last_vertex(&last, true),
        )
    }
}

/// "Prepare common BSpline curve": the curves of the chain edges, each trimmed to its
/// range, concatenated (Standard_ConstructionError when one does not join); the range
/// is that of the last edge afterwards, as natively.
fn concatenated(
    c: &Handle<dyn GeomCurve>,
    (f, l): (&mut f64, &mut f64),
    chain: &List<Shape>,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    let first: Handle<dyn GeomBoundedCurve> =
        Handle::new(GeomTrimmedCurve::new(c, *f, *l, true, true)?).cast();
    let mut concat =
        CompCurveToBSplineCurve::with_curve(&first, ParameterisationType::TgtThetaOver2)?;
    for shape in chain.iter().skip(1) {
        let (curve, first, last) = BRepTool::curve(&Edge::unchecked(shape.clone()))?;
        (*f, *l) = (first, last);
        let trimmed: Handle<dyn GeomBoundedCurve> =
            Handle::new(GeomTrimmedCurve::new(&curve, *f, *l, true, true)?).cast();
        if !concat.add(&trimmed, precision::confusion(), false, true, 0)? {
            // cannot merge curves
            return Err(fusion_error("FuseEdges : Concatenation failed"));
        }
    }
    Ok(concat.bspline_curve().cast())
}

/// The MakeEdge of the fused curve: a closed B-spline between one vertex (its origin
/// moved to the vertex when needed), else the curve between the two vertices, extended
/// to them when that fails.
fn make_fused_edge(
    c: &mut Handle<dyn GeomCurve>,
    vf: &Vertex,
    vl: &Vertex,
    f: &mut f64,
    l: &mut f64,
) -> Result<BRepLibMakeEdge, Failure> {
    let mut maker = BRepLibMakeEdge::new();
    let is_bspline: bool = c.cast::<GeomBSplineCurve>().get().is_some();
    if vf.is_same(vl) && is_bspline {
        closed_edge(&mut maker, c, vf, vl, (f, l))?;
    } else {
        maker.init_curve_vertices(c, vf, vl)?;
    }
    if maker.is_done() {
        return Ok(maker);
    }
    // the MakeEdge has fails, one reason could be that the new Vertices are outside
    // the curve which is not infinite and limited to old vertices
    // we try to use ExtendCurveToPoint, then rebuild the NewEdge
    let curve = c.get().expect("a curve");
    let mut ext_c = curve.copy()?.cast::<dyn GeomBoundedCurve>();
    if ext_c.is_null() {
        return Err(fusion_error("FuseEdges : Fusion failed"));
    }
    geom_lib::extend_curve_to_point(&mut ext_c, BRepTool::pnt(vf)?, 1, false)?;
    geom_lib::extend_curve_to_point(&mut ext_c, BRepTool::pnt(vl)?, 1, true)?;
    maker.init_curve_vertices(&ext_c.cast(), vf, vl)?;
    if !maker.is_done() {
        return Err(fusion_error("FuseEdges : Fusion failed"));
    }
    Ok(maker)
}

/// "closed edge": the B-spline must be closed; its origin is moved to the vertex when
/// its start is away from it, then the edge spans the whole curve.
fn closed_edge(
    maker: &mut BRepLibMakeEdge,
    c: &mut Handle<dyn GeomCurve>,
    vf: &Vertex,
    vl: &Vertex,
    (f, l): (&mut f64, &mut f64),
) -> Result<(), Failure> {
    let curve = c.get().expect("a curve");
    (*f, *l) = (curve.first_parameter(), curve.last_parameter());
    let (pf_curve, pl_curve) = (curve.value(*f)?, curve.value(*l)?);
    if pf_curve.distance(&pl_curve) > precision::confusion() {
        return Err(fusion_error("FuseEdges : Curve must be closed"));
    }
    let pf = BRepTool::pnt(vf)?;
    if pf.distance(&pf_curve) > precision::confusion() {
        make_closed_curve(c, pf, f, l)?;
    }
    maker.init_curve_vertices_params(c, vf, vl, *f, *l)?;
    if !maker.is_done() {
        return Err(fusion_error("FuseEdges : MakeEdge failed for closed curve"));
    }
    Ok(())
}
