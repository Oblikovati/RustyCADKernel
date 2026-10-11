//! Native directional owner highlight: a detected point and two rotated sphere arcs.
use super::light_source_geometry::light_group;
use super::*;
use crate::{
    prs_mgr::*,
    prs3d::{Prs3dDrawer, Prs3dPointAspect},
    select_mgr::*,
};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
pub(super) fn light_owner_highlight(
    owner: &AisLightSourceOwner,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
    mode: i32,
) -> Result<(), Failure> {
    let light = owner.selectable().cast::<dyn AisLightSourceSource>();
    let Some(object) = light.get() else {
        return Ok(());
    };
    if object.light().light_type() != Graphic3dTypeOfLightSource::Directional
        || !object.light_source_base().parameters.read().draggable
    {
        return select_mgr_owner_hilight_with_color(owner, manager, style, mode);
    }
    directional_light_highlight(object, manager, style)
}
fn directional_light_highlight(
    object: &dyn AisLightSourceSource,
    manager: &Handle<PrsMgrPresentationManager>,
    style: &Handle<Prs3dDrawer>,
) -> Result<(), Failure> {
    let prs = object.get_hilight_presentation(manager)?;
    prepare_light_highlight(object, &prs, manager, style);
    let point = light_detected_point(object);
    if point.x() == f64::MAX {
        return Ok(());
    }
    draw_light_highlight(object, &prs, style, point)?;
    display_light_highlight(&prs, manager);
    Ok(())
}
fn prepare_light_highlight(
    object: &dyn AisLightSourceSource,
    prs: &Handle<dyn Graphic3dStructureSource>,
    manager: &PrsMgrPresentationManager,
    style: &Prs3dDrawer,
) {
    let layer = if style.z_layer() != -1 {
        style.z_layer()
    } else if manager.is_immediate_mode_on() {
        GRAPHIC3D_ZLAYER_ID_TOP
    } else {
        object.z_layer()
    };
    prs.clear(false);
    if prs.get_z_layer() != layer {
        prs.set_z_layer(layer);
    }
}
fn draw_light_highlight(
    object: &dyn AisLightSourceSource,
    prs: &Handle<dyn Graphic3dStructureSource>,
    style: &Prs3dDrawer,
    point: Point3,
) -> Result<(), Failure> {
    let points = Handle::new(Graphic3dArrayOfPoints::new(1, GRAPHIC3D_ARRAY_FLAGS_NONE)?);
    points.add_vertex_pnt(&point)?;
    let marker = Prs3dPointAspect::new(AspectTypeOfMarker::OPoint, style.color(), 3.0)?;
    light_group(prs, marker.aspect().cast(), &points.cast(), false)?;
    let arcs = highlight_arcs(object, point)?;
    let group = prs.new_group()?;
    group.add_primitive_array(&arcs.cast(), false);
    group.set_group_primitives_aspect(&style.arrow_aspect().aspect().cast());
    Ok(())
}
fn highlight_arcs(
    object: &dyn AisLightSourceSource,
    point: Point3,
) -> Result<Handle<Graphic3dArrayOfPolylines>, Failure> {
    let radius = object.size() * 0.5;
    let count = (f64::from(object.arc_size()) * 180.0 / (std::f64::consts::PI * radius)) as i32;
    let normal = Direction3::new(point.x(), point.y(), point.z())?;
    let plane = light_arc_plane(normal)?;
    let points = highlight_arc_points(point, plane, count)?;
    let arcs = Handle::new(Graphic3dArrayOfPolylines::new(
        count * 2,
        2,
        0,
        GRAPHIC3D_ARRAY_FLAGS_NONE,
    )?);
    append_highlight_arc(&arcs, &points, Some(Axis1::new(Point3::default(), normal)))?;
    append_highlight_arc(&arcs, &points, None)?;
    Ok(arcs)
}
fn highlight_arc_points(
    point: Point3,
    normal: Direction3,
    count: i32,
) -> Result<Vec<Point3>, Failure> {
    let axis = Axis1::new(Point3::default(), normal);
    (0..count)
        .map(|step| {
            point.rotated(
                axis,
                std::f64::consts::PI / 90.0 * f64::from(step - count / 2),
            )
        })
        .collect()
}
fn append_highlight_arc(
    array: &Graphic3dArrayOfPolylines,
    points: &[Point3],
    axis: Option<Axis1>,
) -> Result<(), Failure> {
    array.add_bound(points.len() as i32)?;
    for &point in points {
        let vertex = if let Some(axis) = axis {
            point.rotated(axis, std::f64::consts::FRAC_PI_2)?
        } else {
            point
        };
        array.add_vertex_pnt(&vertex)?;
    }
    Ok(())
}

fn light_detected_point(object: &dyn AisLightSourceSource) -> Point3 {
    object
        .light_source_base()
        .visual
        .read()
        .sensitive_sphere
        .last_detected_point()
}
fn display_light_highlight(
    prs: &Handle<dyn Graphic3dStructureSource>,
    manager: &PrsMgrPresentationManager,
) {
    if manager.is_immediate_mode_on() {
        manager.add_to_immediate_list(prs);
    } else {
        prs.display();
    }
}

fn light_arc_plane(normal: Direction3) -> Result<Direction3, Failure> {
    Ok(if Direction3::X.is_parallel(normal, precision::angular()) {
        Direction3::Y
    } else {
        Direction3::X.crossed(normal)?
    })
}
