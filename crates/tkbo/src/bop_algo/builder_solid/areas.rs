//! `BOPAlgo_BuilderSolid::PerformAreas`: growth solids from the shells, hole shells put
//! into the closest growth solid containing them.
use super::BuilderSolid;
use super::internal::{is_inside, required_context};
use crate::bop_tools::{BoxSelector, BoxSet};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, ShapeIterator, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, real::real_small};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::bnd::{BoundingBox3, box3_to_bvh};
use rustycad_tkmath::bvh::{BvhSetTraverse, BvhVec3d};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// Holes and the growth solid chosen for each (`aHoleSolidMap`).
type HoleSolids = IndexedDataMap<Shape, Shape, ShapeMapHasher>;

impl BuilderSolid {
    /// Builds the solids from the shells (`PerformAreas`).
    pub(super) fn perform_areas(&mut self) -> Result<(), Failure> {
        self.area.areas.clear();
        let handle = self.area.context.clone();
        let context = required_context(&handle)?;
        let (growths, holes) = classify_shells(&self.area.loops, context)?;
        if holes.is_empty() {
            for solid in growths {
                self.area.areas.append(solid.clone());
                self.boxes.bind(solid.clone(), shape_box(&solid)?);
            }
            return Ok(());
        }
        let mut tree: BoxSet<BvhVec3d> = BoxSet::default();
        tree.set_size(holes.extent());
        for (position, hole) in holes.iter().enumerate() {
            let bounds = shape_box(hole)?;
            tree.add(position as i32 + 1, box3_to_bvh(&bounds)?);
            self.boxes.bind(hole.clone(), bounds);
        }
        tree.build()?;
        let hole_solids = self.closest_growths(&growths, &holes, &mut tree, context)?;
        let solid_holes = holes_by_solid(&hole_solids)?;
        for solid in growths {
            if let Some(solid_hole_list) = solid_holes.seek(&solid) {
                let mut shape = solid.clone();
                for hole in solid_hole_list.iter() {
                    BRepBuilder::default().add(&mut shape, hole)?;
                }
                context
                    .solid_classifier(&Solid::unchecked(solid.clone()))?
                    .write()
                    .load(&solid)?;
            }
            self.area.areas.append(solid);
        }
        self.add_outer_holes(&holes, &hole_solids)
    }

    /// For each hole, the innermost growth solid containing it, among the growths whose
    /// box meets the hole box; binds the box of each growth.
    fn closest_growths(
        &mut self,
        growths: &[Shape],
        holes: &IndexedMap<Shape, ShapeMapHasher>,
        tree: &mut BoxSet<BvhVec3d>,
        context: &Context,
    ) -> Result<HoleSolids, Failure> {
        let mut hole_solids: HoleSolids = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        for solid in growths {
            let bounds = shape_box(solid)?;
            self.boxes.bind(solid.clone(), bounds.clone());
            let mut selector = BoxSelector::default();
            selector.set_box(box3_to_bvh(&bounds)?);
            selector.set_bvh_set(Some(&mut *tree));
            selector.select()?;
            for index in selector.indices().iter() {
                let hole = holes.find_key(*index as usize)?;
                if !is_inside(hole, solid, context)? {
                    continue;
                }
                if let Some(was) = hole_solids.change_seek(hole) {
                    if is_inside(solid, was, context)? {
                        *was = solid.clone();
                    }
                    continue;
                }
                hole_solids.add(hole.clone(), solid.clone());
            }
        }
        Ok(hole_solids)
    }

    /// Holes outside every solid become solids of their own with an infinite box; the boxes
    /// of the hole shells are dropped.
    fn add_outer_holes(
        &mut self,
        holes: &IndexedMap<Shape, ShapeMapHasher>,
        hole_solids: &HoleSolids,
    ) -> Result<(), Failure> {
        for hole in holes.iter() {
            if !hole_solids.contains(hole) {
                let solid = solid_of(hole)?;
                self.area.areas.append(solid.clone());
                let mut bounds = BoundingBox3::default();
                bounds.set_whole();
                self.boxes.bind(solid, bounds);
            }
            self.boxes.unbind(hole);
        }
        Ok(())
    }
}

/// Splits the shells into growth solids and hole shells: a shell sharing a face with a hole
/// is a growth, the others are classified from the infinite point.
fn classify_shells(
    loops: &List<Shape>,
    context: &Context,
) -> Result<(Vec<Shape>, IndexedMap<Shape, ShapeMapHasher>), Failure> {
    let mut growths = Vec::new();
    let mut holes: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    let mut hole_faces: IndexedMap<Shape, ShapeMapHasher> =
        IndexedMap::with_hasher(ShapeMapHasher, 1);
    for shell in loops.iter() {
        let is_growth = is_growth_shell(shell, &hole_faces) || !is_hole(shell, context)?;
        if is_growth {
            growths.push(solid_of(shell)?);
        } else {
            holes.add(shell.clone());
            topexp::map_shapes_of_type(shell, ShapeEnum::Face, &mut hole_faces)?;
        }
    }
    Ok((growths, holes))
}

fn solid_of(shell: &Shape) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut solid = Solid::default();
    builder.make_solid(&mut solid);
    let mut solid = solid.into_shape();
    builder.add(&mut solid, shell)?;
    Ok(solid)
}

fn shape_box(shape: &Shape) -> Result<BoundingBox3, Failure> {
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add(shape, &mut bounds, true)?;
    Ok(bounds)
}

/// Whether the shell shares a face with a hole shell (`IsGrowthShell`).
fn is_growth_shell(shell: &Shape, hole_faces: &IndexedMap<Shape, ShapeMapHasher>) -> bool {
    hole_faces.extent() > 0
        && ShapeIterator::new(shell, true, true).any(|face| hole_faces.contains(&face))
}

/// Whether the infinite point is inside the shell (`IsHole`): the shell is classified as
/// a solid.
fn is_hole(shell: &Shape, context: &Context) -> Result<bool, Failure> {
    let classifier = context.solid_classifier(&Solid::unchecked(shell.clone()))?;
    let mut classifier = classifier.write();
    classifier.perform_infinite_point(real_small())?;
    Ok(classifier.state() == State::In)
}

/// The back map from growth solids to their holes.
fn holes_by_solid(
    hole_solids: &HoleSolids,
) -> Result<IndexedDataMap<Shape, List<Shape>, ShapeMapHasher>, Failure> {
    let mut solid_holes: IndexedDataMap<Shape, List<Shape>, ShapeMapHasher> =
        IndexedDataMap::with_hasher(ShapeMapHasher, 1);
    for index in 1..=hole_solids.extent() {
        let hole = hole_solids.find_key(index)?.clone();
        let solid = hole_solids.find_from_index(index)?;
        if let Some(list) = solid_holes.change_seek(solid) {
            list.append(hole);
            continue;
        }
        let mut list = List::default();
        list.append(hole);
        solid_holes.add(solid.clone(), list);
    }
    Ok(solid_holes)
}
