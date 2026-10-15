//! `BOPAlgo_Builder::FillImagesSolids`, `FillIn3DParts` and `BuildDraftSolid`
//! (BOPAlgo_Builder_3.cxx).
use super::{BOPAlgoBuilder, BuilderHooks, SameDomainShapes, required_context};
use crate::bop_algo::tools::{InParts, ShapeBoxMap, SolidInternalFaces, classify_faces};
use crate::bop_ds::DataStructure;
use crate::bop_tools::is_split_to_reverse_with_warning;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Shape, ShapeIterator, Shell, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::bnd::BoundingBox3;

/// The draft solids of the source solids (`aDraftSolid`).
type DraftSolids = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

/// `FillImagesSolids` of the algorithm `hooks`: its (virtual) `FillIn3DParts`
/// classifies the faces into draft solids, then the solid builder and the internal
/// shapes; `fill_images_solids_with(&mut algo, &mut ds)?`.
pub fn fill_images_solids_with<H: BuilderHooks>(
    hooks: &mut H,
    structure: &mut DataStructure,
) -> Result<(), Failure> {
    let mut has_solid = false;
    for index in 0..structure.nb_source_shapes() {
        if structure.shape_info(index)?.shape_type() == ShapeEnum::Solid {
            has_solid = true;
            break;
        }
    }
    if !has_solid {
        return Ok(());
    }
    let mut draft_solids: SameDomainShapes = DataMap::with_hasher(ShapeMapHasher, 1);
    hooks.fill_in_3d_parts(structure, &mut draft_solids)?;
    let builder = hooks.general_fuse();
    if builder.options().has_errors() {
        return Ok(());
    }
    builder.build_split_solids(structure, &draft_solids)?;
    if builder.options().has_errors() {
        return Ok(());
    }
    builder.fill_internal_shapes(structure)
}

impl BOPAlgoBuilder {
    /// Splits the solids: faces classified into draft solids, then the solid builder and
    /// the internal shapes (`FillImagesSolids`); `builder.fill_images_solids(&mut ds)?`.
    pub fn fill_images_solids(&mut self, structure: &mut DataStructure) -> Result<(), Failure> {
        fill_images_solids_with(self, structure)
    }

    /// Classifies all faces against the draft solids; binds the draft of each solid that has
    /// IN faces or modified shells, and its IN and internal faces (`FillIn3DParts`).
    pub fn fill_in_3d_parts(
        &mut self,
        structure: &mut DataStructure,
        draft_solids: &mut SameDomainShapes,
    ) -> Result<(), Failure> {
        let mut boxes: ShapeBoxMap = DataMap::with_hasher(ShapeMapHasher, 1);
        let faces = self.all_face_splits(structure, &mut boxes)?;
        let mut solids = List::default();
        let mut internal: SolidInternalFaces = DataMap::with_hasher(ShapeMapHasher, 1);
        let mut drafts: DraftSolids = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        for index in 0..structure.nb_source_shapes() {
            if structure.shape_info(index)?.shape_type() != ShapeEnum::Solid {
                continue;
            }
            let bounds = self.solid_box(structure, index)?;
            let solid = structure.shape_info(index)?.shape().clone();
            let mut internal_faces = List::default();
            let draft = self.build_draft_solid(&solid, &mut internal_faces)?;
            solids.append(draft.clone());
            internal.bind(draft.clone(), internal_faces);
            boxes.bind(draft.clone(), bounds);
            drafts.add(solid, draft);
        }
        let handle = self.context.clone();
        let mut in_parts: InParts = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        classify_faces(
            &faces,
            &solids,
            required_context(&handle)?,
            &mut in_parts,
            &boxes,
            &internal,
        )?;
        self.bind_in_parts(&drafts, &in_parts, &internal, draft_solids)
    }

    /// The splits of all faces once each, and the original faces with their DS box.
    fn all_face_splits(
        &self,
        structure: &DataStructure,
        boxes: &mut ShapeBoxMap,
    ) -> Result<List<Shape>, Failure> {
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
        let mut faces = List::default();
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Face {
                continue;
            }
            let Some(splits) = self.images.seek(info.shape()) else {
                faces.append(info.shape().clone());
                boxes.bind(info.shape().clone(), info.box_bounds().clone());
                continue;
            };
            for split in splits.iter() {
                if fence.add(split.clone()) {
                    faces.append(split.clone());
                }
            }
        }
        Ok(faces)
    }

    /// The box of the solid in the DS, built (and stored) when void.
    fn solid_box(
        &self,
        structure: &mut DataStructure,
        index: i32,
    ) -> Result<BoundingBox3, Failure> {
        let mut bounds = structure.shape_info(index)?.box_bounds().clone();
        if bounds.is_void() {
            structure.build_bnd_box_solid(index, &mut bounds, self.check_inverted)?;
            *structure.change_shape_info(index)?.change_box() = bounds.clone();
        }
        Ok(bounds)
    }

    /// Binds the draft of the solids to split and their IN and internal faces.
    fn bind_in_parts(
        &mut self,
        drafts: &DraftSolids,
        in_parts: &InParts,
        internal: &SolidInternalFaces,
        draft_solids: &mut SameDomainShapes,
    ) -> Result<(), Failure> {
        for position in 1..=drafts.extent() {
            let solid = drafts.find_key(position)?;
            let draft = drafts.find_from_index(position)?;
            let inside = in_parts.find_from_key(draft)?;
            let own_internal = internal.find(draft)?;
            if inside.is_empty()
                && !ShapeIterator::new(solid, true, true).any(|shell| self.images.is_bound(&shell))
            {
                // No need to split the solid.
                continue;
            }
            draft_solids.bind(solid.clone(), draft.clone());
            if inside.is_empty() && own_internal.is_empty() {
                continue;
            }
            let parts = self.in_parts.bound(solid.clone(), List::default());
            for face in inside.iter().chain(own_internal.iter()) {
                parts.append(face.clone());
            }
        }
        Ok(())
    }

    /// The solid rebuilt on the splits of its faces, oriented as the originals; the INTERNAL
    /// splits are returned in `internal_faces` (`BuildDraftSolid`).
    pub fn build_draft_solid(
        &mut self,
        solid: &Shape,
        internal_faces: &mut List<Shape>,
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut draft = Solid::default();
        builder.make_solid(&mut draft);
        let mut draft = draft.into_shape();
        draft.set_orientation(solid.orientation());
        for shell in ShapeIterator::new(solid, true, true) {
            if shell.shape_type()? != ShapeEnum::Shell {
                // Maybe internal edges or vertices.
                continue;
            }
            let (draft_shell, has_faces) = self.draft_shell(&shell, internal_faces)?;
            if has_faces {
                builder.add(&mut draft, &draft_shell)?;
            }
        }
        Ok(draft)
    }

    fn draft_shell(
        &mut self,
        shell: &Shape,
        internal_faces: &mut List<Shape>,
    ) -> Result<(Shape, bool), Failure> {
        let builder = BRepBuilder::default();
        let mut draft = Shell::default();
        builder.make_shell(&mut draft);
        let mut draft = draft.into_shape();
        draft.set_orientation(shell.orientation());
        let mut has_faces = false;
        for face in ShapeIterator::new(shell, true, true) {
            let orientation = face.orientation();
            let splits: Vec<Shape> = match self.images.seek(&face) {
                Some(splits) => splits.iter().cloned().collect(),
                None => vec![face.clone()],
            };
            let modified = self.images.is_bound(&face);
            for split in splits {
                let split = self.draft_face_split(&face, split, modified)?;
                if orientation == Orientation::Internal {
                    internal_faces.append(split);
                } else {
                    has_faces = true;
                    builder.add(&mut draft, &split)?;
                }
            }
        }
        if has_faces {
            draft.set_closed(BRepTool::is_closed(&draft)?)?;
        }
        Ok((draft, has_faces))
    }

    /// The split oriented for the draft: a same-domain split as its original goes, any
    /// other split with the orientation of the original face.
    fn draft_face_split(
        &self,
        face: &Shape,
        split: Shape,
        modified: bool,
    ) -> Result<Shape, Failure> {
        let orientation = face.orientation();
        if !modified {
            return Ok(split);
        }
        if !self.shapes_sd.is_bound(&split) || orientation == Orientation::Internal {
            return Ok(split.oriented(orientation));
        }
        let context = required_context(&self.context)?;
        if is_split_to_reverse_with_warning(&split, face, context, Some(self.options().report()))? {
            return Ok(split.reversed());
        }
        Ok(split)
    }
}
