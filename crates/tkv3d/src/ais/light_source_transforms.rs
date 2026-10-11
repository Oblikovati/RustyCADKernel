//! Native light/world binding, including retained persistence identity and zoom anchors.
use super::{AisLightSourceMembers, AisLightSourceSource};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle},
};
use rustycad_tkmath::{
    gp::{Axis2, Axis3, Direction3, Point3, Transform3},
    toploc::Datum3,
};
use rustycad_tkservice::{aspect::AspectTypeOfTriedronPosition, graphic3d::*};
type Flags = Graphic3dTransModeFlags;
type Kind = Graphic3dTypeOfLightSource;

/// Update persistence from the live light; `ais_light_source_update_persistence(&object)?`.
pub fn ais_light_source_update_persistence<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let light = object.light();
    let previous = object.presentable_base().read().transform_persistence;
    let mode = persistence_mode(
        light.light_type(),
        light.is_headlight(),
        object.is_zoomable(),
    );
    let persistence = choose_persistence(&light, previous, mode)?;
    object.set_transform_persistence(&persistence);
    Ok(())
}
fn persistence_mode(kind: Kind, headlight: bool, zoomable: bool) -> Flags {
    match (kind, headlight, zoomable) {
        (Kind::Ambient, _, true) => Flags::NONE,
        (Kind::Ambient, _, false) => Flags::TRIEDRON_PERS,
        (Kind::Directional, false, true) => Flags::NONE,
        (Kind::Directional, true, true) => Flags::CAMERA_PERS,
        (Kind::Directional, true, false) => Flags::TMF_2D,
        (Kind::Directional, false, false) => Flags::TRIEDRON_PERS,
        (_, true, _) => Flags::CAMERA_PERS,
        (_, false, true) => Flags::NONE,
        (_, false, false) => Flags::ZOOM_PERS,
    }
}
fn persistence_matches(
    light: &Graphic3dCLight,
    previous: &Handle<dyn Graphic3dTransformPersSource>,
    mode: Flags,
) -> bool {
    previous.get().is_some_and(|pers| {
        if light.light_type() == Kind::Ambient {
            pers.is_trihedron_or_2d()
        } else {
            pers.mode() == mode
        }
    })
}
fn choose_persistence(
    light: &Graphic3dCLight,
    previous: Handle<dyn Graphic3dTransformPersSource>,
    mode: Flags,
) -> Result<Handle<dyn Graphic3dTransformPersSource>, Failure> {
    if mode == Flags::NONE {
        return Ok(Handle::default());
    }
    let matches = persistence_matches(light, &previous, mode);
    let persistence = if matches {
        previous
    } else {
        make_persistence(light, mode)?
    };
    if mode == Flags::ZOOM_PERS {
        persistence.set_anchor_point(&light.position())?;
    }
    Ok(persistence)
}
fn make_persistence(
    light: &Graphic3dCLight,
    mode: Flags,
) -> Result<Handle<dyn Graphic3dTransformPersSource>, Failure> {
    let persistence = if mode == Flags::CAMERA_PERS {
        Graphic3dTransformPers::new(mode)?
    } else if mode == Flags::ZOOM_PERS {
        Graphic3dTransformPers::with_point(mode, &light.position())?
    } else {
        Graphic3dTransformPers::with_corner(
            mode,
            AspectTypeOfTriedronPosition::LEFT_UPPER,
            Vec2::new(50, 50),
        )?
    };
    Ok(Handle::new(persistence).cast())
}
/// Derive the local transform without propagating it back into the light; `ais_light_source_update_local_transform(&object)?`.
pub fn ais_light_source_update_local_transform<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Result<(), Failure> {
    let light = object.light();
    let transform = light_local_transform(&light, object.is_zoomable())?;
    object
        .presentable_base()
        .fields
        .write()
        .local_transformation = transform
        .map(|trsf| Handle::new(Datum3::new(trsf)))
        .unwrap_or_default();
    object.update_transformation()
}
fn light_local_transform(
    light: &Graphic3dCLight,
    zoomable: bool,
) -> Result<Option<Transform3>, Failure> {
    match light.light_type() {
        Kind::Ambient | Kind::Positional if zoomable => Ok(Some(translated(light.position()))),
        Kind::Ambient | Kind::Positional => Ok(None),
        Kind::Directional | Kind::Spot => Ok(Some(oriented(light, local_origin(light, zoomable))?)),
    }
}
fn local_origin(light: &Graphic3dCLight, zoomable: bool) -> Point3 {
    if !zoomable || (light.light_type() == Kind::Directional && light.is_headlight()) {
        return Point3::default();
    }
    light.position()
}
fn translated(position: Point3) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation_between(Point3::default(), position);
    transform
}
fn oriented(light: &Graphic3dCLight, position: Point3) -> Result<Transform3, Failure> {
    let frame = Axis2::new(position, light.direction()?.reversed())?;
    let mut transform = Transform3::default();
    transform.set_transformation_between(Axis3::from_axis2(frame), Axis3::default());
    Ok(transform)
}
/// Propagate a local transform into the live light, then refresh aspects and persistence.
/// Example: `ais_light_source_set_local_transform(&object, &Handle::new(Datum3::new(transform)))?`.
pub fn ais_light_source_set_local_transform<O: AisLightSourceSource + ?Sized>(
    object: &O,
    datum: &Handle<Datum3>,
) -> Result<(), Failure> {
    let transform = datum
        .get()
        .map(|datum| *datum.transformation())
        .unwrap_or_default();
    propagate_light_transform(object, transform)?;
    object
        .presentable_base()
        .fields
        .write()
        .local_transformation = Handle::new(Datum3::new(transform));
    object.update_transformation()?;
    object.update_light_aspects()?;
    ais_light_source_update_persistence(object)
}
fn propagate_light_transform<O: AisLightSourceSource + ?Sized>(
    object: &O,
    transform: Transform3,
) -> Result<(), Failure> {
    let light = object.light();
    match light.light_type() {
        Kind::Ambient => Ok(()),
        Kind::Directional => propagate_directional(&light, transform, object.is_zoomable()),
        Kind::Positional | Kind::Spot => propagate_positional(&light, transform),
    }
}
fn propagate_directional(
    light: &Graphic3dCLight,
    transform: Transform3,
    zoomable: bool,
) -> Result<(), Failure> {
    light.set_direction(&Direction3::Z.reversed().transformed(transform))?;
    if zoomable {
        light.set_display_position(&Point3::default().transformed(transform))?;
    }
    Ok(())
}
fn propagate_positional(light: &Graphic3dCLight, transform: Transform3) -> Result<(), Failure> {
    light.set_position(&Point3::default().transformed(transform))?;
    if light.light_type() == Kind::Spot {
        light.set_direction(&Direction3::Z.reversed().transformed(transform))?;
    }
    Ok(())
}
