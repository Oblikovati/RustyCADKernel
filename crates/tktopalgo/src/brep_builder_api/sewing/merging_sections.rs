// Rust translation of the two merge blocks of BRepBuilderAPI_Sewing::Merging (the merge
// with the bound and with its cutting sections) (BRepBuilderAPI_Sewing.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The merge of one boundary, or of one of its sections, with the sections found near
//! it: the rejected ones are dropped, the others merged into one edge, and each merged
//! section mapped to that edge with its orientation.
use super::replace_edge::replace_edge;
use super::{BRepBuilderAPISewing, SewingShapeIndexedMap, live_reshape};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum};

/// Where a merge records its sections, and the re-shaper that receives the replacement
/// of each merged section (only for the sections of a cut boundary).
pub(super) struct MergeTarget<'m> {
    pub merged: &'m mut SewingShapeIndexedMap,
    pub loc_reshape: &'m Handle<dyn ReShape>,
    pub replace_sections: bool,
}

impl BRepBuilderAPISewing {
    /// Whether a found section is rejected: merged already (globally or in this
    /// merge), a cut boundary one of whose sections is, or a section whose boundary is.
    fn is_rejected_merge(&self, iedge: &Shape, merged: &SewingShapeIndexedMap) -> bool {
        let taken = |s: &Shape| self.my_merged_edges.contains(s) || merged.contains(s);
        if taken(iedge) {
            return true;
        }
        if let Ok(sections) = self.my_bound_sections.find(iedge)
            && sections.iter().any(taken)
        {
            return true;
        }
        match self.my_section_bound.find(iedge) {
            Ok(bnd) => taken(bnd),
            Err(_) => false,
        }
    }

    /// Drops the rejected sections and records the accepted ones in the merge map.
    fn filter_merged(
        &self,
        seq: &mut Sequence<Shape>,
        ori: &mut Sequence<bool>,
        merged: &mut SewingShapeIndexedMap,
    ) -> Result<(), Failure> {
        let mut ii = 1;
        while ii <= seq.len() {
            let iedge = seq.value(ii)?.clone();
            if self.is_rejected_merge(&iedge, merged) {
                seq.remove(ii)?;
                ori.remove(ii)?;
                continue;
            }
            merged.add(iedge.clone(), iedge);
            ii += 1;
        }
        Ok(())
    }

    /// Merges `edge` with the sections found near it, recording each actually merged
    /// section (and `edge`) with the merged edge in the target map; `edge` is dropped
    /// from the map when nothing was merged. False when no section was found.
    pub(super) fn merge_nearest(
        &self,
        edge: &Shape,
        target: &mut MergeTarget<'_>,
    ) -> Result<bool, Failure> {
        let (mut seq, mut ori) = (Sequence::default(), Sequence::default());
        if !self.merged_nearest_edges(edge, &mut seq, &mut ori)? {
            return Ok(false);
        }
        target.merged.add(edge.clone(), edge.clone());
        self.filter_merged(&mut seq, &mut ori, target.merged)?;
        let mut nb_merged = seq.len();
        if nb_merged != 0 {
            nb_merged = self.merge_accepted(edge, (&seq, &ori), target)?;
        }
        if nb_merged == 0 {
            target.merged.remove_key(edge);
        }
        Ok(true)
    }

    /// The merged edge of the accepted sections; each actually merged section is mapped
    /// to it (reversed with the merged edge and with its relative orientation), the
    /// others dropped; returns the number actually merged.
    fn merge_accepted(
        &self,
        edge: &Shape,
        (seq, ori): (&Sequence<Shape>, &Sequence<bool>),
        target: &mut MergeTarget<'_>,
    ) -> Result<usize, Failure> {
        let mut actually_merged = MapOfShape::default();
        let merged_edge = self.same_parameter_edge_of_sequence(
            edge,
            seq,
            ori,
            &mut actually_merged,
            target.loc_reshape,
        )?;
        let is_forward =
            !merged_edge.is_null() && merged_edge.orientation() == Orientation::Forward;
        let flip = |o: Orientation, keep: bool| if keep { o } else { top_abs::reverse(o) };
        let mut nb_actually_merged = 0;
        for ii in 1..=seq.len() {
            let iedge = seq.value(ii)?;
            if !actually_merged.contains(iedge) {
                target.merged.remove_key(iedge);
                continue;
            }
            nb_actually_merged += 1;
            let orient = flip(flip(iedge.orientation(), is_forward), *ori.value(ii)?);
            self.record_merged(iedge, merged_edge.oriented(orient), target)?;
        }
        if nb_actually_merged != 0 {
            let orient = flip(edge.orientation(), is_forward);
            self.record_merged(edge, merged_edge.oriented(orient), target)?;
        }
        Ok(nb_actually_merged)
    }

    /// Maps a merged section to the merged edge; for the sections of a cut boundary,
    /// the replacement is also recorded in the local re-shaper.
    fn record_merged(
        &self,
        iedge: &Shape,
        oedge: Shape,
        target: &mut MergeTarget<'_>,
    ) -> Result<(), Failure> {
        *target.merged.change_from_key(iedge)? = oedge.clone();
        if !target.replace_sections {
            return Ok(());
        }
        let image = live_reshape(&self.my_re_shape)?.apply(iedge, ShapeEnum::Shape)?;
        replace_edge(&image, &oedge, live_reshape(target.loc_reshape)?)
    }
}
