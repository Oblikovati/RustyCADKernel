// Rust translation of BRepExtrema_DistanceSS::Perform for vertex-vertex, vertex-edge and
// vertex-face (ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_DistanceSS.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The distance overloads of `BRepExtrema_DistanceSS` with a vertex.
use super::SearchState;
use super::helpers::{is_on_boundary, min_of, near, tri_solution};
use crate::brep_class::BRepClassFaceClassifier;
use crate::brep_extrema::{
    BRepExtremaExtPC, BRepExtremaExtPF, BRepExtremaSolutionElem, BRepExtremaSupportType,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{Point2, Point3};

type Solutions = Vec<BRepExtremaSolutionElem>;

impl SearchState {
    /// `Perform(S1, S2, ...)` for two vertices: their distance when accepted.
    pub(super) fn vertex_vertex(
        &mut self,
        s1: &Vertex,
        s2: &Vertex,
        seq1: &mut Solutions,
        seq2: &mut Solutions,
    ) -> Result<(), Failure> {
        let p1: Point3 = BRepTool::pnt(s1)?;
        let p2: Point3 = BRepTool::pnt(s2)?;
        let dst: f64 = p1.distance(&p2);
        if self.accepts(dst) {
            self.improve(dst);
            seq1.push(BRepExtremaSolutionElem::with_vertex(
                dst,
                p1,
                BRepExtremaSupportType::IsVertex,
                s1,
            ));
            seq2.push(BRepExtremaSolutionElem::with_vertex(
                dst,
                p2,
                BRepExtremaSupportType::IsVertex,
                s2,
            ));
        }
        Ok(())
    }

    /// `Perform(theS1, theS2, ...)` for a vertex and an edge: the nearest extrema of the
    /// vertex on the (non-degenerated) edge that are new and not at an edge vertex.
    pub(super) fn vertex_edge(
        &mut self,
        s1: &Vertex,
        s2: &Edge,
        seq1: &mut Solutions,
        seq2: &mut Solutions,
    ) -> Result<(), Failure> {
        if BRepTool::degenerated(s2)? {
            return Ok(());
        }
        let ext = BRepExtremaExtPC::with_vertex_edge(s1, s2)?;
        let nb_extrema: i32 = if ext.is_done() { ext.nb_ext()? } else { 0 };
        if nb_extrema <= 0 {
            return Ok(());
        }
        let dst_min: f64 = min_of(nb_extrema, |i| ext.square_distance(i))?.sqrt();
        if !self.accepts(dst_min) {
            return Ok(());
        }
        let p1: Point3 = BRepTool::pnt(s1)?;
        let eps_p: f64 = precision::p_confusion();
        for i in 1..=nb_extrema {
            if !near(dst_min, ext.square_distance(i)?.sqrt(), self.eps) {
                continue;
            }
            let pt: Point3 = ext.point(i)?;
            if !tri_solution(seq2, &pt) {
                continue;
            }
            // Check if the parameter does not correspond to a vertex.
            let t: f64 = ext.parameter(i)?;
            if !is_on_boundary(s2, &pt, t, eps_p)? {
                self.improve(dst_min);
                seq1.push(BRepExtremaSolutionElem::with_vertex(
                    dst_min,
                    p1,
                    BRepExtremaSupportType::IsVertex,
                    s1,
                ));
                seq2.push(BRepExtremaSolutionElem::with_edge(
                    dst_min,
                    pt,
                    BRepExtremaSupportType::IsOnEdge,
                    s2,
                    t,
                ));
            }
        }
        Ok(())
    }

    /// `Perform(theS1, theS2, ...)` for a vertex and a face: the nearest extrema of the
    /// vertex on the face (with the flag and algorithm of the search) that are new and
    /// strictly inside the face.
    pub(super) fn vertex_face(
        &mut self,
        s1: &Vertex,
        s2: &Face,
        seq1: &mut Solutions,
        seq2: &mut Solutions,
    ) -> Result<(), Failure> {
        let ext = BRepExtremaExtPF::with_vertex_face(s1, s2, self.flag, self.algo)?;
        let nb_extrema: i32 = if ext.is_done() { ext.nb_ext() } else { 0 };
        if nb_extrema <= 0 {
            return Ok(());
        }
        let dst_min: f64 = min_of(nb_extrema, |i| ext.square_distance(i))?.sqrt();
        if !self.accepts(dst_min) {
            return Ok(());
        }
        let p1: Point3 = BRepTool::pnt(s1)?;
        let mut classifier = BRepClassFaceClassifier::new();
        let tol: f64 = BRepTool::face_tolerance(s2)?;
        for i in 1..=nb_extrema {
            if !near(dst_min, ext.square_distance(i)?.sqrt(), self.eps) {
                continue;
            }
            let pt: Point3 = ext.point(i)?;
            if !tri_solution(seq2, &pt) {
                continue;
            }
            // Check if the parameter does not correspond to a vertex.
            let (u, v) = ext.parameter(i)?;
            classifier.perform(s2, &Point2::new(u, v), tol, false, 0.1)?;
            if classifier.state() == State::In {
                self.improve(dst_min);
                seq1.push(BRepExtremaSolutionElem::with_vertex(
                    dst_min,
                    p1,
                    BRepExtremaSupportType::IsVertex,
                    s1,
                ));
                seq2.push(BRepExtremaSolutionElem::with_face(
                    dst_min,
                    pt,
                    BRepExtremaSupportType::IsInFace,
                    s2,
                    u,
                    v,
                ));
            }
        }
        Ok(())
    }
}
