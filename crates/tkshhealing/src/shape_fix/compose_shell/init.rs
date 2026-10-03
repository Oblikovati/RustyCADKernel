use super::ShapeFixComposeShell;
use crate::shape_extend::ShapeExtendCompositeSurface;
use crate::shape_fix::FixRoot;
use rustycad_tkbrep::{brep::BRepTool, brep_tools, topods::Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomElementarySurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::toploc::Location;

pub(super) fn initialize(
    composer: &ShapeFixComposeShell,
    grid: &Handle<ShapeExtendCompositeSurface>,
    location: &Location,
    face: &Face,
    fix_precision: f64,
) -> Result<(), Failure> {
    set_grid(composer, grid)?;
    initialize_face_state(composer, location, face, fix_precision)?;
    initialize_resolutions(composer, grid)?;
    Ok(())
}

fn set_grid(
    composer: &ShapeFixComposeShell,
    grid: &Handle<ShapeExtendCompositeSurface>,
) -> Result<(), Failure> {
    composer.fields.write().grid = grid.clone();
    let live = grid.get().ok_or_else(null_grid)?;
    let u_closed = live.is_u_closed()?;
    let v_closed = live.is_v_closed()?;
    let u_last = live.u_joint_value(live.nb_u_patches()? + 1)?;
    let u_first = live.u_joint_value(1)?;
    let v_last = live.v_joint_value(live.nb_v_patches()? + 1)?;
    let v_first = live.v_joint_value(1)?;
    let mut fields = composer.fields.write();
    fields.u_closed = u_closed;
    fields.v_closed = v_closed;
    fields.u_period = u_last - u_first;
    fields.v_period = v_last - v_first;
    Ok(())
}

fn initialize_face_state(
    composer: &ShapeFixComposeShell,
    location: &Location,
    face: &Face,
    fix_precision: f64,
) -> Result<(), Failure> {
    let (surface, _) = BRepTool::surface_with_location(face)?;
    let (u_closed, v_closed) = closed_directions(composer, &surface, face)?;
    store_face_state(composer, location, face, u_closed, v_closed);
    composer.set_precision(fix_precision);
    Ok(())
}

fn store_face_state(
    composer: &ShapeFixComposeShell,
    location: &Location,
    face: &Face,
    u_closed: bool,
    v_closed: bool,
) {
    {
        let mut fields = composer.fields.write();
        fields.u_closed = u_closed;
        fields.v_closed = v_closed;
        fields.location = location.clone();
        let mut forward_face = face.clone();
        forward_face.set_orientation(rustycad_tkg3d::top_abs::Orientation::Forward);
        fields.face = forward_face;
        fields.orientation = face.orientation();
        fields.status = 0;
    }
}

fn closed_directions(
    composer: &ShapeFixComposeShell,
    surface: &Handle<dyn GeomSurface>,
    face: &Face,
) -> Result<(bool, bool), Failure> {
    let fields = composer.fields.read();
    let mut u_closed = fields.u_closed;
    let mut v_closed = fields.v_closed;
    drop(fields);
    if is_elementary_surface(surface) {
        u_closed &= surface.get().ok_or_else(null_surface)?.is_u_closed()?;
        v_closed &= surface.get().ok_or_else(null_surface)?.is_v_closed()?;
        return Ok((u_closed, v_closed));
    }
    if !u_closed && !v_closed {
        return Ok((false, false));
    }
    let bounds = closure_bounds(surface, face)?;
    check_closure(u_closed, v_closed, surface, bounds)
}

fn is_elementary_surface(surface: &Handle<dyn GeomSurface>) -> bool {
    surface.is_kind(<dyn GeomElementarySurface as RuntimeTyped>::static_type())
}

fn closure_bounds(
    surface: &Handle<dyn GeomSurface>,
    face: &Face,
) -> Result<(f64, f64, f64, f64), Failure> {
    let mut bounds = surface.get().ok_or_else(null_surface)?.bounds();
    if [bounds.0, bounds.1, bounds.2, bounds.3]
        .iter()
        .any(|value| precision::is_infinite(*value))
    {
        let face_bounds = brep_tools::uv_bounds(face)?;
        bounds = substitute_infinite_bounds(bounds, face_bounds);
    }
    Ok(bounds)
}

fn substitute_infinite_bounds(
    bounds: (f64, f64, f64, f64),
    fallback: (f64, f64, f64, f64),
) -> (f64, f64, f64, f64) {
    let values = [bounds.0, bounds.1, bounds.2, bounds.3];
    let defaults = [fallback.0, fallback.1, fallback.2, fallback.3];
    let mut result = values;
    for index in 0..values.len() {
        if precision::is_infinite(values[index]) {
            result[index] = defaults[index];
        }
    }
    (result[0], result[1], result[2], result[3])
}

fn check_closure(
    u_closed: bool,
    v_closed: bool,
    surface: &Handle<dyn GeomSurface>,
    bounds: (f64, f64, f64, f64),
) -> Result<(bool, bool), Failure> {
    let live = surface.get().ok_or_else(null_surface)?;
    Ok((
        closed_along_u(live, bounds, u_closed)?,
        closed_along_v(live, bounds, v_closed)?,
    ))
}

fn closed_along_u(
    surface: &dyn GeomSurface,
    (u0, u1, v0, v1): (f64, f64, f64, f64),
    enabled: bool,
) -> Result<bool, Failure> {
    if !enabled {
        return Ok(false);
    }
    endpoints_match(surface, (u0, (v0 + v1) / 2.0), (u1, (v0 + v1) / 2.0))
}

fn closed_along_v(
    surface: &dyn GeomSurface,
    (u0, u1, v0, v1): (f64, f64, f64, f64),
    enabled: bool,
) -> Result<bool, Failure> {
    if !enabled {
        return Ok(false);
    }
    endpoints_match(surface, ((u0 + u1) / 2.0, v0), ((u0 + u1) / 2.0, v1))
}

fn endpoints_match(
    surface: &dyn GeomSurface,
    first: (f64, f64),
    last: (f64, f64),
) -> Result<bool, Failure> {
    let p0 = surface.value(first.0, first.1)?;
    let p1 = surface.value(last.0, last.1)?;
    Ok(p0.distance(&p1) <= precision::confusion() * 10.0)
}

fn initialize_resolutions(
    composer: &ShapeFixComposeShell,
    grid: &Handle<ShapeExtendCompositeSurface>,
) -> Result<(), Failure> {
    let (u, v) = patch_resolutions(grid)?;
    let mut fields = composer.fields.write();
    fields.u_resolution = u;
    fields.v_resolution = v;
    Ok(())
}

fn patch_resolutions(grid: &Handle<ShapeExtendCompositeSurface>) -> Result<(f64, f64), Failure> {
    let mut u_min = precision::infinite();
    let mut v_min = precision::infinite();
    let live = grid.get().ok_or_else(null_grid)?;
    for i in 1..=live.nb_u_patches()? {
        for j in 1..=live.nb_v_patches()? {
            let (u_range, v_range, bounds, adaptor) = patch_resolution_inputs(live, i, j)?;
            u_min = smaller_positive(
                u_min,
                adaptor.u_resolution(1.0)? * u_range / (bounds.1 - bounds.0),
            );
            v_min = smaller_positive(
                v_min,
                adaptor.v_resolution(1.0)? * v_range / (bounds.3 - bounds.2),
            );
        }
    }
    Ok((fallback_resolution(u_min), fallback_resolution(v_min)))
}

fn patch_resolution_inputs(
    grid: &ShapeExtendCompositeSurface,
    i: i32,
    j: i32,
) -> Result<(f64, f64, (f64, f64, f64, f64), GeomAdaptorSurface), Failure> {
    let u_range = grid.u_joint_value(i + 1)? - grid.u_joint_value(i)?;
    let v_range = grid.v_joint_value(j + 1)? - grid.v_joint_value(j)?;
    let surface = grid.patch(i, j)?;
    let bounds = surface.get().ok_or_else(null_surface)?.bounds();
    let adaptor = GeomAdaptorSurface::with_surface(&surface)?;
    Ok((u_range, v_range, bounds, adaptor))
}

fn smaller_positive(current: f64, candidate: f64) -> f64 {
    if candidate > 0.0 && current > candidate {
        candidate
    } else {
        current
    }
}

fn fallback_resolution(value: f64) -> f64 {
    if value == precision::infinite() {
        precision::parametric(1.0)
    } else {
        value
    }
}

fn null_grid() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::Init: null grid",
    )
}

fn null_surface() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "ShapeFix_ComposeShell::Init: null surface",
    )
}
