//! `BOPAlgo_RemoveFeatures::RemoveFeatures` and `RemoveFeature`: the gaps of all features
//! filled, then each feature removed by rebuilding its solids with the reconstructed faces.
use super::BOPAlgoRemoveFeatures;
use super::fill_gap::{Connections, FillGap};
use super::helpers::{compound_of, explore, find_internals, make_removed, remove_internal_wires};
use super::solids::{
    FeatureInputs, OriginalFaces, fill_solids_history, get_original_faces, get_valid_solids,
};
use crate::bop_algo::{AlertUnableToRemoveTheFeature, BOPAlgoMakerVolume};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::Arc;

/// The validated rebuild of the solids of one feature.
struct Rebuilt {
    maker: BOPAlgoMakerVolume,
    faces: OriginalFaces,
    solids: Vec<Shape>,
    /// The faces and solids dropped from the maker-volume result.
    removed: Vec<Shape>,
}

impl BOPAlgoRemoveFeatures {
    /// Fills the gap of every feature, then removes the features one by one, so a failed
    /// removal does not stop the next ones (`RemoveFeatures`). The gaps are filled in
    /// sequence: BOPTools_Parallel gives the same results.
    pub(super) fn remove_features(&mut self) -> Result<(), Failure> {
        let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.input_shape,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut edge_faces,
        )?;
        let mut face_solids = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            &self.input_shape,
            ShapeEnum::Face,
            ShapeEnum::Solid,
            &mut face_solids,
        )?;
        let connections = Connections {
            edge_faces: &edge_faces,
            face_solids: &face_solids,
        };
        let run_parallel = self.base.options.run_parallel();
        let mut gaps: Vec<FillGap> = self
            .features
            .iter()
            .map(|feature| FillGap::new(feature, run_parallel))
            .collect();
        for gap in &mut gaps {
            gap.perform(&connections);
        }
        // The history tracks the solids and faces between the removals even when it is
        // not requested.
        if self.base.history.is_null() {
            self.base.history = Handle::new(History::new());
        }
        let count = gaps.len();
        for (position, gap) in gaps.iter().enumerate() {
            // The last removal needs no solid history unless the history is requested.
            let solids_history_needed = self.base.fill_history || position + 1 < count;
            self.remove_feature(gap, solids_history_needed)?;
        }
        Ok(())
    }

    /// Rebuilds the solids of the feature from their kept faces and the reconstructed
    /// adjacent faces with the maker-volume algorithm; any invalid outcome only warns
    /// (`RemoveFeature`).
    fn remove_feature(
        &mut self,
        gap: &FillGap,
        solids_history_needed: bool,
    ) -> Result<(), Failure> {
        let handle = self.base.history.clone();
        let history = handle.get().ok_or_else(null_history)?;
        let Some(mut rebuilt) = self.rebuild_solids(gap, history)? else {
            self.warn_unable_to_remove(&gap.feature);
            return Ok(());
        };
        let faces = &mut rebuilt.faces;
        remove_internal_wires(&rebuilt.solids, Some(&mut faces.internals))?;
        history.merge_handle(&gap.history)?;
        if let Some(maker_history) = rebuilt.maker.history() {
            history.merge_handle(&maker_history)?;
        }
        let removals = faces
            .internals
            .iter()
            .chain(&rebuilt.removed)
            .cloned()
            .collect();
        self.record_removals(&rebuilt.solids, removals, history)?;
        if solids_history_needed {
            let solids_history = History::new();
            let solids = &mut rebuilt.solids;
            fill_solids_history(
                &faces.solids_to_rebuild,
                solids,
                &gap.faces,
                &rebuilt.maker,
                &solids_history,
            )?;
            history.merge(&solids_history)?;
        }
        rebuilt.solids.extend(faces.untouched.iter().cloned());
        self.base.shape = compound_of(&rebuilt.solids)?;
        Ok(())
    }

    /// The solids of the feature rebuilt by the maker-volume algorithm and validated;
    /// None when the removal must only warn: adjacent faces not rebuilt, no solid, a face
    /// to keep lost, or not one result solid per input solid.
    fn rebuild_solids(&self, gap: &FillGap, history: &History) -> Result<Option<Rebuilt>, Failure> {
        if gap.faces.extent() == 0 && gap.has_adjacent_faces {
            return Ok(None);
        }
        // Without adjacent faces an internal part is removed: no fuse is needed.
        let fuse = gap.faces.extent() > 0;
        let inputs = FeatureInputs {
            solids: &gap.solids,
            feature_faces: &gap.feature_faces,
            adjacent_faces: &gap.faces,
        };
        let mut faces = get_original_faces(&self.base.shape, &inputs, history)?;
        let kept: Vec<Shape> = faces.to_be_kept.iter().cloned().collect();
        let original = compound_of(&kept)?;
        let mut maker = self.volume_maker(fuse, &original);
        self.add_adjacent_faces(gap, &mut maker, &mut faces)?;
        maker.perform();
        if maker.options().has_errors() || explore(maker.shape(), ShapeEnum::Solid)?.is_empty() {
            return Ok(None);
        }
        if !keeps_every_face(&maker, &faces, gap, history)? {
            return Ok(None);
        }
        let mut removed = Vec::new();
        let solids =
            get_valid_solids(&maker, &faces, &original, gap.solids.extent(), &mut removed)?;
        if solids.len() != gap.solids.extent() {
            return Ok(None);
        }
        Ok(Some(Rebuilt {
            maker,
            faces,
            solids,
            removed,
        }))
    }

    /// The maker-volume tool on the original faces, avoiding internal shapes and keeping
    /// the inputs.
    fn volume_maker(&self, fuse: bool, original: &Shape) -> BOPAlgoMakerVolume {
        let mut maker = BOPAlgoMakerVolume::new();
        maker
            .options_mut()
            .set_run_parallel(self.base.options.run_parallel());
        maker.set_avoid_internal_shapes(true);
        maker.set_intersect(fuse);
        maker.set_non_destructive(true);
        maker.add_argument(original);
        maker
    }

    /// Adds the reconstructed adjacent faces (several images of one face as a compound, so
    /// they do not intersect) to the maker and to the faces to keep; with the history, the
    /// internal parts of the original adjacent faces are collected.
    fn add_adjacent_faces(
        &self,
        gap: &FillGap,
        maker: &mut BOPAlgoMakerVolume,
        faces: &mut OriginalFaces,
    ) -> Result<(), Failure> {
        for position in 1..=gap.faces.extent() {
            let images: Vec<Shape> = gap
                .faces
                .find_from_index(position)?
                .iter()
                .cloned()
                .collect();
            let argument = match images.as_slice() {
                [single] => single.clone(),
                _ => compound_of(&images)?,
            };
            maker.add_argument(&argument);
            faces.to_be_kept.add(argument);
            if self.base.fill_history {
                find_internals(gap.faces.find_key(position)?, &mut faces.internals);
            }
        }
        Ok(())
    }

    /// With the history, the internal shapes and the dropped faces and solids without trace
    /// in the solids are removed.
    fn record_removals(
        &self,
        solids: &[Shape],
        removals: Vec<Shape>,
        history: &History,
    ) -> Result<(), Failure> {
        if !self.base.fill_history {
            return Ok(());
        }
        let mut result_shapes = IndexedMapOfShape::default();
        for solid in solids {
            topexp::map_shapes(solid, &mut result_shapes, true, true);
        }
        let removed = History::new();
        make_removed(&removals, &removed, &result_shapes)?;
        history.merge(&removed)
    }

    fn warn_unable_to_remove(&self, feature: &Shape) {
        let alert: AlertHandle = Arc::new(AlertUnableToRemoveTheFeature::new(feature));
        self.base.options.add_warning(&alert);
    }
}

/// Whether every face to keep, except the earlier splits of the adjacent faces, keeps a
/// face in the maker-volume result.
fn keeps_every_face(
    maker: &BOPAlgoMakerVolume,
    faces: &OriginalFaces,
    gap: &FillGap,
    history: &History,
) -> Result<bool, Failure> {
    let mut adjacent_splits = MapOfShape::default();
    for face in (1..=gap.faces.extent()).map(|position| gap.faces.find_key(position)) {
        let face = face?;
        let images = history.modified(face)?;
        if images.is_empty() {
            adjacent_splits.add(face.clone());
        }
        for image in images {
            adjacent_splits.add(image);
        }
    }
    for shape in faces
        .to_be_kept
        .iter()
        .filter(|shape| !adjacent_splits.contains(shape))
    {
        let mut kept = false;
        for face in explore(shape, ShapeEnum::Face)? {
            if !maker.is_deleted(&face)? {
                kept = true;
                break;
            }
        }
        if !kept {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The failure of a removal run without the history `RemoveFeatures` creates.
fn null_history() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BOPAlgo_RemoveFeatures::RemoveFeature: the history is null; expected RemoveFeatures to create it",
        "",
    )
}
