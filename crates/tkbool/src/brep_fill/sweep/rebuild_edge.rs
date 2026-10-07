// Rust translation of BRepFill_Sweep::RebuildTopOrBottomEdge
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_Sweep.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The rebuilding of an edge of a bounding section with the curves of the swept
//! surfaces.
use super::{BRepFillSweep, null};
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::toploc::Location;

impl BRepFillSweep {
    /// `RebuildTopOrBottomEdge(aNewEdge, anEdge, ReversedEdges)` (protected): a v-iso
    /// edge of the top or bottom section given the curves of the new edge (its 3d curve
    /// and p-curves, its range, tolerance and degeneracy), reversed (and recorded) when
    /// the new edge runs the other way.
    /// `sweep.rebuild_top_or_bottom_edge(&new_edge, &mut edge, &mut reversed)?`.
    pub fn rebuild_top_or_bottom_edge(
        &self,
        a_new_edge: &Edge,
        an_edge: &mut Edge,
        reversed_edges: &mut MapOfShape,
    ) -> Result<(), Failure> {
        let (a_new_curve, mut fpar, mut lpar) = BRepTool::curve(a_new_edge)?;
        let is_degen: bool = BRepTool::degenerated(a_new_edge)?;
        let to_reverse: bool = if is_degen {
            (fpar, lpar) = BRepTool::range(a_new_edge)?;
            false
        } else {
            runs_reversed(a_new_edge, an_edge, (&a_new_curve, fpar, lpar))?
        };
        an_edge.set_location(&Location::default(), false)?;
        edge_tshape(an_edge, "BRepFill_Sweep::RebuildTopOrBottomEdge")?
            .set_tolerance(BRepTool::edge_tolerance(a_new_edge)?);
        let builder = BRepBuilder::default();
        builder.range(an_edge, fpar, lpar, false)?;
        builder.update_edge_curve(an_edge, &a_new_curve, precision::confusion())?;
        for curve_rep in edge_tshape(a_new_edge, "BRepFill_Sweep::RebuildTopOrBottomEdge")?.curves()
        {
            let Some(rep) = curve_rep.get() else { continue };
            if rep.is_curve_on_surface() {
                let a_loc: Location = a_new_edge.location().multiplied(&rep.location())?;
                builder.update_edge_pcurve(
                    an_edge,
                    &rep.pcurve()?,
                    &rep.surface()?,
                    &a_loc,
                    precision::confusion(),
                    None,
                )?;
            }
        }
        reattach_vertices(an_edge, to_reverse)?;
        if to_reverse {
            an_edge.reverse();
            reversed_edges.add(an_edge.as_ref().clone());
        }
        builder.degenerated(an_edge, is_degen)
    }
}

/// Whether the new edge runs against the old one: by its vertices (the first old vertex,
/// without location, is neither the first new one nor at its point, or it is the last
/// new one), or for a closed old edge by the tangents at the middles.
fn runs_reversed(
    a_new_edge: &Edge,
    an_edge: &Edge,
    (a_new_curve, fpar, lpar): (
        &rustycad_tkernel::standard::Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
        f64,
        f64,
    ),
) -> Result<bool, Failure> {
    let (mut v1, v2) = topexp::vertices(an_edge, false);
    if v1.is_same(&v2) {
        let (old_curve, old_first, old_last) = BRepTool::curve(an_edge)?;
        let (_, old_d1) = old_curve
            .get()
            .ok_or_else(|| null("curve"))?
            .d1(0.5 * (old_first + old_last))?;
        let (_, new_d1) = a_new_curve
            .get()
            .ok_or_else(|| null("curve"))?
            .d1(0.5 * (fpar + lpar))?;
        return Ok(old_d1.dot(new_d1) < 0.);
    }
    let (new_v1, new_v2) = topexp::vertices(a_new_edge, false);
    v1.set_location(&Location::default(), false)?;
    if v1.is_same(&new_v1) {
        return Ok(false);
    }
    if v1.is_same(&new_v2) {
        return Ok(true);
    }
    let tol_sum: f64 = BRepTool::vertex_tolerance(&v1)? + BRepTool::vertex_tolerance(&new_v1)?;
    Ok(!BRepTool::pnt(&v1)?.is_equal(&BRepTool::pnt(&new_v1)?, tol_sum))
}

/// The vertices of the edge re-added without location (exchanged in orientation when
/// the edge is to be reversed).
fn reattach_vertices(an_edge: &mut Edge, to_reverse: bool) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    an_edge.set_free(true)?;
    let (mut v1, mut v2) = topexp::vertices(an_edge, false);
    let mut an_edge_forward: Shape = an_edge.oriented(Orientation::Forward);
    builder.remove(&mut an_edge_forward, &v1)?;
    builder.remove(&mut an_edge_forward, &v2)?;
    v1.set_location(&Location::default(), false)?;
    v2.set_location(&Location::default(), false)?;
    if to_reverse {
        v2.set_orientation(Orientation::Forward);
        v1.set_orientation(Orientation::Reversed);
    }
    builder.add(&mut an_edge_forward, &v1)?;
    builder.add(&mut an_edge_forward, &v2)
}
