//! The static helpers of BOPAlgo_RemoveFeatures.cxx selecting the faces and solids that
//! take part in the removal of one feature, and validating the rebuilt solids.
use super::helpers::{children, explore, find_shape, take_modified, take_modified_into};
use crate::bop_algo::builder_area::required_context;
use crate::bop_algo::{BOPAlgoBuilder, BuilderShape, BuilderSolid};
use crate::bop_tools::{ShapeSet, is_split_to_reverse_with_warning};
use rustycad_tkbrep::brep_tools::History;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// What one feature removal works on (the outputs of `GetOriginalFaces`).
#[derive(Default)]
pub(super) struct OriginalFaces {
    /// Faces that must keep a trace in the result (`theFacesToBeKept`).
    pub(super) to_be_kept: IndexedMapOfShape,
    /// Internal shapes to avoid and remove (`theInternalShapes`).
    pub(super) internals: Vec<Shape>,
    /// Faces of one solid only, whose orientation the result must keep (`theFacesToCheckOri`).
    pub(super) to_check_orientation: MapOfShape,
    /// The current images of the solids of the feature (`theSolidsToRebuild`).
    pub(super) solids_to_rebuild: IndexedMapOfShape,
    /// Faces shared by two solids (`theSharedFaces`).
    pub(super) shared: Vec<Shape>,
    /// Solids not involved, added to the result as they are (`theUnTouchedSolids`).
    pub(super) untouched: Vec<Shape>,
}

/// The feature, adjacent and solid data a removal starts from (`GetOriginalFaces`).
pub(super) struct FeatureInputs<'a> {
    pub(super) solids: &'a IndexedMapOfShape,
    pub(super) feature_faces: &'a MapOfShape,
    pub(super) adjacent_faces: &'a IndexedDataMapOfShapeListOfShape,
}

/// Collects the faces and solids of `shape` taking part in the removal of a feature, with
/// the images of earlier removals from `history` (`GetOriginalFaces`).
pub(super) fn get_original_faces(
    shape: &Shape,
    inputs: &FeatureInputs<'_>,
    history: &History,
) -> Result<OriginalFaces, Failure> {
    let mut faces = OriginalFaces::default();
    for solid in inputs.solids.iter() {
        let images = history.modified(solid)?;
        faces
            .solids_to_rebuild
            .add(images.first().unwrap_or(solid).clone());
    }
    let mut feature_splits = MapOfShape::default();
    for face in inputs.feature_faces.iter() {
        let images = history.modified(face)?;
        if images.is_empty() {
            feature_splits.add(face.clone());
        }
        for image in images {
            feature_splits.add(image);
        }
    }
    for solid in explore(shape, ShapeEnum::Solid)? {
        if !faces.solids_to_rebuild.contains(&solid) {
            faces.untouched.push(solid);
            continue;
        }
        for part in children(&solid) {
            faces.collect_shell(part, &feature_splits, inputs.adjacent_faces)?;
        }
    }
    Ok(faces)
}

impl OriginalFaces {
    /// The faces of one shell of a solid to rebuild; a non-shell part is internal.
    fn collect_shell(
        &mut self,
        shell: Shape,
        feature_splits: &MapOfShape,
        adjacent_faces: &IndexedDataMapOfShapeListOfShape,
    ) -> Result<(), Failure> {
        if shell.shape_type()? != ShapeEnum::Shell {
            self.internals.push(shell);
            return Ok(());
        }
        for face in children(&shell) {
            if feature_splits.contains(&face) || adjacent_faces.contains(&face) {
                continue;
            }
            if face.orientation() == Orientation::Internal {
                self.internals.push(shell.clone());
                continue;
            }
            self.to_be_kept.add(face.clone());
            if !self.to_check_orientation.add(face.clone()) {
                // A second solid holds the face: it is shared.
                self.to_check_orientation.remove(&face);
                self.shared.push(face);
            }
        }
        Ok(())
    }
}

/// The solids of the maker-volume result without those filling holes, splitting solids by
/// shared faces, or made of adjacent faces only (`GetValidSolids`); the removed faces and
/// solids go to `removed`.
pub(super) fn get_valid_solids(
    maker: &BOPAlgoBuilder,
    faces: &OriginalFaces,
    original_faces: &Shape,
    solid_count: usize,
    removed: &mut Vec<Shape>,
) -> Result<Vec<Shape>, Failure> {
    let mut solids = explore(maker.shape(), ShapeEnum::Solid)?;
    if solids.len() > solid_count {
        let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            maker.shape(),
            ShapeEnum::Face,
            ShapeEnum::Solid,
            &mut ancestors,
        )?;
        let mut to_avoid = MapOfShape::default();
        find_extra_shapes(
            &ancestors,
            &faces.to_check_orientation,
            maker,
            &mut to_avoid,
            None,
        )?;
        solids.retain(|solid| !to_avoid.contains(solid));
    }
    if solids.len() > solid_count {
        avoid_extra_shared_faces(&mut solids, &faces.shared, maker, removed)?;
    }
    if solids.len() > solid_count {
        remove_adjacent_only_solids(&mut solids, original_faces, maker, removed)?;
    }
    Ok(solids)
}

/// Removes the solids without an image of an original face.
fn remove_adjacent_only_solids(
    solids: &mut Vec<Shape>,
    original_faces: &Shape,
    maker: &BuilderShape,
    removed: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let mut original_images = MapOfShape::default();
    for face in explore(original_faces, ShapeEnum::Face)? {
        take_modified_into(&face, maker, &mut original_images)?;
    }
    let mut kept = Vec::new();
    for solid in solids.drain(..) {
        if explore(&solid, ShapeEnum::Face)?
            .iter()
            .any(|face| original_images.contains(face))
        {
            kept.push(solid);
        } else {
            removed.push(solid);
        }
    }
    *solids = kept;
    Ok(())
}

/// The shapes holding an image of a shape to check in the reversed orientation go to
/// `to_avoid`, unless another image validates them (`FindExtraShapes`).
pub(super) fn find_extra_shapes(
    connections: &IndexedDataMapOfShapeListOfShape,
    to_check: &MapOfShape,
    builder: &BOPAlgoBuilder,
    to_avoid: &mut MapOfShape,
    valid: Option<&mut MapOfShape>,
) -> Result<(), Failure> {
    let context = required_context(builder.context(), "BOPAlgo_RemoveFeatures")?;
    let mut own_valid = MapOfShape::default();
    let valid = valid.unwrap_or(&mut own_valid);
    for original in to_check.iter() {
        for image in take_modified(original, builder)? {
            let Some(holders) = connections.seek(&image) else {
                continue;
            };
            for holder in holders.iter() {
                if valid.contains(holder) {
                    continue;
                }
                let in_holder = find_shape(&image, holder)?;
                if is_split_to_reverse_with_warning(&in_holder, original, context, None)? {
                    to_avoid.add(holder.clone());
                } else {
                    valid.add(holder.clone());
                }
            }
        }
    }
    for shape in valid.iter() {
        to_avoid.remove(shape);
    }
    Ok(())
}

/// Rebuilds the solids without the faces shared by two of them that are not splits of
/// faces shared originally; the avoided faces go to `extra` (`AvoidExtraSharedFaces`).
fn avoid_extra_shared_faces(
    solids: &mut Vec<Shape>,
    shared_to_avoid: &[Shape],
    builder: &BuilderShape,
    extra: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let mut shared_splits = MapOfShape::default();
    for face in shared_to_avoid {
        take_modified_into(face, builder, &mut shared_splits)?;
    }
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    for solid in solids.iter() {
        topexp::map_shapes_and_ancestors(solid, ShapeEnum::Face, ShapeEnum::Solid, &mut ancestors)?;
    }
    let (mut arguments, mut extra_faces) = (List::default(), Vec::new());
    for solid in solids.iter() {
        for face in explore(solid, ShapeEnum::Face)? {
            if ancestors.find_from_key(&face)?.extent() != 2 || shared_splits.contains(&face) {
                arguments.append(face);
            } else {
                extra_faces.push(face);
            }
        }
    }
    if extra_faces.is_empty() {
        return Ok(());
    }
    let mut solid_builder = BuilderSolid::new();
    solid_builder.set_avoid_internal_shapes(true);
    solid_builder.set_shapes(&arguments);
    solid_builder.perform()?;
    if solid_builder.options().has_errors() {
        return Ok(());
    }
    *solids = solid_builder.areas().iter().cloned().collect();
    extra.extend(extra_faces);
    Ok(())
}

/// Each solid to rebuild is modified into its image in the result, or removed without
/// one; an image with the same faces is replaced by the solid itself in `solids_out`
/// (`FillSolidsHistory`).
pub(super) fn fill_solids_history(
    solids_in: &IndexedMapOfShape,
    solids_out: &mut [Shape],
    adjacent_faces: &IndexedDataMapOfShapeListOfShape,
    builder: &BOPAlgoBuilder,
    history: &History,
) -> Result<(), Failure> {
    for solid_in in solids_in.iter() {
        let solid_out = find_solid(solid_in, solids_out, adjacent_faces, builder)?;
        if solid_out.is_null() {
            history.remove(solid_in)?;
            continue;
        }
        let (mut set_in, mut set_out) = (ShapeSet::new(), ShapeSet::new());
        set_in.add(solid_in, ShapeEnum::Face)?;
        set_out.add(&solid_out, ShapeEnum::Face)?;
        if !set_in.is_equal(&set_out) {
            history.add_modified(solid_in, &solid_out)?;
            continue;
        }
        // The solids are the same: the result keeps the initial solid.
        if let Some(slot) = solids_out
            .iter_mut()
            .find(|solid| solid.is_same(&solid_out))
        {
            *slot = solid_in.clone();
        }
    }
    Ok(())
}

/// The result solid holding an image of a face of `solid_in` (of its reconstructed
/// adjacent faces) with the same orientation; null when there is none (`FindSolid`).
fn find_solid(
    solid_in: &Shape,
    solids_out: &[Shape],
    adjacent_faces: &IndexedDataMapOfShapeListOfShape,
    builder: &BOPAlgoBuilder,
) -> Result<Shape, Failure> {
    let context = required_context(builder.context(), "BOPAlgo_RemoveFeatures")?;
    for face in explore(solid_in, ShapeEnum::Face)? {
        let mut images = MapOfShape::default();
        match adjacent_faces.seek(&face) {
            Some(rebuilt) => {
                for adjacent in rebuilt.iter() {
                    take_modified_into(adjacent, builder, &mut images)?;
                }
            }
            None => take_modified_into(&face, builder, &mut images)?,
        }
        for solid in solids_out {
            for out_face in explore(solid, ShapeEnum::Face)? {
                if images.contains(&out_face)
                    && !is_split_to_reverse_with_warning(&out_face, &face, context, None)?
                {
                    return Ok(solid.clone());
                }
            }
        }
    }
    Ok(Shape::default())
}
