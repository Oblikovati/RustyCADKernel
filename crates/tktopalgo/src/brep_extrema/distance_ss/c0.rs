// Rust translation of the file-static helper PERFORM_C0 of
// ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extra solutions of an edge-edge distance at the C1 breaks of a C0 curve.
use super::helpers::{accepts, min_of, near, tri_solution};
use crate::brep_builder_api::BRepBuilderAPIMakeVertex;
use crate::brep_extrema::{BRepExtremaExtPC, BRepExtremaSolutionElem, BRepExtremaSupportType};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

/// The solution sequences of `PERFORM_C0` and the distances it compares with.
pub(super) struct C0Search<'a> {
    pub(super) seq_sol1: &'a mut Vec<BRepExtremaSolutionElem>,
    pub(super) seq_sol2: &'a mut Vec<BRepExtremaSolutionElem>,
    /// `DstRef`: the reference distance given on entry.
    pub(super) dst_ref: f64,
    /// `mDstRef`: the running minimum, lowered by each solution.
    pub(super) m_dst_ref: &'a mut f64,
    pub(super) eps: f64,
}

/// The C1 interval bounds of `curve` on `[first, last]` (`NbIntervals` + 1 values).
fn c1_breaks(adaptor: &GeomAdaptorCurve) -> Result<Vec<f64>, Failure> {
    let count: i32 = adaptor.nb_intervals(Continuity::C1)?;
    let mut bounds: Array1<'static, f64> = Array1::with_bounds(1, 1 + count)
        .map_err(|error| error.into_failure("BRepExtrema_DistanceSS PERFORM_C0"))?;
    adaptor.intervals(&mut bounds, Continuity::C1)?;
    Ok(bounds.as_slice().to_vec())
}

/// One side of the search: the edge whose curve is C0 and the other edge.
struct Side<'e> {
    e: &'e Edge,
    e_other: &'e Edge,
    /// True for `iE == 0` (E is S1).
    first: bool,
}

impl C0Search<'_> {
    /// `PERFORM_C0(S1, S2, SeqSol1, SeqSol2, DstRef, mDstRef, Eps)`: for each edge whose
    /// 3D curve is only C0 (first S1, then S2), the extrema from each of its C1 break
    /// points to the other edge and the distances to the other edge's C1 breaks. Stops
    /// at the first edge without a 3D curve, as natively; degenerated edges give nothing.
    pub(super) fn perform(&mut self, s1: &Edge, s2: &Edge) -> Result<(), Failure> {
        if BRepTool::degenerated(s1)? || BRepTool::degenerated(s2)? {
            return Ok(());
        }
        for (e, e_other, first) in [(s1, s2, true), (s2, s1, false)] {
            let (curve, a_first, a_last) = BRepTool::curve(e)?;
            let (curve_other, f_other, l_other) = BRepTool::curve(e_other)?;
            let Some(geometry) = curve.get() else {
                return Ok(());
            };
            if geometry.continuity() != Continuity::C0 {
                continue;
            }
            let side = Side { e, e_other, first };
            let adaptor = GeomAdaptorCurve::with_range(&curve, a_first, a_last)?;
            self.side(&side, &adaptor, (&curve_other, f_other, l_other))?;
        }
        Ok(())
    }

    /// The loop over the C1 breaks of one C0 edge.
    fn side(
        &mut self,
        side: &Side<'_>,
        adaptor: &GeomAdaptorCurve,
        (curve_other, f_other, l_other): (&Handle<dyn GeomCurve>, f64, f64),
    ) -> Result<(), Failure> {
        let breaks: Vec<f64> = c1_breaks(adaptor)?;
        // The other adaptor is built after the breaks of this one, as natively.
        let other = GeomAdaptorCurve::with_range(curve_other, f_other, l_other)?;
        let breaks_other: Vec<f64> = c1_breaks(&other)?;
        let range_other: (f64, f64) = BRepTool::range(side.e_other)?;
        for parameter in breaks {
            let point: Point3 = adaptor.value(parameter)?;
            let vertex: Vertex = BRepBuilderAPIMakeVertex::new(point)?.vertex()?;
            let ext = BRepExtremaExtPC::with_vertex_edge(&vertex, side.e_other)?;
            self.projections(side, (parameter, point), &ext, range_other)?;
            for parameter_other in &breaks_other {
                let point_other: Point3 = other.value(*parameter_other)?;
                let dst: f64 = point.distance(&point_other);
                if accepts(dst, self.dst_ref, self.eps) {
                    let sol1 = (dst, point, side.e, parameter);
                    self.append(side, sol1, (point_other, side.e_other, *parameter_other));
                }
            }
        }
        Ok(())
    }

    /// The extrema of one break point `(parameter, point)` on the other edge: the
    /// nearest ones not already found (on the other edge) and not at a vertex.
    fn projections(
        &mut self,
        side: &Side<'_>,
        (parameter, point): (f64, Point3),
        ext: &BRepExtremaExtPC,
        (udeb, ufin): (f64, f64),
    ) -> Result<(), Failure> {
        let nb_extrema: i32 = if ext.is_done() { ext.nb_ext()? } else { 0 };
        if nb_extrema <= 0 {
            return Ok(());
        }
        let dst_min: f64 = min_of(nb_extrema, |i| ext.square_distance(i))?.sqrt();
        if !accepts(dst_min, self.dst_ref, self.eps) {
            return Ok(());
        }
        let eps_p: f64 = precision::p_confusion();
        for ii in 1..=nb_extrema {
            if !near(dst_min, ext.square_distance(ii)?.sqrt(), self.eps) {
                continue;
            }
            let pt: Point3 = ext.point(ii)?;
            // Pt is on the other edge: S2 for the first side, S1 for the second one.
            let known = if side.first {
                &*self.seq_sol2
            } else {
                &*self.seq_sol1
            };
            if !tri_solution(known, &pt) {
                continue;
            }
            let t: f64 = ext.parameter(ii)?;
            if (t - udeb).abs() >= eps_p && (t - ufin).abs() > eps_p {
                self.append(
                    side,
                    (dst_min, point, side.e, parameter),
                    (pt, side.e_other, t),
                );
            }
        }
        Ok(())
    }

    /// Lowers the running minimum to `dst` and appends the pair of edge solutions, in
    /// the order of S1 and S2.
    fn append(
        &mut self,
        side: &Side<'_>,
        (dst, point, e, t): (f64, Point3, &Edge, f64),
        (pt, e_other, t_other): (Point3, &Edge, f64),
    ) {
        if *self.m_dst_ref > dst {
            *self.m_dst_ref = dst;
        }
        let sol1 =
            BRepExtremaSolutionElem::with_edge(dst, point, BRepExtremaSupportType::IsOnEdge, e, t);
        let sol2 = BRepExtremaSolutionElem::with_edge(
            dst,
            pt,
            BRepExtremaSupportType::IsOnEdge,
            e_other,
            t_other,
        );
        let (first, second) = if side.first {
            (sol1, sol2)
        } else {
            (sol2, sol1)
        };
        self.seq_sol1.push(first);
        self.seq_sol2.push(second);
    }
}
