//! `BOPAlgo_BOP::BuildSolid`, `MapFacesToBuildSolids` and `CheckArgsForOpenSolid`
//! (BOPAlgo_BOP.cxx): the solids of a FUSE of solids and the detection of open solids.
use super::BOPAlgoBOP;
use super::build_shape::collect_containers;
use crate::bop_algo::alerts::alert;
use crate::bop_algo::{AlertSolidBuilderFailed, AlertSolidBuilderUnusedFaces, BuilderSolid};
use crate::bop_ds::DataStructure;
use crate::bop_tools::{ShapeSet, make_connexity_compounds, make_container};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape, ShapeMapHasher};
use rustycad_tkernel::message::Gravity;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

impl BOPAlgoBOP {
    /// The solids of a FUSE of solids (`BuildSolid`): untouched solids kept as they are, the
    /// others rebuilt by the solid builder from their outer faces, then grouped into
    /// compsolids when the arguments had compsolids.
    pub(super) fn build_solid(&mut self) -> Result<(), Failure> {
        let (argument_solids, shared_solids, containers, mut face_solids) =
            self.argument_solids()?;
        let mut maybe_untouched: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        face_solids.clear(false);
        for part in ShapeIterator::new(&self.rc, true, true) {
            if argument_solids.contains(&part) && !shared_solids.contains(&part) {
                maybe_untouched.add(part);
                continue;
            }
            map_faces_to_build_solids(&part, &mut face_solids)?;
        }
        let untouched = untouched_solids(&maybe_untouched, &mut face_solids)?;
        let mut result = Shape::default();
        make_container(ShapeEnum::Compound, &mut result);
        if !self.add_built_solids(&face_solids, &mut result)? {
            return Ok(());
        }
        for position in 1..=untouched.extent() {
            BRepBuilder::default().add(&mut result, untouched.find_from_index(position)?)?;
        }
        self.provider.builder.shape = if containers.is_empty() {
            result
        } else {
            self.group_into_compsolids(&result, &containers)?
        };
        Ok(())
    }

    /// The solids of the groups (tools first, as natively), the solids sharing faces, the
    /// compsolid containers and the face/solid map of the argument solids.
    #[allow(clippy::type_complexity)]
    fn argument_solids(
        &self,
    ) -> Result<
        (
            MapOfShape,
            MapOfShape,
            Vec<Shape>,
            IndexedDataMapOfShapeListOfShape,
        ),
        Failure,
    > {
        let mut solids = MapOfShape::default();
        let mut face_solids = IndexedDataMapOfShapeListOfShape::default();
        let mut containers = Vec::new();
        for group in [self.provider.tools(), &self.provider.builder.arguments] {
            for shape in group.iter() {
                for solid in Explorer::new(shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
                    solids.add(solid.clone());
                    topexp::map_shapes_and_ancestors(
                        &solid,
                        ShapeEnum::Face,
                        ShapeEnum::Solid,
                        &mut face_solids,
                    )?;
                }
                collect_containers(shape, &mut containers)?;
            }
        }
        let mut shared = MapOfShape::default();
        // Native: the last face of the map is not checked (loop to `i < aNb`).
        for position in 1..face_solids.extent() {
            let owners = face_solids.find_from_index(position)?;
            if owners.len() > 1 {
                for owner in owners.iter() {
                    shared.add(owner.clone());
                }
            }
        }
        Ok((solids, shared, containers, face_solids))
    }

    /// Adds the solids built from the faces met once; false after the solid builder failed.
    fn add_built_solids(
        &mut self,
        face_solids: &IndexedDataMapOfShapeListOfShape,
        result: &mut Shape,
    ) -> Result<bool, Failure> {
        let mut faces = List::default();
        for position in 1..=face_solids.extent() {
            if face_solids.find_from_index(position)?.len() == 1 {
                faces.append(face_solids.find_key(position)?.clone());
            }
        }
        if faces.is_empty() {
            return Ok(true);
        }
        let mut builder = BuilderSolid::new();
        builder.set_context(self.provider.builder.context.clone());
        builder.set_shapes(&faces);
        builder.set_avoid_internal_shapes(true);
        builder.perform()?;
        if builder.options().has_errors() {
            self.builder()
                .options()
                .add_error(&alert(AlertSolidBuilderFailed));
            return Ok(false);
        }
        self.builder()
            .options()
            .report()
            .merge(builder.options().report());
        for solid in builder.areas().iter() {
            BRepBuilder::default().add(result, solid)?;
        }
        Ok(true)
    }

    /// The new solids grouped into compsolids where they hold splits of compsolid faces.
    fn group_into_compsolids(
        &self,
        solids: &Shape,
        containers: &[Shape],
    ) -> Result<Shape, Failure> {
        let builder = BRepBuilder::default();
        let mut result = Shape::default();
        make_container(ShapeEnum::Compound, &mut result);
        let parts: Vec<Shape> = ShapeIterator::new(solids, true, true).collect();
        if parts.is_empty() {
            return Ok(solids.clone());
        }
        if parts.len() == 1 {
            // One solid in the result.
            let mut compsolid = Shape::default();
            make_container(ShapeEnum::CompSolid, &mut compsolid);
            builder.add(&mut compsolid, &parts[0])?;
            builder.add(&mut result, &compsolid)?;
            return Ok(result);
        }
        let compsolid_faces = self.compsolid_face_splits(containers)?;
        let mut blocks = Vec::new();
        make_connexity_compounds(solids, ShapeEnum::Face, ShapeEnum::Solid, &mut blocks)?;
        for block in blocks {
            let has_compsolid_face = Explorer::new(&block, ShapeEnum::Face, ShapeEnum::Shape)?
                .any(|face| compsolid_faces.contains(&face));
            if !has_compsolid_face {
                // Origins are not compsolids: the solids go directly into the result.
                for solid in ShapeIterator::new(&block, true, true) {
                    builder.add(&mut result, &solid)?;
                }
                continue;
            }
            let mut compsolid = Shape::default();
            make_container(ShapeEnum::CompSolid, &mut compsolid);
            for solid in ShapeIterator::new(&block, true, true) {
                builder.add(&mut compsolid, &solid)?;
            }
            builder.add(&mut result, &compsolid)?;
        }
        Ok(result)
    }

    /// The faces of the compsolid containers, replaced by their splits.
    fn compsolid_face_splits(&self, containers: &[Shape]) -> Result<MapOfShape, Failure> {
        let mut faces = MapOfShape::default();
        for container in containers {
            for face in Explorer::new(container, ShapeEnum::Face, ShapeEnum::Shape)? {
                match self.builder().images().seek(&face) {
                    Some(splits) => splits.iter().for_each(|split| {
                        faces.add(split.clone());
                    }),
                    None => {
                        faces.add(face);
                    }
                }
            }
        }
        Ok(faces)
    }

    /// Whether an argument solid is open and the solid builder warned about it or its splits
    /// acquired new INTERNAL faces (`CheckArgsForOpenSolid`).
    pub(super) fn check_open_solid_arguments(
        &self,
        structure: &DataStructure,
    ) -> Result<bool, Failure> {
        let failed = self.solids_with_unused_faces()?;
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Solid {
                continue;
            }
            let solid = info.shape();
            let (edge_faces, internal) = self.solid_face_maps(solid)?;
            if is_closed_solid(&edge_faces)? {
                continue;
            }
            if failed.contains(solid) {
                return Ok(true);
            }
            if self.has_new_internal_faces(solid, &internal) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The solids of the shapes of the `AlertSolidBuilderUnusedFaces` warnings.
    fn solids_with_unused_faces(&self) -> Result<MapOfShape, Failure> {
        let mut solids = MapOfShape::default();
        for warning in self
            .builder()
            .options()
            .report()
            .get_alerts(Gravity::Warning)
        {
            let any: &dyn std::any::Any = warning.as_ref();
            let Some(unused) = any.downcast_ref::<AlertSolidBuilderUnusedFaces>() else {
                continue;
            };
            let shape = unused.get_shape();
            if shape.is_null() {
                continue;
            }
            for solid in Explorer::new(&shape, ShapeEnum::Solid, ShapeEnum::Shape)? {
                solids.add(solid);
            }
        }
        Ok(solids)
    }

    /// The edge/face map of the not INTERNAL faces of the solid shells and the splits of
    /// its INTERNAL faces.
    fn solid_face_maps(
        &self,
        solid: &Shape,
    ) -> Result<(IndexedDataMapOfShapeListOfShape, MapOfShape), Failure> {
        let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
        let mut internal = MapOfShape::default();
        for shell in ShapeIterator::new(solid, true, true) {
            if shell.shape_type()? != ShapeEnum::Shell {
                continue;
            }
            for face in ShapeIterator::new(&shell, true, true) {
                if face.orientation() != Orientation::Internal {
                    topexp::map_shapes_and_ancestors(
                        &face,
                        ShapeEnum::Edge,
                        ShapeEnum::Face,
                        &mut edge_faces,
                    )?;
                    continue;
                }
                match self.builder().images().seek(&face) {
                    Some(splits) => splits.iter().for_each(|split| {
                        internal.add(split.clone());
                    }),
                    None => {
                        internal.add(face);
                    }
                }
            }
        }
        Ok((edge_faces, internal))
    }

    /// Whether a split of the solid has an INTERNAL face it did not have.
    fn has_new_internal_faces(&self, solid: &Shape, internal: &MapOfShape) -> bool {
        let Some(splits) = self.builder().images().seek(solid) else {
            return false;
        };
        splits.iter().any(|split| {
            ShapeIterator::new(split, true, true)
                .filter(|shell| shell.shape_type().ok() == Some(ShapeEnum::Shell))
                .any(|shell| {
                    ShapeIterator::new(&shell, true, true).any(|face| {
                        face.orientation() == Orientation::Internal && !internal.contains(&face)
                    })
                })
        })
    }
}

/// Whether every non-degenerated edge has two faces, is a seam or is INTERNAL in its face.
fn is_closed_solid(edge_faces: &IndexedDataMapOfShapeListOfShape) -> Result<bool, Failure> {
    for position in 1..=edge_faces.extent() {
        let edge = Edge::unchecked(edge_faces.find_key(position)?.clone());
        if BRepTool::degenerated(&edge)? {
            continue;
        }
        let faces = edge_faces.find_from_index(position)?;
        if faces.len() > 1 {
            continue;
        }
        let face = Face::unchecked(faces.iter().next().expect("one face").clone());
        if BRepTool::is_closed_on_face(&edge, &face)? {
            continue;
        }
        let internal = Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)?
            .find(|occurrence| occurrence.is_same(&edge))
            .is_some_and(|occurrence| occurrence.orientation() == Orientation::Internal);
        if !internal {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The untouched solids whose faces are not used by the solids to rebuild, by face set;
/// the others have their faces mapped for rebuilding.
fn untouched_solids(
    candidates: &IndexedMap<Shape, ShapeMapHasher>,
    face_solids: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<IndexedDataMap<ShapeSet, Shape>, Failure> {
    let mut untouched: IndexedDataMap<ShapeSet, Shape> = IndexedDataMap::new(1);
    for solid in candidates.iter() {
        let shares_face = Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape)?
            .any(|face| face_solids.contains(&face));
        if shares_face {
            map_faces_to_build_solids(solid, face_solids)?;
            continue;
        }
        let mut set = ShapeSet::new();
        set.add(solid, ShapeEnum::Face)?;
        if !untouched.contains(&set) {
            untouched.add(set, solid.clone());
        }
    }
    Ok(untouched)
}

/// Maps the not INTERNAL faces of the solid to the solids using them, a solid counted again
/// only when it uses the face with the other orientation (`MapFacesToBuildSolids`).
fn map_faces_to_build_solids(
    solid: &Shape,
    face_solids: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    for face in Explorer::new(solid, ShapeEnum::Face, ShapeEnum::Shape)? {
        if face.orientation() == Orientation::Internal {
            continue;
        }
        let index = face_solids.find_index(&face);
        if index == 0 {
            let mut owners = List::default();
            owners.append(solid.clone());
            face_solids.add(face, owners);
            continue;
        }
        if face_solids.find_key(index)?.orientation() != face.orientation() {
            face_solids.change_from_index(index)?.append(solid.clone());
        }
    }
    Ok(())
}
