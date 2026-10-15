//! The ON parts of `TopOpeBRepBuild_Builder1` (new builder): the ON splits of the
//! section edges of a face kept by the side of their faces, the 2d pieces IN a same
//! domain face, and the orientation of an edge on the base face.
use super::SD_EDGE_MAP;
use super::same_dom_edge::{oriented_nearest_normal, oriented_normal, oriented_tangent};
use crate::top_ope_brep_build::TopOpeBRepBuildShapeSet;
use crate::top_ope_brep_build::builder::fun_aresamegeom;
use crate::top_ope_brep_build::tools::PAR_T;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::export::fds_data;
use crate::top_ope_brep_ds::{
    TopOpeBRepDSKind, TopOpeBRepDSShapeShapeInterference, TopOpeBRepDSTransition,
};
use crate::top_ope_brep_tool::fc2d_curve_on_surface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The static `ClassifyEdgeToFaceByOnePoint(E, F)`: the 2d state in the face of the
/// sample point of the p-curve of the edge (UNKNOWN without p-curve).
pub(crate) fn classify_edge_to_face_by_one_point(e: &Edge, f: &Face) -> Result<State, Failure> {
    let (c2d, f2, l2, _) = fc2d_curve_on_surface(e, f, true)?;
    let Some(c) = c2d.get() else {
        return Ok(State::Unknown);
    };
    let p2d = c.value(f2 * PAR_T + (1. - PAR_T) * l2)?;
    BRepTopAdaptorFClass2d::new(f, 1e-7)?.perform(p2d, true)
}

/// The edge of the face same as `eg`, else its last edge (null without edges), as the
/// native explorer loop leaves it.
fn edge_in_face(f: &Shape, eg: &Shape) -> Result<Shape, Failure> {
    let mut eor = Shape::default();
    for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let found = eg.is_same(&e);
        eor = e;
        if found {
            break;
        }
    }
    Ok(eor)
}

/// The scalar product of the binormal of the edge on a face (`aTg ^ aN2`) with a normal,
/// retried with the normal of the face moved inside; None when still zero.
fn binormal_side(
    face: &Shape,
    e: &Shape,
    n3: rustycad_tkmath::gp::Vector3,
) -> Result<Option<f64>, Failure> {
    let tg = oriented_tangent(e)?;
    let scalar = tg.crossed(oriented_normal(face, e)?).dot(n3);
    if scalar.abs() > 1e-10 {
        return Ok(Some(scalar));
    }
    let scalar = tg.crossed(oriented_nearest_normal(face, e)?).dot(n3);
    Ok((scalar.abs() > 1e-10).then_some(scalar))
}

impl TopOpeBRepBuildBuilder {
    /// Whether an ON split of the edge (oriented by its occurrences in the faces of the
    /// edge, left so) is kept: by the side of the face relative to the binormal of the
    /// split on one of the faces of the edge.
    fn on_part_kept(
        &self,
        new_e: &mut Shape,
        eg: &Shape,
        sd_face: &Shape,
        etb: State,
    ) -> Result<bool, Failure> {
        let edge_faces = self.b1d()?.my_map_of_edge_faces.find_from_key(eg)?.clone();
        let mut keep = false;
        for f in edge_faces.iter() {
            let eor = edge_in_face(f, eg)?;
            if eor.is_null() {
                continue;
            }
            new_e.set_orientation(eor.orientation());
            let Some(scalar) = binormal_side(f, new_e, oriented_normal(sd_face, new_e)?)? else {
                continue;
            };
            keep = (if scalar > 0. { State::In } else { State::Out }) == etb;
            if keep {
                break;
            }
        }
        Ok(keep)
    }

    /// Whether the face is oriented against the base face at the edge (inverted for a
    /// tool face in a cut).
    fn state_of_face_ori(&self, face: &Shape, base: &Shape, e: &Shape) -> Result<bool, Failure> {
        let mut st = oriented_normal(face, e)?.dot(oriented_normal(base, e)?) < 0.;
        if self.hds()?.ds().ancestor_rank(face) == 2 && (self.opec12() || self.opec21()) {
            st = !st;
        }
        Ok(st)
    }

    /// The kept ON splits of one face/edge interference.
    fn on_parts_of(
        &mut self,
        for1: &Shape,
        ssi: &TopOpeBRepDSShapeShapeInterference,
        gi: i32,
        g1: &TopOpeBRepBuildGTopo,
        ftb: State,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let eg = self.hds()?.ds().shape(gi, false).clone();
        if SD_EDGE_MAP.with_borrow(|m| m.contains(&eg)) {
            return Ok(());
        }
        let spl_on = self.hds()?.ds().get_shape_with_state(&eg).part(State::On);
        if spl_on.is_empty() {
            return Ok(());
        }
        let tr: TopOpeBRepDSTransition = ssi.transition();
        let (etb, _) = Self::rank_states(g1, self.hds()?.ds().ancestor_rank(&eg))?;
        let base = self.b1d()?.my_base_face_to_fill.clone();
        for e in spl_on.iter() {
            let mut new_e = e.clone();
            let st = classify_edge_to_face_by_one_point(
                &rustycad_tkbrep::topods::edge(&new_e)?,
                &rustycad_tkbrep::topods::face(for1)?,
            )?;
            if st != State::In && st != State::On {
                continue;
            }
            if !self.on_part_kept(&mut new_e, &eg, for1, etb)? {
                continue;
            }
            let state_of_face_ori = self.state_of_face_ori(for1, &base, &new_e)?;
            let mut neworie = tr.orientation(ftb, ShapeEnum::Face);
            if !fun_aresamegeom(for1, &base)? {
                neworie = top_abs::complement(neworie);
            }
            new_e.set_orientation(neworie);
            self.b1d_mut()?
                .my_map_of_edge_with_face_state
                .bind(new_e.clone(), state_of_face_ori);
            wes.add_start_element(&new_e)?;
        }
        Ok(())
    }

    /// `PerformONParts(FOR1, SDFaces, G1, WES)` (Builder1): the ON splits of the edges of
    /// the face/edge interferences of the face (edges out of the same domain faces) in
    /// the face, kept by the side of the face, oriented by the transition;
    /// `b.perform_on_parts(&f, &sd, &g, &mut wes)?`.
    pub fn perform_on_parts(
        &mut self,
        for1: &Shape,
        _sd_faces: &IndexedMapOfShape,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (ftb, _) = Self::rank_states(g1, self.hds()?.ds().ancestor_rank(for1))?;
        let li = self.hds()?.ds().shape_interferences(for1, true).clone();
        for i in li.iter() {
            let ssi: Handle<TopOpeBRepDSShapeShapeInterference> = i.clone().cast();
            let Some(s) = ssi.get() else {
                continue;
            };
            let (gt, gi, st, _) = fds_data(i);
            if gt != TopOpeBRepDSKind::Edge || st != TopOpeBRepDSKind::Face {
                continue;
            }
            self.on_parts_of(for1, s, gi, g1, ftb, wes)?;
        }
        Ok(())
    }

    /// The keep of a 2d piece by the side of the adjacent faces of the edge relative to
    /// the binormal of the piece on `to_face` (None when no side was found).
    fn piece_side_keep(
        &self,
        e: &Edge,
        eor: &Edge,
        edge_face: &Face,
        bin: rustycad_tkmath::gp::Vector3,
        tb: State,
    ) -> Result<(bool, f64), Failure> {
        let fel = self.b1d()?.my_map_of_edge_faces.find_from_key(eor)?.clone();
        let (mut keep, mut scalar) = (false, 0.);
        if fel.len() > 2 {
            return Ok((keep, scalar));
        }
        for f in fel.iter() {
            if edge_face.is_same(f) {
                continue;
            }
            scalar = bin.dot(oriented_normal(f, e)?);
            if scalar.abs() <= 1e-10 {
                scalar = bin.dot(oriented_nearest_normal(f, e)?);
                if scalar.abs() <= 1e-10 {
                    continue;
                }
            }
            keep = (if scalar > 0. { State::In } else { State::Out }) == tb;
            if keep {
                break;
            }
        }
        Ok((keep, scalar))
    }

    /// `PerformPieceIn2D(EdgeToPerform, EOR, edgeFace, toFace, G1, keep)` (Builder1):
    /// whether the piece is kept by the side of the adjacent faces of the edge relative
    /// to its binormal on `to_face`, else (no side) by the operation and the relative
    /// orientation of the faces; `b.perform_piece_in2d(&e, &eor, &f1, &f2, &g)?`.
    pub fn perform_piece_in2d(
        &self,
        e: &Edge,
        eor: &Edge,
        edge_face: &Face,
        to_face: &Face,
        g1: &TopOpeBRepBuildGTopo,
    ) -> Result<bool, Failure> {
        let (tb, _) = Self::rank_states(g1, self.hds()?.ds().ancestor_rank(eor))?;
        let n2 = oriented_normal(to_face, e)?;
        let n3 = oriented_normal(edge_face, e)?;
        let mut tg = oriented_tangent(e)?;
        if edge_face.orientation() == Orientation::Reversed {
            tg.reverse();
        }
        let (mut keep, scalar) = self.piece_side_keep(e, eor, edge_face, tg.crossed(n2), tb)?;
        if scalar.abs() <= 1e-10 {
            let d = n3.dot(n2);
            if self.opefus() {
                keep = d > 0.;
            }
            if self.opec12() || self.opec21() {
                keep = d < 0.;
            }
            if self.opecom() {
                keep = d > 0.;
            }
        }
        Ok(keep)
    }

    /// `OrientateEdgeOnFace(EdgeToPerform, baseFace, edgeFace, G1, stateOfFaceOri)`
    /// (Builder1): the edge oriented by the rank of the same domain face to fill,
    /// reversed when that face and the base face differ in orientation and when the
    /// edge face is against the base face (the returned state, inverted for a tool face
    /// in a cut); `let st = b.orientate_edge_on_face(&mut e, &base, &f, &g)?;`.
    pub fn orientate_edge_on_face(
        &self,
        e: &mut Shape,
        base_face: &Shape,
        edge_face: &Shape,
        g1: &TopOpeBRepBuildGTopo,
    ) -> Result<bool, Failure> {
        let sd = self.b1d()?.my_sd_face_to_fill.clone();
        let (_, rev_ori) = Self::rank_states(g1, self.hds()?.ds().ancestor_rank(&sd))?;
        let neworie = Self::orient(e.orientation(), rev_ori);
        let state_of_face_ori = self.state_of_face_ori(edge_face, base_face, e)?;
        e.set_orientation(neworie);
        if sd.orientation() != base_face.orientation() {
            e.reverse();
        }
        if state_of_face_ori {
            e.reverse();
        }
        Ok(state_of_face_ori)
    }
}
