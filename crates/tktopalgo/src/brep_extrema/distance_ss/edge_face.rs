// Rust translation of BRepExtrema_DistanceSS::Perform for edge-edge, edge-face and
// face-face (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The distance overloads of `BRepExtrema_DistanceSS` between edges and faces.
use super::SearchState;
use super::c0::C0Search;
use super::helpers::{is_on_boundary, min_of, min_solution, near, tri_solution};
use crate::brep_builder_api::BRepBuilderAPIMakeVertex;
use crate::brep_class::BRepClassFaceClassifier;
use crate::brep_extrema::{
    BRepExtremaExtCC, BRepExtremaExtCF, BRepExtremaExtFF, BRepExtremaExtPF,
    BRepExtremaSolutionElem, BRepExtremaSupportType,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point2, Point3};

type Solutions = Vec<BRepExtremaSolutionElem>;

/// Whether the face classifier puts `(u, v)` strictly inside `face`.
fn is_in(
    classifier: &mut BRepClassFaceClassifier,
    face: &Face,
    (u, v): (f64, f64),
    tol: f64,
) -> Result<bool, Failure> {
    classifier.perform(face, &Point2::new(u, v), tol, false, 0.1)?;
    Ok(classifier.state() == State::In)
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
        let dst_ref: f64 = self.dst_ref;
        let ext = BRepExtremaExtCC::with_edges(s1, s2)?;
        let nb_extrema: i32 = match ext.is_done() {
            true if ext.is_parallel()? => 0,
            true => ext.nb_ext()?,
            false => 0,
        };
        if nb_extrema > 0 {
            self.edge_edge_extrema(&ext, nb_extrema, (s1, s2), (seq1, seq2))?;
        }
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
        if !solutions1.is_empty() && !solutions2.is_empty() {
            min_solution(
                (&solutions1, &solutions2),
                self.dst_ref,
                self.eps,
                &mut seq_sol1,
                &mut seq_sol2,
            );
        }
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
        let dst_min: f64 = min_of(nb_extrema, |i| ext.square_distance(i))?.sqrt();
        if !self.accepts(dst_min) {
            return Ok(());
        }
        let eps_p: f64 = precision::p_confusion();
        for i in 1..=nb_extrema {
            if !near(dst_min, ext.square_distance(i)?.sqrt(), self.eps) {
                continue;
            }
            let pt1: Point3 = ext.point_on_e1(i)?;
            let pt2: Point3 = ext.point_on_e2(i)?;
            if !(tri_solution(seq1, &pt1) || tri_solution(seq2, &pt2)) {
                continue;
            }
            // Check if the parameters do not correspond to a vertex.
            let t1: f64 = ext.parameter_on_e1(i)?;
            let t2: f64 = ext.parameter_on_e2(i)?;
            if !is_on_boundary(s1, &pt1, t1, eps_p)? && !is_on_boundary(s2, &pt2, t2, eps_p)? {
                self.improve(dst_min);
                seq1.push(BRepExtremaSolutionElem::with_edge(
                    dst_min,
                    pt1,
                    BRepExtremaSupportType::IsOnEdge,
                    s1,
                    t1,
                ));
                seq2.push(BRepExtremaSolutionElem::with_edge(
                    dst_min,
                    pt2,
                    BRepExtremaSupportType::IsOnEdge,
                    s2,
                    t2,
                ));
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
        let nb_extrema: i32 = match ext.is_done() {
            true if ext.is_parallel()? => 0,
            true => ext.nb_ext(),
            false => 0,
        };
        if nb_extrema > 0 {
            self.edge_face_extrema(&ext, nb_extrema, (s1, s2), &mut classifier, (seq1, seq2))?;
        }
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
        let dst_min: f64 = min_of(nb_extrema, |i| ext.square_distance(i))?.sqrt();
        if !self.accepts(dst_min) {
            return Ok(());
        }
        let tol: f64 = BRepTool::face_tolerance(s2)?;
        let eps_p: f64 = precision::p_confusion();
        for i in 1..=nb_extrema {
            if !near(dst_min, ext.square_distance(i)?.sqrt(), self.eps) {
                continue;
            }
            let pt1: Point3 = ext.point_on_edge(i)?;
            let pt2: Point3 = ext.point_on_face(i)?;
            if !(tri_solution(seq1, &pt1) || tri_solution(seq2, &pt2)) {
                continue;
            }
            // Check if the parameter does not correspond to a vertex.
            let t1: f64 = ext.parameter_on_edge(i)?;
            if is_on_boundary(s1, &pt1, t1, eps_p)? {
                continue;
            }
            let (u, v) = ext.parameter_on_face(i)?;
            if is_in(classifier, s2, (u, v), tol)? {
                self.improve(dst_min);
                seq1.push(BRepExtremaSolutionElem::with_edge(
                    dst_min,
                    pt1,
                    BRepExtremaSupportType::IsOnEdge,
                    s1,
                    t1,
                ));
                seq2.push(BRepExtremaSolutionElem::with_face(
                    dst_min,
                    pt2,
                    BRepExtremaSupportType::IsInFace,
                    s2,
                    u,
                    v,
                ));
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
        let count: i32 = adaptor.nb_intervals(Continuity::C1)?;
        let mut breaks: Array1<'static, f64> = Array1::with_bounds(1, 1 + count)
            .map_err(|error| error.into_failure("BRepExtrema_DistanceSS::Perform(Edge, Face)"))?;
        adaptor.intervals(&mut breaks, Continuity::C1)?;
        let tol: f64 = BRepTool::face_tolerance(s2)?;
        let (mut solutions1, mut solutions2): (Solutions, Solutions) = (Vec::new(), Vec::new());
        for parameter in breaks.as_slice().to_vec() {
            let point: Point3 = adaptor.value(parameter)?;
            let vertex = BRepBuilderAPIMakeVertex::new(point)?.vertex()?;
            let ext =
                BRepExtremaExtPF::with_vertex_face(&vertex, s2, ExtFlag::MinMax, ExtAlgo::Grad)?;
            let nb: i32 = if ext.is_done() { ext.nb_ext() } else { 0 };
            if nb <= 0 {
                continue;
            }
            let dst_min: f64 = min_of(nb, |i| ext.square_distance(i))?.sqrt();
            if !self.accepts(dst_min) {
                continue;
            }
            for ii in 1..=nb {
                if !near(dst_min, ext.square_distance(ii)?.sqrt(), self.eps) {
                    continue;
                }
                // Check if the parameter does not correspond to a vertex.
                let (u, v) = ext.parameter(ii)?;
                if !is_in(classifier, s2, (u, v), tol)? {
                    continue;
                }
                self.improve(dst_min);
                solutions1.push(BRepExtremaSolutionElem::with_edge(
                    dst_min,
                    point,
                    BRepExtremaSupportType::IsOnEdge,
                    s1,
                    parameter,
                ));
                solutions2.push(BRepExtremaSolutionElem::with_face(
                    dst_min,
                    ext.point(ii)?,
                    BRepExtremaSupportType::IsInFace,
                    s2,
                    u,
                    v,
                ));
            }
        }
        if !solutions1.is_empty() && !solutions2.is_empty() {
            min_solution(
                (&solutions1, &solutions2),
                self.dst_ref,
                self.eps,
                seq1,
                seq2,
            );
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
        let nb_extrema: i32 = match ext.is_done() {
            true if ext.is_parallel()? => 0,
            true => ext.nb_ext(),
            false => 0,
        };
        if nb_extrema <= 0 {
            return Ok(());
        }
        let dst_min: f64 = min_of(nb_extrema, |i| ext.square_distance(i))?.sqrt();
        if !self.accepts(dst_min) {
            return Ok(());
        }
        let tol1: f64 = BRepTool::face_tolerance(s1)?;
        let tol2: f64 = BRepTool::face_tolerance(s2)?;
        let mut classifier = BRepClassFaceClassifier::new();
        for i in 1..=nb_extrema {
            if !near(dst_min, ext.square_distance(i)?.sqrt(), self.eps) {
                continue;
            }
            let pt1: Point3 = ext.point_on_face1(i)?;
            let pt2: Point3 = ext.point_on_face2(i)?;
            if !(tri_solution(seq1, &pt1) || tri_solution(seq2, &pt2)) {
                continue;
            }
            // Check if the parameter does not correspond to a vertex.
            let (u1, v1) = ext.parameter_on_face1(i)?;
            if !is_in(&mut classifier, s1, (u1, v1), tol1)? {
                continue;
            }
            let (u2, v2) = ext.parameter_on_face2(i)?;
            if is_in(&mut classifier, s2, (u2, v2), tol2)? {
                self.improve(dst_min);
                seq1.push(BRepExtremaSolutionElem::with_face(
                    dst_min,
                    pt1,
                    BRepExtremaSupportType::IsInFace,
                    s1,
                    u1,
                    v1,
                ));
                seq2.push(BRepExtremaSolutionElem::with_face(
                    dst_min,
                    pt2,
                    BRepExtremaSupportType::IsInFace,
                    s2,
                    u2,
                    v2,
                ));
            }
        }
        Ok(())
    }
}
