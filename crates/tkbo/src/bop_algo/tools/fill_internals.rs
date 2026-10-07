//! `BOPAlgo_Tools::FillInternals`: internal parts put into the solids containing them.
use crate::bop_tools::{compute_state_by_one_point, make_connexity_compounds};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Compound, Shape, ShapeIterator, Shell, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// Shapes and their image lists (`theImages`).
type Images = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// Puts the vertices, edges and faces of `parts` (or their images) that are not already in
/// the solids into the solid containing them: vertices and edges as INTERNAL, faces as
/// INTERNAL shells of connected faces (`FillInternals`).
/// `fill_internals(&solids, &parts, &images, &context)?`.
pub fn fill_internals(
    solids: &List<Shape>,
    parts: &List<Shape>,
    images: &Images,
    context: &Context,
) -> Result<(), Failure> {
    if solids.is_empty() || parts.is_empty() {
        return Ok(());
    }
    let own = solid_sub_shapes(solids)?;
    let mut candidates = split_parts(parts, images, &own)?;
    let mut inside_faces: Vec<(Shape, Vec<Shape>)> = Vec::new();
    for solid in solids.iter() {
        if solid.shape_type()? != ShapeEnum::Solid {
            continue;
        }
        let faces = settle_parts(solid, &mut candidates, context)?;
        if !faces.is_empty() {
            inside_faces.push((solid.clone(), faces));
        }
    }
    for (solid, faces) in inside_faces {
        add_internal_shells(&solid, &faces)?;
    }
    Ok(())
}

/// The vertices, edges and faces of the solids (`aMSSolids`).
fn solid_sub_shapes(solids: &List<Shape>) -> Result<IndexedMap<Shape, ShapeMapHasher>, Failure> {
    let mut own: IndexedMap<Shape, ShapeMapHasher> = IndexedMap::with_hasher(ShapeMapHasher, 1);
    for solid in solids.iter() {
        if solid.shape_type()? == ShapeEnum::Solid {
            for kind in [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face] {
                topexp::map_shapes_of_type(solid, kind, &mut own)?;
            }
        }
    }
    Ok(own)
}

/// The vertices, edges and faces of the parts (containers explored), replaced by their
/// images, without the sub-shapes of the solids.
fn split_parts(
    parts: &List<Shape>,
    images: &Images,
    own: &IndexedMap<Shape, ShapeMapHasher>,
) -> Result<Vec<Shape>, Failure> {
    let mut input: Vec<Shape> = parts.iter().cloned().collect();
    let mut result = Vec::new();
    let mut position = 0;
    while position < input.len() {
        let part = input[position].clone();
        position += 1;
        match part.shape_type()? {
            ShapeEnum::Vertex | ShapeEnum::Edge | ShapeEnum::Face => match images.seek(&part) {
                Some(splits) => {
                    result.extend(splits.iter().filter(|split| !own.contains(split)).cloned())
                }
                None if !own.contains(&part) => result.push(part),
                None => {}
            },
            _ => input.extend(ShapeIterator::new(&part, true, true)),
        }
    }
    Ok(result)
}

/// Adds the vertices and edges inside the solid as INTERNAL and returns the faces inside
/// it; the parts placed leave `candidates`.
fn settle_parts(
    solid: &Shape,
    candidates: &mut Vec<Shape>,
    context: &Context,
) -> Result<Vec<Shape>, Failure> {
    let mut target = solid.clone();
    let mut faces = Vec::new();
    let mut position = 0;
    while position < candidates.len() {
        let part = candidates[position].clone();
        let state = compute_state_by_one_point(
            &part,
            &Solid::unchecked(solid.clone()),
            precision::confusion(),
            context,
        )?;
        if state != State::In {
            position += 1;
            continue;
        }
        if part.shape_type()? == ShapeEnum::Face {
            faces.push(part);
        } else {
            BRepBuilder::default().add(&mut target, &part.oriented(Orientation::Internal))?;
        }
        candidates.remove(position);
    }
    Ok(faces)
}

/// One INTERNAL shell per connected block of the faces, added to the solid.
fn add_internal_shells(solid: &Shape, faces: &[Shape]) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut compound = compound.into_shape();
    for face in faces {
        builder.add(&mut compound, face)?;
    }
    let mut blocks = Vec::new();
    make_connexity_compounds(&compound, ShapeEnum::Edge, ShapeEnum::Face, &mut blocks)?;
    let mut target = solid.clone();
    for block in blocks {
        let mut shell = Shell::default();
        builder.make_shell(&mut shell);
        let mut shell = shell.into_shape();
        for face in Explorer::new(&block, ShapeEnum::Face, ShapeEnum::Shape)? {
            builder.add(&mut shell, &face.oriented(Orientation::Internal))?;
        }
        builder.add(&mut target, &shell)?;
    }
    Ok(())
}
