use super::super::ShapeFixFace;
use super::plan::analyze;
use super::prepare::prepare;
use crate::shape_analysis::ShapeAnalysisSurface;
use crate::shape_build::ShapeBuildReShape;
use crate::shape_fix::{FixRoot, ShapeFixComposeShell, ShapeFixWire};
use rustycad_tkbrep::brep_tools::{self, ReShape};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::message::Msg;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::toploc::Location;

pub(in crate::shape_fix::face) fn fix(fixer: &ShapeFixFace) -> Result<bool, Failure> {
    let surface_handle = fixer.analysis_surface.read().clone();
    let Some(surface) = surface_handle.get() else {
        return Ok(false);
    };
    if !surface_is_closed(surface)? {
        return Ok(false);
    }
    let (face, context) = apply_current_face(fixer)?;
    let Some(mut plan) = analyze(&face, &surface.surface())? else {
        return Ok(false);
    };
    let Some(prepared) = prepare(&mut plan, fixer.precision())? else {
        return Ok(false);
    };
    compose_missing_seam(fixer, prepared.face, &prepared.grid, &context)?;
    cleanup_composed_result(fixer, &context)?;
    send_missing_seam_warning(fixer);
    Ok(true)
}

fn surface_is_closed(surface: &ShapeAnalysisSurface) -> Result<bool, Failure> {
    let live = surface.surface();
    let live = live.get().ok_or_else(null_surface)?;
    Ok(live.is_u_closed()? || live.is_v_closed()?)
}

fn apply_current_face(fixer: &ShapeFixFace) -> Result<(Face, Handle<ShapeBuildReShape>), Failure> {
    let (shape, context) = fixer
        .fix_root_data()
        .with(|fields| (fixer.face_state.read().clone(), fields.context.clone()));
    let applied = apply_reshape(&context, &shape)?;
    let face = topods::face(&applied)?;
    *fixer.face_state.write() = applied;
    Ok((face, context))
}

fn compose_missing_seam(
    fixer: &ShapeFixFace,
    face: Face,
    grid: &Handle<crate::shape_extend::ShapeExtendCompositeSurface>,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    let context = ensure_context(fixer, context);
    let composer = ShapeFixComposeShell::new();
    composer.init(grid, &Location::default(), &face, precision::confusion())?;
    composer.set_closed_mode(true);
    composer.set_context(&context);
    composer.set_max_tolerance(fixer.max_tolerance());
    composer.perform()?;
    store_composed_result(fixer, &composer.result(), &context)?;
    reset_analysis_surface(fixer, grid)?;
    Ok(())
}

fn ensure_context(
    fixer: &ShapeFixFace,
    context: &Handle<ShapeBuildReShape>,
) -> Handle<ShapeBuildReShape> {
    if !context.is_null() {
        return context.clone();
    }
    let created = Handle::new(ShapeBuildReShape::new());
    fixer.set_context(&created);
    created
}

fn store_composed_result(
    fixer: &ShapeFixFace,
    result: &Shape,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    *fixer.result.write() = result.clone();
    if let Some(context) = context.get() {
        let original = fixer.face_state.read().clone();
        context.replace(&original, result)?;
    }
    Ok(())
}

fn reset_analysis_surface(
    fixer: &ShapeFixFace,
    grid: &Handle<crate::shape_extend::ShapeExtendCompositeSurface>,
) -> Result<(), Failure> {
    let patch = grid.patch(1, 1)?;
    let analysis = Handle::new(ShapeAnalysisSurface::new(&patch)?);
    *fixer.analysis_surface.write() = analysis;
    Ok(())
}

fn cleanup_composed_result(
    fixer: &ShapeFixFace,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    let mut face_count = 0;
    let initial_result = fixer.result.read().clone();
    discard_empty_face_wires(fixer, &initial_result, context, &mut face_count)?;
    store_result(fixer, apply_reshape(context, &initial_result)?)?;
    cleanup_small_faces(fixer, context, face_count)?;
    let final_result = apply_reshape(context, &fixer.result.read())?;
    store_result(fixer, final_result)
}

fn discard_empty_face_wires(
    fixer: &ShapeFixFace,
    result: &Shape,
    context: &Handle<ShapeBuildReShape>,
    face_count: &mut i32,
) -> Result<(), Failure> {
    let mut faces = Explorer::new(result, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        let face = topods::face(faces.current())?;
        if clean_face_wires(fixer, &face, context)? {
            *face_count += 1;
        }
        faces.next()?;
    }
    Ok(())
}

fn clean_face_wires(
    fixer: &ShapeFixFace,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let wire_count = count_live_wires(fixer, face, context)?;
    if wire_count != 0 {
        return Ok(true);
    }
    remove_from_context(context, &face.clone().into_shape())?;
    Ok(false)
}

fn count_live_wires(
    fixer: &ShapeFixFace,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
) -> Result<i32, Failure> {
    let mut wire_count = 0;
    let mut wires = Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Wire,
        ShapeEnum::Shape,
    )?;
    while wires.more() {
        if fix_or_remove_wire(fixer, face, context, wires.current())? {
            wire_count += 1;
        }
        wires.next()?;
    }
    Ok(wire_count)
}

fn fix_or_remove_wire(
    fixer: &ShapeFixFace,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
    wire_shape: &Shape,
) -> Result<bool, Failure> {
    let wire = topods::wire(wire_shape)?;
    if fix_composed_wire(fixer, &wire, face, context)? {
        return Ok(true);
    }
    remove_from_context(context, wire_shape)?;
    Ok(false)
}

fn fix_composed_wire(
    fixer: &ShapeFixFace,
    wire: &rustycad_tkbrep::topods::Wire,
    face: &Face,
    context: &Handle<ShapeBuildReShape>,
) -> Result<bool, Failure> {
    let wire_fixer = ShapeFixWire::from_wire_face(wire, face, fixer.precision())?;
    wire_fixer.set_context(context);
    if wire_fixer.nb_edges() != 0 {
        wire_fixer.fix_small_all(true, fixer.precision())?;
    }
    Ok(wire_fixer.nb_edges() != 0)
}

fn cleanup_small_faces(
    fixer: &ShapeFixFace,
    context: &Handle<ShapeBuildReShape>,
    face_count: i32,
) -> Result<(), Failure> {
    let result = fixer.result.read().clone();
    let mut faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)?;
    while faces.more() {
        cleanup_small_face(fixer, context, faces.current(), face_count)?;
        faces.next()?;
    }
    Ok(())
}

fn cleanup_small_face(
    fixer: &ShapeFixFace,
    context: &Handle<ShapeBuildReShape>,
    candidate: &Shape,
    face_count: i32,
) -> Result<(), Failure> {
    let face = topods::face(&apply_reshape(context, candidate)?)?;
    if face.is_null() {
        return Ok(());
    }
    *fixer.face_state.write() = face.into_shape();
    remove_small_wires_for_multiple_faces(fixer, face_count)?;
    update_current_face(fixer, context)
}

fn remove_small_wires_for_multiple_faces(
    fixer: &ShapeFixFace,
    face_count: i32,
) -> Result<(), Failure> {
    if face_count > 1 {
        fixer.fix_small_area_wire(true)?;
    }
    Ok(())
}

fn update_current_face(
    fixer: &ShapeFixFace,
    context: &Handle<ShapeBuildReShape>,
) -> Result<(), Failure> {
    let current = topods::face(&apply_reshape(context, &fixer.face_state.read())?)?;
    if current.is_null() {
        return Ok(());
    }
    brep_tools::update_face(&current)?;
    *fixer.face_state.write() = current.into_shape();
    Ok(())
}

fn store_result(fixer: &ShapeFixFace, result: Shape) -> Result<(), Failure> {
    *fixer.result.write() = result;
    Ok(())
}

fn apply_reshape(context: &Handle<ShapeBuildReShape>, shape: &Shape) -> Result<Shape, Failure> {
    let Some(context) = context.get() else {
        return Ok(shape.clone());
    };
    context.apply(shape, ShapeEnum::Shape)
}

fn remove_from_context(context: &Handle<ShapeBuildReShape>, shape: &Shape) -> Result<(), Failure> {
    if let Some(context) = context.get() {
        context.remove(shape)?;
    }
    Ok(())
}

fn send_missing_seam_warning(fixer: &ShapeFixFace) {
    let message = Msg::from_key(&AsciiString::from("FixAdvFace.FixMissingSeam.MSG0"));
    fixer.send_warning_on_shape(&message);
}

fn null_surface() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_Face::FixMissingSeam: null analysis surface; expected a live closed surface",
    )
}
