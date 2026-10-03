//! Root-wire and hole assignment used by `ShapeFix_ComposeShell::MakeFacesOnPatch`.
mod assemble;
mod roots;

use super::{ShapeFixFace, ShapeFixFaceWireMap};
use crate::shape_build::ShapeBuildReShape;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Builder, Face, Shape};
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

/// Assigns patch wires to outside roots and their contained holes.
/// Example: `let faces = make_faces_on_patch(&surface, &mut loops, &source, &location, &context, true)?`.
pub(crate) fn make_faces_on_patch(
    surface: &Handle<dyn GeomSurface>,
    loops: &mut Sequence<Shape>,
    original_face: &Face,
    location: &Location,
    context: &Handle<ShapeBuildReShape>,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    if loops.len() == 1 {
        return make_single_loop_face(surface, loops.value(1)?, location, invert_edge_status);
    }
    let support = support_face(surface, location)?;
    let roots = roots::find_root_wires(loops, &support)?;
    let roots = roots::recover_lost_roots(loops, roots)?;
    assemble::assemble_root_faces(surface, loops, roots, original_face, location, context)
}

pub(super) fn support_face(
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
) -> Result<Face, Failure> {
    let mut face = Face::default();
    BRepBuilder::default().make_face_on_located_surface(
        &mut face,
        surface,
        location,
        precision::confusion(),
    )?;
    Ok(face)
}

fn make_single_loop_face(
    surface: &Handle<dyn GeomSurface>,
    shape: &Shape,
    location: &Location,
    invert_edge_status: bool,
) -> Result<Vec<Face>, Failure> {
    if shape.shape_type()? != ShapeEnum::Wire {
        return Ok(Vec::new());
    }
    let mut face = support_face(surface, location)?;
    Builder.add(&mut face, shape)?;
    orient_single_loop(&mut face, invert_edge_status)?;
    Ok(vec![face])
}

fn orient_single_loop(face: &mut Face, enabled: bool) -> Result<(), Failure> {
    if !enabled {
        return Ok(());
    }
    let fixer = ShapeFixFace::new(face.clone().into_shape());
    fixer.set_fix_add_natural_bound_mode(0);
    let mut map = ShapeFixFaceWireMap::default();
    fixer.fix_orientation_with_map(&mut map)?;
    *face = fixer.face()?;
    Ok(())
}

#[cfg(test)]
#[path = "make_faces_on_patch/tests.rs"]
mod tests;
