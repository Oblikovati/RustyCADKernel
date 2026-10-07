// Rust port of DataExchange/TKXSBase/XSAlgo/XSAlgo_ShapeProcessor.cxx (MakeEdgeOnCurve,
// CheckPCurve) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The check (and repair) of the p-curve of an edge on a face read from a file.
use super::XSAlgoShapeProcessor;
use crate::interface::InterfaceStatic;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp,
    topods::{self, Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::{geom::GeomSurface, top_abs::Orientation};
use rustycad_tkmath::gp::Point2;
use rustycad_tkshhealing::{
    shape_analysis::ShapeAnalysisEdge, shape_build::ShapeBuildEdge, shape_fix::ShapeFixEdge,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

/// A p-curve with its range.
type PCurve = (Handle<dyn Geom2dCurve>, f64, f64);

/// `MakeEdgeOnCurve` (private): a new edge on the 3D curve of `source` (null without one).
fn make_edge_on_curve(source: &Edge) -> Result<Edge, Failure> {
    let (curve, start, end) = ShapeAnalysisEdge::new().curve_3d(source, false)?;
    let Some(live) = curve.get() else {
        return Ok(Edge::default());
    };
    let (p1, p2) = (live.value(start)?, live.value(end)?);
    let maker = BRepBuilderAPIMakeEdge::from_curve_points_params(&curve, p1, p2, start, end)?;
    ShapeBuildEdge.set_range_3d(&maker.edge()?, start, end)?;
    maker.edge()
}

/// The p-curve spans more than 6/8 of the surface span (native rule kept as is).
fn spans_too_much(
    surface: &Handle<dyn GeomSurface>,
    uv1: Point2,
    uv2: Point2,
) -> Result<bool, Failure> {
    let live = surface.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_ShapeProcessor::CheckPCurve: face without surface",
        )
    })?;
    let (u1, u2, v1, v2) = live.bounds();
    let span_x = (uv1.x() - uv2.x()).abs();
    let span_y = (uv1.y() - uv2.y()).abs();
    Ok(span_x / 8. > (u2 / 6. - u1 / 6.) || span_y / 8. > (v2 / 6. - v1 / 6.))
}

/// The ends of the p-curve on the surface are farther than `precision` from the ends of
/// the 3D curve (or the vertices without one).
fn ends_mismatch(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    uv1: Point2,
    uv2: Point2,
    precision: f64,
) -> Result<bool, Failure> {
    let live = surface.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_ShapeProcessor::CheckPCurve: face without surface",
        )
    })?;
    let (curve, c1, c2) = ShapeAnalysisEdge::new().curve_3d(edge, false)?;
    let (s1, s2) = (live.value(uv1.x(), uv1.y())?, live.value(uv2.x(), uv2.y())?);
    let (pv1, pv2) = match curve.get() {
        Some(c3d) => (c3d.value(c1)?, c3d.value(c2)?),
        None => (
            BRepTool::pnt(&topexp::first_vertex(edge, false))?,
            BRepTool::pnt(&topexp::last_vertex(edge, false))?,
        ),
    };
    Ok(pv1.distance(&s1) > precision || pv2.distance(&s2) > precision)
}

/// The seam p-curve of the reversed edge (a copy of `pcurve` when missing or the same).
fn seam_pcurve(
    edge: &Edge,
    face: &Face,
    pcurve: &Handle<dyn Geom2dCurve>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let reversed = topods::edge(&edge.reversed())?;
    let (seam, _, _) = ShapeAnalysisEdge::new().pcurve_on_face(&reversed, face, false)?;
    if seam.is_some() && seam != *pcurve {
        return Ok(seam);
    }
    let live = pcurve.get().ok_or_else(|| {
        rustycad_tkernel::standard::exceptions::null_object(
            "XSAlgo_ShapeProcessor::CheckPCurve: null p-curve",
        )
    })?;
    Ok(live.copy()?.cast())
}

/// The temporary edge on the 3D curve with the p-curve(s), as `CheckPCurve` builds it.
fn temporary_edge(
    edge: &Edge,
    face: &Face,
    pcurve: &PCurve,
    is_seam: bool,
) -> Result<(Edge, Handle<dyn Geom2dCurve>), Failure> {
    let tmp = make_edge_on_curve(edge)?;
    let builder = BRepBuilder::default();
    let mut seam = Handle::default();
    if is_seam {
        seam = seam_pcurve(edge, face, &pcurve.0)?;
        builder.update_edge_pcurves_on_face(&tmp, (&pcurve.0, &seam), face, 0.)?;
    } else {
        builder.update_edge_pcurve_on_face(&tmp, &pcurve.0, face, 0.)?;
    }
    builder.range_on_face(&tmp, face, pcurve.1, pcurve.2)?;
    builder.same_range(&tmp, false)?;
    if InterfaceStatic::i_val("read.stdsameparameter.mode") != 0 {
        builder.same_parameter(&tmp, false)?;
    }
    Ok((tmp, seam))
}

/// The best fixed temporary edge: same-parameter fixed, else a recomputed p-curve when
/// the tolerance is too big or the range is not the same. Returns the edge, its
/// tolerance and its flags.
fn best_edge(
    edge: &Edge,
    face: &Face,
    tmp: Edge,
    is_seam: bool,
    precision: f64,
) -> Result<(Edge, f64, bool, bool), Failure> {
    let fixer = ShapeFixEdge::new();
    fixer.fix_same_parameter(&tmp, 0.)?;
    let mut best = (
        tmp.clone(),
        BRepTool::edge_tolerance(&tmp)?,
        BRepTool::same_range(&tmp)?,
        BRepTool::same_parameter(&tmp)?,
    );
    if best.1 > f64::min(1., 2. * precision) || !best.2 {
        let projected = make_edge_on_curve(edge)?;
        fixer.fix_add_pcurve(&projected, face, is_seam, precision)?;
        fixer.fix_same_parameter(&projected, 0.)?;
        let tolerance = BRepTool::edge_tolerance(&projected)?;
        if tolerance < best.1 || !best.2 {
            best = (
                projected.clone(),
                tolerance,
                BRepTool::same_range(&projected)?,
                BRepTool::same_parameter(&projected)?,
            );
        }
    }
    Ok(best)
}

impl XSAlgoShapeProcessor {
    /// `CheckPCurve(theEdge, theFace, thePrecision, theIsSeam)`: false (and the p-curve
    /// removed) for a p-curve longer than the surface or whose ends are farther than
    /// `precision` from the edge ends, false without 3D curve; else the p-curve is
    /// replaced by the best same-parameter one and true.
    pub fn check_pcurve(
        edge: &Edge,
        face: &Face,
        precision: f64,
        is_seam: bool,
    ) -> Result<bool, Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let pcurve: PCurve = analyzer.pcurve_on_face(edge, face, false)?;
        let Some(curve2d) = pcurve.0.get() else {
            return Ok(false);
        };
        let surface = BRepTool::surface(face)?;
        let (uv1, uv2) = (curve2d.value(pcurve.1)?, curve2d.value(pcurve.2)?);
        if spans_too_much(&surface, uv1, uv2)?
            || ends_mismatch(edge, &surface, uv1, uv2, precision)?
        {
            ShapeBuildEdge.remove_pcurve_on_face(edge, face)?;
            return Ok(false);
        }
        if analyzer.curve_3d(edge, false)?.0.is_null() {
            return Ok(false);
        }
        let (tmp, _) = temporary_edge(edge, face, &pcurve, is_seam)?;
        let (best, tolerance, same_range, same_parameter) =
            best_edge(edge, face, tmp, is_seam, precision)?;
        Self::put_back_pcurve(edge, face, &best, is_seam, tolerance)?;
        let builder = BRepBuilder::default();
        if BRepTool::same_range(edge)? {
            builder.same_range(edge, same_range)?;
        }
        if BRepTool::same_parameter(edge)? {
            builder.same_parameter(edge, same_parameter)?;
        }
        Ok(true)
    }
    /// The corrected p-curve(s) of the temporary edge put on `edge`, with the tolerance
    /// on the edge and its vertices and the range.
    fn put_back_pcurve(
        edge: &Edge,
        face: &Face,
        best: &Edge,
        is_seam: bool,
        tolerance: f64,
    ) -> Result<(), Failure> {
        let analyzer = ShapeAnalysisEdge::new();
        let builder = BRepBuilder::default();
        let (curve2d, first, last) = analyzer.pcurve_on_face(best, face, false)?;
        if is_seam {
            let (seam, _, _) =
                analyzer.pcurve_on_face(&topods::edge(&best.reversed())?, face, false)?;
            // abv 14.11.01 (coneEl.sat loop): a REVERSED edge takes the seam first
            let pair = if edge.orientation() == Orientation::Reversed {
                (&seam, &curve2d)
            } else {
                (&curve2d, &seam)
            };
            builder.update_edge_pcurves_on_face(edge, pair, face, tolerance)?;
        } else {
            builder.update_edge_pcurve_on_face(edge, &curve2d, face, tolerance)?;
        }
        builder.update_vertex_tolerance(&topexp::first_vertex(edge, false), tolerance)?;
        builder.update_vertex_tolerance(&topexp::last_vertex(edge, false), tolerance)?;
        builder.range_on_face(edge, face, first, last)
    }
}
