//! `BOPAlgo_BuilderSolid::PerformInternalShapes`: the internal faces put into the solids
//! that contain them, with `MakeInternalShells` and the `IsInside` classification.
use super::BuilderSolid;
use crate::bop_algo::AlertSolidBuilderUnusedFaces;
use crate::bop_algo::tools::{InParts, SolidInternalFaces, classify_faces};
use crate::bop_tools::compute_state_face;
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Face, Shape, ShapeIterator, Shell, Solid};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::message::AlertHandle;
use rustycad_tkernel::ncollection::{IndexedDataMap, IndexedMap, List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, real::real_small};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use std::sync::Arc;

impl BuilderSolid {
    /// Puts the internal faces into the solids containing them, warning about the faces
    /// left over (`PerformInternalShapes`).
    pub(super) fn perform_internal_shapes(&mut self) -> Result<(), Failure> {
        if self.area.avoid_internal_shapes || self.area.loops_internal.is_empty() {
            // Internal parts are avoided by option, or there are none.
            return Ok(());
        }
        let mut faces: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        for shell in self.area.loops_internal.iter() {
            for face in ShapeIterator::new(shell, true, true) {
                faces.add(face);
            }
        }
        if self.area.areas.is_empty() {
            // No growth: a solid of the internal shells.
            let mut solid = empty_solid();
            for shell in make_internal_shells(&faces)?.iter() {
                BRepBuilder::default().add(&mut solid, shell)?;
            }
            self.area.areas.append(solid);
            return Ok(());
        }
        let handle = self.area.context.clone();
        let context = required_context(&handle)?;
        let face_list: List<Shape> = faces.iter().cloned().collect();
        let mut in_parts: InParts = IndexedDataMap::with_hasher(ShapeMapHasher, 1);
        classify_faces(
            &face_list,
            &self.area.areas,
            context,
            &mut in_parts,
            &self.boxes,
            &SolidInternalFaces::with_hasher(ShapeMapHasher, 1),
        )?;
        let done = add_internal_faces(&in_parts)?;
        let mut unused: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        for face in faces.iter() {
            if !done.contains(face) {
                unused.add(face.clone());
            }
        }
        if unused.is_empty() {
            return Ok(());
        }
        self.warn_unused_faces(&unused)
    }

    fn warn_unused_faces(
        &mut self,
        unused: &IndexedMap<Shape, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let shells = make_internal_shells(unused)?;
        let warning = if shells.len() == 1 {
            shells.iter().next().expect("one shell").clone()
        } else {
            let builder = BRepBuilder::default();
            let mut compound = Compound::default();
            builder.make_compound(&mut compound);
            let mut compound = compound.into_shape();
            for shell in shells.iter() {
                builder.add(&mut compound, shell)?;
            }
            compound
        };
        let alert: AlertHandle = Arc::new(AlertSolidBuilderUnusedFaces::new(&warning));
        self.area.options.add_warning(&alert);
        Ok(())
    }
}

/// Adds the internal shells of the faces classified inside each solid; the faces placed.
fn add_internal_faces(in_parts: &InParts) -> Result<Map<Shape, ShapeMapHasher>, Failure> {
    let mut done: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    for index in 1..=in_parts.extent() {
        let faces = in_parts.find_from_index(index)?;
        if faces.is_empty() {
            continue;
        }
        let mut solid = in_parts.find_key(index)?.clone();
        let mut inside: IndexedMap<Shape, ShapeMapHasher> =
            IndexedMap::with_hasher(ShapeMapHasher, 1);
        for face in faces.iter() {
            inside.add(face.clone());
            done.add(face.clone());
        }
        for shell in make_internal_shells(&inside)?.iter() {
            BRepBuilder::default().add(&mut solid, shell)?;
        }
    }
    Ok(done)
}

fn empty_solid() -> Shape {
    let mut solid = Solid::default();
    BRepBuilder::default().make_solid(&mut solid);
    solid.into_shape()
}

/// Connected INTERNAL shells of the faces (`MakeInternalShells`).
pub(super) fn make_internal_shells(
    faces: &IndexedMap<Shape, ShapeMapHasher>,
) -> Result<List<Shape>, Failure> {
    let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
    for face in faces.iter() {
        topexp::map_shapes_and_ancestors(face, ShapeEnum::Edge, ShapeEnum::Face, &mut edge_faces)?;
    }
    let mut added: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    let mut shells = List::default();
    for face in faces.iter() {
        if !added.add(face.clone()) {
            continue;
        }
        shells.append(internal_shell(face, &edge_faces, &mut added)?);
    }
    Ok(shells)
}

fn internal_shell(
    seed: &Shape,
    edge_faces: &IndexedDataMapOfShapeListOfShape,
    added: &mut Map<Shape, ShapeMapHasher>,
) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell = shell.into_shape();
    builder.add(&mut shell, &seed.oriented(Orientation::Internal))?;
    let mut position = 0;
    while let Some(face) = ShapeIterator::new(&shell, true, true).nth(position) {
        position += 1;
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            for other in edge_faces.find_from_key(&edge)?.iter() {
                if added.add(other.clone()) {
                    builder.add(&mut shell, &other.oriented(Orientation::Internal))?;
                }
            }
        }
    }
    shell.set_closed(BRepTool::is_closed(&shell)?)?;
    Ok(shell)
}

/// Whether `shape` is inside the solid `solid` (`IsInside`): its first face is classified,
/// or the infinite point when it has no face.
pub(super) fn is_inside(shape: &Shape, solid: &Shape, context: &Context) -> Result<bool, Failure> {
    let solid = Solid::unchecked(solid.clone());
    let Some(face) = Iterator::next(&mut Explorer::new(
        shape,
        ShapeEnum::Face,
        ShapeEnum::Shape,
    )?) else {
        let classifier = context.solid_classifier(&solid)?;
        let mut classifier = classifier.write();
        classifier.perform_infinite_point(real_small())?;
        return Ok(classifier.state() == State::In);
    };
    let mut bounds: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    topexp::map_shapes_of_type(&solid, ShapeEnum::Edge, &mut bounds)?;
    let state = compute_state_face(
        &Face::unchecked(face),
        &solid,
        precision::confusion(),
        &bounds,
        context,
    )?;
    Ok(state == State::In)
}

/// The context created by `Perform`.
pub(super) fn required_context(handle: &Handle<Context>) -> Result<&Context, Failure> {
    crate::bop_algo::builder_area::required_context(handle, "BOPAlgo_BuilderSolid")
}
