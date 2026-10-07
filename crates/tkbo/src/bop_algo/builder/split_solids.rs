//! `BOPAlgo_Builder::BuildSplitSolids` (BOPAlgo_Builder_3.cxx): the solid builder on the
//! draft solids with their IN faces.
use super::images::append_listed;
use super::{BOPAlgoBuilder, SameDomainShapes};
use crate::bop_algo::BuilderSolid;
use crate::bop_algo::alerts::alert_with_shape;
use crate::bop_ds::DataStructure;
use crate::bop_tools::{ShapeSet, make_container};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::message::Gravity;
use rustycad_tkernel::ncollection::{IndexedDataMap, List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The splits of each source solid (`aSolidsIm`).
type SolidSplits = IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>;

impl BOPAlgoBuilder {
    /// Builds the splits of the solids with IN faces, keeps the draft of the others, and
    /// binds them as images, a split equal to an untouched solid being replaced by it
    /// (`BuildSplitSolids`); `builder.build_split_solids(&ds, &drafts)?`.
    pub fn build_split_solids(
        &mut self,
        structure: &DataStructure,
        draft_solids: &SameDomainShapes,
    ) -> Result<(), Failure> {
        let mut untouched = self.untouched_solid_sets(structure, draft_solids)?;
        let mut splits: SolidSplits = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        let mut builders: Vec<(Shape, BuilderSolid)> = Vec::new();
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Solid {
                continue;
            }
            let solid = info.shape();
            let Some(draft) = draft_solids.seek(solid) else {
                continue;
            };
            match self.in_parts.seek(solid).filter(|faces| !faces.is_empty()) {
                None => {
                    let mut list = List::default();
                    list.append(draft.clone());
                    splits.add(solid.clone(), list);
                }
                Some(inside) => {
                    let builder = self.solid_builder(draft, inside)?;
                    builders.push((solid.clone(), builder));
                }
            }
        }
        for (solid, builder) in builders.iter_mut() {
            builder.perform()?;
            splits.add(solid.clone(), builder.areas().clone());
            self.merge_solid_builder_report(solid, builder)?;
        }
        self.bind_solid_splits(&splits, &mut untouched)
    }

    /// The face sets of the solids that are not split (`aMST`).
    fn untouched_solid_sets(
        &self,
        structure: &DataStructure,
        draft_solids: &SameDomainShapes,
    ) -> Result<Map<ShapeSet>, Failure> {
        let mut fence: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 100);
        let mut sets: Map<ShapeSet> = Map::new(100);
        for index in 0..structure.nb_source_shapes() {
            let info = structure.shape_info(index)?;
            if info.shape_type() != ShapeEnum::Solid {
                continue;
            }
            let solid = info.shape();
            if !fence.add(solid.clone()) || draft_solids.is_bound(solid) {
                continue;
            }
            let mut set = ShapeSet::new();
            set.add(solid, ShapeEnum::Face)?;
            sets.add(set);
        }
        Ok(sets)
    }

    /// A solid builder on the faces of the draft and the IN faces in both orientations.
    fn solid_builder(&self, draft: &Shape, inside: &List<Shape>) -> Result<BuilderSolid, Failure> {
        let mut faces = List::default();
        for face in Explorer::new(draft, ShapeEnum::Face, ShapeEnum::Shape)? {
            faces.append(face);
        }
        for face in inside.iter() {
            faces.append(face.oriented(Orientation::Forward));
            faces.append(face.oriented(Orientation::Reversed));
        }
        let mut builder = BuilderSolid::new();
        builder.set_shapes(&faces);
        let parallel = self.options().run_parallel();
        builder.options_mut().set_run_parallel(parallel);
        Ok(builder)
    }

    /// Warnings and errors of the solid builder become warnings, those with a shape getting
    /// a compound of the solid and that shape.
    fn merge_solid_builder_report(
        &mut self,
        solid: &Shape,
        builder: &BuilderSolid,
    ) -> Result<(), Failure> {
        for gravity in [Gravity::Warning, Gravity::Fail] {
            for alert in builder.options().report().get_alerts(gravity) {
                if let Some(with_shape) = alert_with_shape(&alert) {
                    let builder = BRepBuilder::default();
                    let mut warning = Shape::default();
                    make_container(ShapeEnum::Compound, &mut warning);
                    builder.add(&mut warning, solid)?;
                    builder.add(&mut warning, &with_shape.get_shape())?;
                    with_shape.set_shape(&warning);
                }
                self.options().add_warning(&alert);
            }
        }
        Ok(())
    }

    /// Binds the splits as images of the solids not bound yet; a split equal to an untouched
    /// solid is replaced by that solid and bound to it as same-domain.
    fn bind_solid_splits(
        &mut self,
        splits: &SolidSplits,
        untouched: &mut Map<ShapeSet>,
    ) -> Result<(), Failure> {
        for position in 1..=splits.extent() {
            let solid = splits.find_key(position)?;
            if self.images.is_bound(solid) {
                continue;
            }
            self.images.bound(solid.clone(), List::default());
            for split in splits.find_from_index(position)?.iter() {
                let mut set = ShapeSet::new();
                set.add(split, ShapeEnum::Face)?;
                let same_domain = untouched.contains(&set);
                let image = untouched.added(set).shape().clone();
                if let Some(images) = self.images.change_seek(solid) {
                    images.append(image.clone());
                }
                append_listed(&mut self.origins, &image, solid);
                if same_domain {
                    self.shapes_sd.bind(split.clone(), image);
                }
            }
        }
        Ok(())
    }
}
