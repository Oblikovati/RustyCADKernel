//! Native click target preserves the closest quarter-turn up direction and fits selection/scene.
use super::AisViewCubeSource;
use crate::{ais::AisInteractiveObjectMembers, v3d::V3dView};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{
    Failure, Handle, HandleFieldRef, HandleReference, exceptions::null_object,
};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};
use rustycad_tkservice::graphic3d::*;
/// Fit selected geometry, falling back to the scene and preserving an empty scene; `ais_view_cube_view_fit_all(&cube,&view,&camera)?`.
pub fn ais_view_cube_view_fit_all<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    view: &dyn HandleReference<V3dView>,
    camera: &dyn HandleReference<Graphic3dCamera>,
) -> Result<(), Failure> {
    let selected = cube.interaction_base().fields.read().fit_selected;
    let mut bounds = cube_fit_bounds(cube, view, selected)?;
    if selected && bounds.is_void() {
        bounds = cube_view_reference_bounds(view)?;
    }
    if !bounds.is_void() {
        fit_cube_reference_bounds(view, camera, &bounds)?;
    }
    Ok(())
}
pub(super) fn cube_oriented_end_camera<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    view: &Handle<V3dView>,
    orientation: crate::v3d::V3dTypeOfOrientation,
) -> Result<(), Failure> {
    let (start, end, reset) = {
        let fields = cube.interaction_base().fields.read();
        (
            fields.start.observe(),
            fields.end.observe(),
            fields.reset_up,
        )
    };
    start.copy(&view.camera());
    end.copy(&view.camera());
    set_cube_end_orientation(cube, view, &end, orientation)?;
    align_cube_end_up(&start, &end, reset)?;
    let endpoint = HandleFieldRef::new(&cube.interaction_base().fields, cube_end_camera_slot);
    cube.view_fit_all(view, &endpoint)
}
fn set_cube_end_orientation<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    view: &V3dView,
    end: &Handle<Graphic3dCamera>,
    orientation: crate::v3d::V3dTypeOfOrientation,
) -> Result<(), Failure> {
    let backup = view.camera();
    let immediate = view.set_immediate_update(false);
    let result = view.set_camera(end).and_then(|()| {
        view.set_proj_orientation(
            orientation,
            cube.view_cube_presentation_base().settings.read().is_y_up,
        )
    });
    let restored = view.set_camera(&backup);
    view.set_immediate_update(immediate);
    result?;
    restored
}
fn closest_cube_up(start: &Graphic3dCamera, end: &Graphic3dCamera) -> Result<Direction3, Failure> {
    let axis = Axis1::new(Point3::default(), end.direction());
    let mut best_angle = precision::infinite();
    let mut best_up = end.up();
    for quarter in 0..4 {
        let candidate = end
            .up()
            .rotated(axis, f64::from(quarter) * std::f64::consts::FRAC_PI_2)?;
        let angle = candidate.angle(&start.up());
        if best_angle > angle {
            best_angle = angle;
            best_up = candidate;
        }
    }
    Ok(best_up)
}

fn cube_fit_bounds<C: AisViewCubeSource + ?Sized>(
    cube: &C,
    view: &dyn HandleReference<V3dView>,
    selected: bool,
) -> Result<rustycad_tkmath::bnd::BoundingBox3, Failure> {
    if !selected {
        return cube_view_reference_bounds(view);
    }
    let context = cube.get_context();
    let context = context.get().ok_or_else(|| {
        null_object("AIS_ViewCube context=null; expected an attached interactive context")
    })?;
    context.bounding_box_of_selection(observed_cube_fit_view(view)?.get())
}

fn cube_end_camera_slot(fields: &super::AisViewCubeInteractionFields) -> &Handle<Graphic3dCamera> {
    // Native viewFitAll receives myEndState's actual slot, whose replacement remains visible.
    &fields.end
}
fn observed_cube_fit_view(
    view: &dyn HandleReference<V3dView>,
) -> Result<rustycad_tkernel::standard::HandleObserver<V3dView>, Failure> {
    let observed = view.observe();
    if observed.is_null() {
        return Err(null_object(
            "AIS_ViewCube fit view=null; expected a live view handle",
        ));
    }
    Ok(observed)
}
fn fit_cube_reference_bounds(
    view: &dyn HandleReference<V3dView>,
    camera: &dyn HandleReference<Graphic3dCamera>,
    bounds: &rustycad_tkmath::bnd::BoundingBox3,
) -> Result<(), Failure> {
    let view = observed_cube_fit_view(view)?;
    let observed = camera.observe();
    let camera = observed.get().ok_or_else(|| {
        null_object("AIS_ViewCube fit camera=null; expected a live camera handle")
    })?;
    view.fit_min_max(camera, bounds, 0.01, 10.0 * precision::confusion(), true)
        .map(|_| ())
}

fn cube_view_reference_bounds(
    view: &dyn HandleReference<V3dView>,
) -> Result<rustycad_tkmath::bnd::BoundingBox3, Failure> {
    let observed = observed_cube_fit_view(view)?;
    observed
        .get()
        .expect("validated non-null ViewCube fit view")
        .view()
        .min_max_values(false)
}

fn align_cube_end_up(
    start: &Graphic3dCamera,
    end: &Graphic3dCamera,
    reset: bool,
) -> Result<(), Failure> {
    if reset
        || end
            .direction()
            .is_equal(&start.direction(), precision::angular())
    {
        return Ok(());
    }
    end.set_up(&closest_cube_up(start, end)?);
    Ok(())
}
