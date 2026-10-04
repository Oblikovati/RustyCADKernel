use super::{EdgeMergeOptions, global_merge, rebuild_planar_face_pcurves};
use crate::shape_fix::{FixRoot, ShapeFixFace};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkmath::geom_abs::SurfaceType;

/// Merges face boundaries under one reshape context for the complete input.
pub(super) fn merge_nested_children(
    shape: &Shape,
    options: &EdgeMergeOptions<'_>,
) -> Result<Shape, Failure> {
    let merged = global_merge::merge_edges(shape, options)?;
    let faces = unique_faces(shape)?;
    for face in faces {
        fix_face_in_context(&face, options)?;
    }
    options.context.apply(&merged, ShapeEnum::Shape)
}

fn unique_faces(shape: &Shape) -> Result<Vec<Face>, Failure> {
    let mut unique = Vec::new();
    for candidate in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = rustycad_tkbrep::topods::face(&candidate)?;
        if !unique.iter().any(|known: &Face| known.is_same(&face)) {
            unique.push(face);
        }
    }
    Ok(unique)
}

fn fix_face_in_context(original: &Face, options: &EdgeMergeOptions<'_>) -> Result<(), Failure> {
    let applied = options
        .context
        .apply(&original.clone().into_shape(), ShapeEnum::Shape)?;
    if applied.is_same(&original.clone().into_shape()) {
        return Ok(());
    }
    let fixed = fix_changed_face(&applied, options)?;
    options.context.replace(&applied, &fixed)?;
    Ok(())
}

fn fix_changed_face(shape: &Shape, options: &EdgeMergeOptions<'_>) -> Result<Shape, Failure> {
    if !options.safe_input_mode {
        rebuild_pcurves_if_planar(shape)?;
    }
    let fixer = ShapeFixFace::new(shape.clone());
    configure_face_fixer(&fixer, options)?;
    fixer.perform()?;
    let fixed = fixer.result();
    if options.safe_input_mode {
        return rebuild_pcurves_if_planar(&fixed);
    }
    Ok(fixed)
}

fn configure_face_fixer(
    fixer: &ShapeFixFace,
    options: &EdgeMergeOptions<'_>,
) -> Result<(), Failure> {
    set_face_fix_modes(fixer);
    set_wire_fix_modes(fixer.fix_wire_tool().get().ok_or_else(null_wire_tool)?);
    set_face_fix_tolerances(fixer);
    if options.safe_input_mode {
        fixer.set_context(options.context);
    }
    Ok(())
}

fn set_face_fix_modes(fixer: &ShapeFixFace) {
    fixer.set_fix_orientation_mode(0);
    fixer.set_fix_add_natural_bound_mode(0);
    fixer.set_fix_intersecting_wires_mode(0);
    fixer.set_fix_loop_wires_mode(0);
    fixer.set_fix_split_face_mode(0);
    fixer.set_fix_periodic_degenerated_mode(0);
}

fn set_wire_fix_modes(fixer: &crate::shape_fix::ShapeFixWire) {
    fixer.set_fix_self_intersection_mode(0);
    fixer.set_fix_nonadjacent_intersecting_edges_mode(0);
    fixer.set_fix_lacking_mode(0);
    fixer.set_fix_notched_edges_mode(0);
    fixer.set_topology_mode(false);
    fixer.set_remove_loop_mode(0);
    fixer.set_fix_gaps_by_ranges(false);
    fixer.set_fix_small_mode(0);
}

fn set_face_fix_tolerances(fixer: &ShapeFixFace) {
    let precision = rustycad_tkernel::precision::confusion();
    fixer.set_precision(precision);
    fixer.set_min_tolerance(precision);
    fixer.set_max_tolerance(1.0_f64.max(precision * 1000.0));
}

fn null_wire_tool() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "ShapeUpgrade_UnifySameDomain: null face wire tool; expected ShapeFix_Wire",
        "",
    )
}

fn rebuild_pcurves_if_planar(shape: &Shape) -> Result<Shape, Failure> {
    let face = rustycad_tkbrep::topods::face(shape)?;
    if BRepAdaptorSurface::with_face(&face, false)?.get_type()? != SurfaceType::Plane {
        return Ok(shape.clone());
    }
    rebuild_planar_face_pcurves(shape)
}
