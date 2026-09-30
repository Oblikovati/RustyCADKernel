// Rust translation of BRepExtrema_DistanceSS::Perform for edge-edge, edge-face and
// face-face (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The distance overloads of `BRepExtrema_DistanceSS` between edges and faces.
use super::SearchState;
use super::c0::C0Search;
use super::helpers::{is_in, is_on_boundary, min_solution, tri_solution};
use crate::brep_builder_api::BRepBuilderAPIMakeVertex;
use crate::brep_class::BRepClassFaceClassifier;
use crate::brep_extrema::solution_elem::{in_face, on_edge};
use crate::brep_extrema::{
    BRepExtremaExtCC, BRepExtremaExtCF, BRepExtremaExtFF, BRepExtremaExtPF, BRepExtremaSolutionElem,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::Point3;

type Solutions = Vec<BRepExtremaSolutionElem>;

/// The number of extrema of a wrapper that is done and not parallel, else 0.
fn nb_non_parallel(
    is_done: bool,
    is_parallel: impl FnOnce() -> Result<bool, Failure>,
    nb_ext: impl FnOnce() -> Result<i32, Failure>,
) -> Result<i32, Failure> {
    if !is_done || is_parallel()? {
        return Ok(0);
    }
    nb_ext()
}

/// The C1 break parameters of the edge curve of `adaptor` (`NbIntervals` + 1 values).
fn c1_breaks(adaptor: &GeomAdaptorCurve) -> Result<Vec<f64>, Failure> {
    let count: i32 = adaptor.nb_intervals(Continuity::C1)?;
    let mut breaks: Array1<'static, f64> = Array1::with_bounds(1, 1 + count)
        .map_err(|error| error.into_failure("BRepExtrema_DistanceSS::Perform(Edge, Face)"))?;
    adaptor.intervals(&mut breaks, Continuity::C1)?;
    Ok(breaks.as_slice().to_vec())
}

impl SearchState {
    /// `Perform(theS1, theS2, ...)` for two edges: the nearest non-parallel extrema that
    /// are new on one edge at least and away from the vertices of both, then the
    /// solutions of `PERFORM_C0` at the minimum distance that are new.
    pub(super) fn edge_edge(
        &mut self,
        s1: &Edge,
        s2: &Edge,
        seq1: &mut Solutions,
        seq2: &mut Solutions,
    ) -> Result<(), Failure> {
        if BRepTool::degenerated(s1)? || BRepTool::degenerated(s2)? {
            return Ok(());
        }
        // PERFORM_C0 gets the reference of the entry, before the extrema lower it.
        let dst_ref: f64 = self.dst_ref;
        let ext = BRepExtremaExtCC::with_edges(s1, s2)?;
        let nb_extrema: i32 =
            nb_non_parallel(ext.is_done(), || ext.is_parallel(), || ext.nb_ext())?;
        self.edge_edge_extrema(&ext, nb_extrema, (s1, s2), (seq1, seq2))?;
        self.edge_edge_c0((s1, s2), dst_ref, (seq1, seq2))
    }

    /// The `PERFORM_C0` part of the edge-edge overload: its solutions at the minimum
    /// distance that are new on one edge at least.
    fn edge_edge_c0(
        &mut self,
        (s1, s2): (&Edge, &Edge),
        dst_ref: f64,
        (seq1, seq2): (&mut Solutions, &mut Solutions),
    ) -> Result<(), Failure> {
        let (mut solutions1, mut solutions2) = (Vec::new(), Vec::new());
        let mut c0 = C0Search {
            seq_sol1: &mut solutions1,
            seq_sol2: &mut solutions2,
            dst_ref,
            m_dst_ref: &mut self.dst_ref,
            eps: self.eps,
        };
        c0.perform(s1, s2)?;
        let (mut seq_sol1, mut seq_sol2) = (Vec::new(), Vec::new());
        let solutions = (solutions1.as_slice(), solutions2.as_slice());
        min_solution(
            solutions,
            self.dst_ref,
            self.eps,
            &mut seq_sol1,
            &mut seq_sol2,
        );
        for (sol1, sol2) in seq_sol1.into_iter().zip(seq_sol2) {
            if tri_solution(seq1, sol1.point()) || tri_solution(seq2, sol2.point()) {
                seq1.push(sol1);
                seq2.push(sol2);
                self.modif = true;
            }
        }
        Ok(())
    }

    /// The extrema part of the edge-edge overload.
    fn edge_edge_extrema(
        &mut self,
        ext: &BRepExtremaExtCC,
        nb_extrema: i32,
        (s1, s2): (&Edge, &Edge),
        (seq1, seq2): (&mut Solutions, &mut Solutions),
    ) -> Result<(), Failure> {
        let Some((dst_min, indices)) =
            self.nearest_extrema(nb_extrema, |i| ext.square_distance(i))?
        else {
            return Ok(());
        };
        let eps_p: f64 = precision::p_confusion();
        for i in indices {
            let pt1: Point3 = ext.point_on_e1(i)?;
            let pt2: Point3 = ext.point_on_e2(i)?;
            if !(tri_solution(seq1, &pt1) || tri_solution(seq2, &pt2)) {
                continue;
            }
            // Check if the parameters do not correspond to a vertex.
            let t1: f64 = ext.parameter_on_e1(i)?;
            let t2: f64 = ext.parameter_on_e2(i)?;
            if !is_on_boundary(s1, &pt1, t1, eps_p)? && !is_on_boundary(s2, &pt2, t2, eps_p)? {
                let pair = (on_edge(dst_min, pt1, s1, t1), on_edge(dst_min, pt2, s2, t2));
                self.push_pair(dst_min, pair, (seq1, seq2));
            }
        }
        Ok(())
    }

    /// `Perform(theS1, theS2, ...)` for an edge and a face: the nearest non-parallel
    /// extrema that are new on one side at least, away from the edge vertices and
    /// strictly inside the face; then, for a C0 3D curve, the extrema from its C1 break
    /// points to the face at the minimum distance.
    pub(super) fn edge_face(
        &mut self,
        s1: &Edge,
        s2: &Face,
        seq1: &mut Solutions,
        seq2: &mut Solutions,
    ) -> Result<(), Failure> {
        if BRepTool::degenerated(s1)? {
            return Ok(());
        }
        let mut classifier = BRepClassFaceClassifier::new();
        let ext = BRepExtremaExtCF::with_edge_face(s1, s2)?;
        let nb_extrema: i32 =
            nb_non_parallel(ext.is_done(), || ext.is_parallel(), || Ok(ext.nb_ext()))?;
        self.edge_face_extrema(&ext, nb_extrema, (s1, s2), &mut classifier, (seq1, seq2))?;
        let (curve, first, last) = BRepTool::curve(s1)?;
        let Some(geometry) = curve.get() else {
            return Ok(());
        };
        if geometry.continuity() != Continuity::C0 {
            return Ok(());
        }
        let adaptor = GeomAdaptorCurve::with_range(&curve, first, last)?;
        self.edge_face_breaks((s1, s2), &adaptor, &mut classifier, (seq1, seq2))
    }

    /// The extrema part of the edge-face overload.
    fn edge_face_extrema(
        &mut self,
        ext: &BRepExtremaExtCF,
        nb_extrema: i32,
        (s1, s2): (&Edge, &Face),
        classifier: &mut BRepClassFaceClassifier,
        (seq1, seq2): (&mut Solutions, &mut Solutions),
    ) -> Result<(), Failure> {
        let Some((dst_min, indices)) =
            self.nearest_extrema(nb_extrema, |i| ext.square_distance(i))?
        else {
            return Ok(());
        };
        let tol: f64 = BRepTool::face_tolerance(s2)?;
        for i in indices {
            let pt1: Point3 = ext.point_on_edge(i)?;
            let pt2: Point3 = ext.point_on_face(i)?;
            if !(tri_solution(seq1, &pt1) || tri_solution(seq2, &pt2)) {
                continue;
            }
            // Check if the parameter does not correspond to a vertex.
            let t1: f64 = ext.parameter_on_edge(i)?;
            if is_on_boundary(s1, &pt1, t1, precision::p_confusion())? {
                continue;
            }
            let uv: (f64, f64) = ext.parameter_on_face(i)?;
            if is_in(classifier, s2, uv, tol)? {
                let pair = (on_edge(dst_min, pt1, s1, t1), in_face(dst_min, pt2, s2, uv));
                self.push_pair(dst_min, pair, (seq1, seq2));
            }
        }
        Ok(())
    }

    /// The C0 part of the edge-face overload: the vertex-face extrema of each C1 break
    /// point of the edge curve (default flag and algorithm), strictly inside the face,
    /// then those at the minimum distance.
    fn edge_face_breaks(
        &mut self,
        (s1, s2): (&Edge, &Face),
        adaptor: &GeomAdaptorCurve,
        classifier: &mut BRepClassFaceClassifier,
        (seq1, seq2): (&mut Solutions, &mut Solutions),
    ) -> Result<(), Failure> {
        let breaks: Vec<f64> = c1_breaks(adaptor)?;
        let tol: f64 = BRepTool::face_tolerance(s2)?;
        let (mut solutions1, mut solutions2): (Solutions, Solutions) = (Vec::new(), Vec::new());
        for parameter in breaks {
            let point: Point3 = adaptor.value(parameter)?;
            let solutions = (&mut solutions1, &mut solutions2);
            self.break_point((s1, s2), (parameter, point), (classifier, tol), solutions)?;
        }
        let solutions = (solutions1.as_slice(), solutions2.as_slice());
        min_solution(solutions, self.dst_ref, self.eps, seq1, seq2);
        Ok(())
    }

    /// The nearest vertex-face extrema of one break point `(parameter, point)` of the
    /// edge `s1`, strictly inside the face `s2`.
    fn break_point(
        &mut self,
        (s1, s2): (&Edge, &Face),
        (parameter, point): (f64, Point3),
        (classifier, tol): (&mut BRepClassFaceClassifier, f64),
        (solutions1, solutions2): (&mut Solutions, &mut Solutions),
    ) -> Result<(), Failure> {
        let vertex = BRepBuilderAPIMakeVertex::new(point)?.vertex()?;
        let ext = BRepExtremaExtPF::with_vertex_face(&vertex, s2, ExtFlag::MinMax, ExtAlgo::Grad)?;
        let nb: i32 = if ext.is_done() { ext.nb_ext() } else { 0 };
        let Some((dst_min, indices)) = self.nearest_extrema(nb, |i| ext.square_distance(i))? else {
            return Ok(());
        };
        for ii in indices {
            // Check if the parameter does not correspond to a vertex.
            let uv: (f64, f64) = ext.parameter(ii)?;
            if is_in(classifier, s2, uv, tol)? {
                let sol1 = on_edge(dst_min, point, s1, parameter);
                let sol2 = in_face(dst_min, ext.point(ii)?, s2, uv);
                self.push_pair(dst_min, (sol1, sol2), (solutions1, solutions2));
            }
        }
        Ok(())
    }

    /// `Perform(theS1, theS2, ...)` for two faces: the nearest non-parallel extrema that
    /// are new on one face at least and strictly inside both faces.
    pub(super) fn face_face(
        &mut self,
        s1: &Face,
        s2: &Face,
        seq1: &mut Solutions,
        seq2: &mut Solutions,
    ) -> Result<(), Failure> {
        let ext = BRepExtremaExtFF::with_faces(s1, s2)?;
        let nb_extrema: i32 =
            nb_non_parallel(ext.is_done(), || ext.is_parallel(), || Ok(ext.nb_ext()))?;
        let Some((dst_min, indices)) =
            self.nearest_extrema(nb_extrema, |i| ext.square_distance(i))?
        else {
            return Ok(());
        };
        let tols: (f64, f64) = (BRepTool::face_tolerance(s1)?, BRepTool::face_tolerance(s2)?);
        let mut classifier = BRepClassFaceClassifier::new();
        for i in indices {
            let pt1: Point3 = ext.point_on_face1(i)?;
            let pt2: Point3 = ext.point_on_face2(i)?;
            if !(tri_solution(seq1, &pt1) || tri_solution(seq2, &pt2)) {
                continue;
            }
            // Check if the parameter does not correspond to a vertex.
            let uv1: (f64, f64) = ext.parameter_on_face1(i)?;
            if !is_in(&mut classifier, s1, uv1, tols.0)? {
                continue;
            }
            let uv2: (f64, f64) = ext.parameter_on_face2(i)?;
            if is_in(&mut classifier, s2, uv2, tols.1)? {
                let pair = (
                    in_face(dst_min, pt1, s1, uv1),
                    in_face(dst_min, pt2, s2, uv2),
                );
                self.push_pair(dst_min, pair, (seq1, seq2));
            }
        }
        Ok(())
    }
}
