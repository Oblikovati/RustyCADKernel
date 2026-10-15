//! `GFillEdgeSameDomWES` of `TopOpeBRepBuild_Builder1`: the parts of an edge of a same
//! domain face, the ON parts kept by their 2d pieces, their position in the same
//! domain faces or the side of the adjacent faces.
use super::first;
use super::on_parts::classify_edge_to_face_by_one_point;
use crate::top_ope_brep_build::TopOpeBRepBuildShapeSet;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildBuilder, TopOpeBRepBuildGTopo, TopOpeBRepBuildTools, TopOpeBRepBuildWireEdgeSet,
};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Vector3;

/// The normal of a face at the edge, reversed for a reversed face.
pub(super) fn oriented_normal(f: &Shape, e: &Shape) -> Result<Vector3, Failure> {
    let mut n =
        TopOpeBRepBuildTools::get_normal_to_face_on_edge(&topods::face(f)?, &topods::edge(e)?)?;
    if f.orientation() == Orientation::Reversed {
        n.reverse();
    }
    Ok(n)
}

/// The normal of a face moved inside from the edge, reversed for a reversed face.
pub(super) fn oriented_nearest_normal(f: &Shape, e: &Shape) -> Result<Vector3, Failure> {
    let mut n =
        TopOpeBRepBuildTools::get_normal_in_nearest_point(&topods::face(f)?, &topods::edge(e)?)?;
    if f.orientation() == Orientation::Reversed {
        n.reverse();
    }
    Ok(n)
}

/// The tangent of the edge at its sample point, reversed for a reversed edge.
pub(super) fn oriented_tangent(e: &Shape) -> Result<Vector3, Failure> {
    let (_, mut t) = TopOpeBRepBuildTools::get_tangent_to_edge(&topods::edge(e)?)?;
    if e.orientation() == Orientation::Reversed {
        t.reverse();
    }
    Ok(t)
}

impl TopOpeBRepBuildBuilder {
    /// A kept part of an edge of a same domain face: moved to the base face, oriented on
    /// it, with its face state, as a start element.
    fn add_same_dom_part(
        &mut self,
        part: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let mut e = part.clone();
        self.move_to_base_face(&e)?;
        let st = self.orientate_edge_on_face_by_sd(&mut e, g1)?;
        self.b1d_mut()?
            .my_map_of_edge_with_face_state
            .bind(e.clone(), st);
        wes.add_start_element(&e)
    }

    /// `GFillEdgeSameDomWES(EOR, LSclass, G1, WES)` (Builder1): the parts of the edge on
    /// its state kept, then each ON part kept by its 2d pieces with the same domain
    /// edges, by its position in the same domain faces or by the side of the adjacent
    /// same domain face; `b.g_fill_edge_same_dom_wes(&e, &l, &g, &mut wes)?`.
    pub fn g_fill_edge_same_dom_wes(
        &mut self,
        eor: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let iref = self.hds()?.ds().ancestor_rank(eor);
        let (tb, _) = Self::rank_states(g1, iref)?;
        let orie = eor.orientation();
        let (lse, lse_on) = {
            let ds = self.hds()?.ds();
            let sws = ds.get_shape_with_state(eor);
            (sws.part(tb), sws.part(State::On))
        };
        for e in lse.iter() {
            self.add_same_dom_part(&e.oriented(orie), g1, wes)?;
        }
        for e in lse_on.iter() {
            let split_part = e.oriented(eor.orientation());
            if (self.hds()?.has_same_domain(&split_part, true)
                || self.hds()?.has_same_domain(eor, true))
                && self.same_dom_on_pieces(&split_part, eor, orie, g1, wes)?
            {
                continue;
            }
            self.same_dom_on_part(&split_part, eor, lsclass, tb, g1, wes)?;
        }
        Ok(())
    }

    /// The 2d pieces of an ON part with same domain edges: the out pieces added both
    /// ways, the pieces of the same domain face kept, oriented on the base face; true
    /// when pieces were found (or the part was processed), else the part is classified.
    fn same_dom_on_pieces(
        &mut self,
        split_part: &Shape,
        eor: &Shape,
        orie: Orientation,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<bool, Failure> {
        let sd = self.b1d()?.my_sd_face_to_fill.clone();
        let (mut e_on, mut n_eor) = (split_part.clone(), eor.clone());
        if sd.orientation() == Orientation::Reversed {
            e_on.reverse();
            n_eor.reverse();
        }
        if self.b1d()?.my_processed_parts_out2d.contains(&e_on) {
            return Ok(true);
        }
        let (mut pieces, mut faces, mut out2d) =
            (List::default(), List::default(), List::default());
        let flag =
            self.perform_piece_on2d(&e_on, &sd, &n_eor, &mut pieces, &mut faces, &mut out2d)?;
        for p in out2d.iter() {
            let fp = p.reversed();
            wes.add_start_element(&fp)?;
            wes.add_start_element(p)?;
            self.b1d_mut()?.my_processed_parts_out2d.add(fp);
        }
        for (piece, face) in pieces.iter().zip(faces.iter()) {
            if *face != sd {
                continue;
            }
            let is_rev = piece.orientation() == n_eor.orientation();
            let mut keep = piece.oriented(orie);
            let st = self.orientate_edge_on_face_by_sd(&mut keep, g1)?;
            if !is_rev {
                keep.reverse();
            }
            self.b1d_mut()?
                .my_map_of_edge_with_face_state
                .bind(keep.clone(), st);
            wes.add_start_element(&keep)?;
        }
        Ok(flag != 0)
    }

    /// The same domain face of the classes holding the ON part (IN or ON), its state;
    /// with one class, its state whatever it is.
    fn classify_on_part(
        split_part: &Shape,
        lsclass: &List<Shape>,
    ) -> Result<(Shape, State), Failure> {
        let e = topods::edge(split_part)?;
        if lsclass.len() == 1 {
            let sd = first(lsclass)?;
            let st = classify_edge_to_face_by_one_point(&e, &topods::face(&sd)?)?;
            return Ok((sd, st));
        }
        let mut st = State::Unknown;
        for c in lsclass.iter() {
            st = classify_edge_to_face_by_one_point(&e, &topods::face(c)?)?;
            if matches!(st, State::In | State::On) {
                return Ok((c.clone(), st));
            }
        }
        Ok((Shape::default(), st))
    }

    /// One ON part of the edge (no 2d pieces): skipped ON or UNKNOWN in the same domain
    /// faces; OUT (or in none), kept by the side of the adjacent same domain face; IN,
    /// kept by `PerformPieceIn2D` (and kept for the ON splits).
    fn same_dom_on_part(
        &mut self,
        split_part: &Shape,
        eor: &Shape,
        lsclass: &List<Shape>,
        tb: State,
        g1: &TopOpeBRepBuildGTopo,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<(), Failure> {
        let (sd_shape, state) = Self::classify_on_part(split_part, lsclass)?;
        if matches!(state, State::On | State::Unknown) {
            return Ok(());
        }
        if state == State::Out || sd_shape.is_null() {
            if self.keep_by_adjacent(split_part, eor, tb)? {
                self.add_same_dom_part(split_part, g1, wes)?;
            }
            return Ok(());
        }
        let sd = self.b1d()?.my_sd_face_to_fill.clone();
        let keep = self.perform_piece_in2d(
            &topods::edge(split_part)?,
            &topods::edge(eor)?,
            &topods::face(&sd)?,
            &topods::face(&sd_shape)?,
            g1,
        )?;
        if !keep {
            return Ok(());
        }
        self.b1d_mut()?.my_splits_on_to_keep.add(split_part.clone());
        let base = self.b1d()?.my_base_face_to_fill.clone();
        let mut e = split_part.clone();
        let st = self.orientate_edge_on_face(&mut e, &base, &sd_shape, g1)?;
        self.b1d_mut()?
            .my_map_of_edge_with_face_state
            .bind(e.clone(), st);
        if base == sd {
            self.b1d_mut()?.my_source_shapes.add(e.clone());
        }
        wes.add_start_element(&e)
    }

    /// The first face of the edge (of two at most) other than the same domain face to
    /// fill and with same domain faces (null when none).
    fn adjacent_sd_face(&self, eor: &Shape) -> Result<Shape, Failure> {
        let sd = self.b1d()?.my_sd_face_to_fill.clone();
        let fel = self.b1d()?.my_map_of_edge_faces.find_from_key(eor)?.clone();
        if fel.len() > 2 {
            return Ok(Shape::default());
        }
        for f in fel.iter() {
            if !sd.is_same(f) && self.hds()?.has_same_domain(f, true) {
                return Ok(f.clone());
            }
        }
        Ok(Shape::default())
    }

    /// Whether an OUT part is kept: by the side of the same domain face to fill relative
    /// to the binormal of the part on the same domain face of the adjacent face (IN when
    /// there is no adjacent face).
    fn keep_by_adjacent(
        &self,
        split_part: &Shape,
        eor: &Shape,
        tb: State,
    ) -> Result<bool, Failure> {
        let adj = self.adjacent_sd_face(eor)?;
        if adj.is_null() {
            return Ok(tb == State::In);
        }
        let mut em = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&adj, ShapeEnum::Edge, &mut em)?;
        let adj_eor = em.find_key(em.find_index(eor))?.clone();
        let sd_to_adj = first(&self.hds()?.same_domain(&adj))?;
        let split_p = split_part.oriented(adj_eor.orientation());
        let n2 = oriented_normal(&sd_to_adj, &split_p)?;
        oriented_normal(&adj, &split_p)?;
        let bin = oriented_tangent(&split_p)?.crossed(n2);
        let sd = self.b1d()?.my_sd_face_to_fill.clone();
        let mut scalar = bin.dot(oriented_normal(&sd, &split_p)?);
        if scalar.abs() <= 1e-10 {
            scalar = bin.dot(oriented_nearest_normal(&sd, &split_p)?);
        }
        let part_state = if scalar > 0. { State::In } else { State::Out };
        Ok(part_state == tb)
    }
}
