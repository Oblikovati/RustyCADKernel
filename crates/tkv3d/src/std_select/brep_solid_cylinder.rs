//! Native whole-solid cone/cylinder recognition preserves plane-normal checks and the coplanar-axis reversal quirk.
use super::StdSelectBrepSelectionTool;
use crate::{
    select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelection},
    select3d::Select3DSensitiveCylinder,
};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorSurface,
    topods::{self, Face},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, catch_native_failure},
};
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, geom::*};
use rustycad_tkmath::{
    gp::{Axis3, Cone3, Transform3, xoy},
    toploc::Location,
};
struct SolidSelectionSurface {
    face: Face,
    surface: Handle<dyn GeomSurface>,
    location: Location,
}
impl StdSelectBrepSelectionTool {
    /// Recognize actual two/three-face conical solids before per-face fallback; `Self::get_sensitive_for_cylinder(&faces,&owner,&selection)?`.
    pub fn get_sensitive_for_cylinder(
        faces: &IndexedMapOfShape,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        selection: &Handle<SelectMgrSelection>,
    ) -> Result<bool, Failure> {
        if faces.extent() != 2 && faces.extent() != 3 {
            return Ok(false);
        }
        catch_native_failure(|| recognize_solid_cylinder(faces, owner, selection))
    }
}
fn recognize_solid_cylinder(
    faces: &IndexedMapOfShape,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    selection: &Handle<SelectMgrSelection>,
) -> Result<bool, Failure> {
    let mut surfaces = Vec::new();
    for index in 1..=faces.extent() {
        let face = topods::face(faces.find_key(index)?)?;
        let (surface, location) = BRepTool::surface_with_location(&face)?;
        surfaces.push(SolidSelectionSurface {
            face,
            surface,
            location,
        });
    }
    let primitive = if surfaces.len() == 2 {
        recognize_pointed_cone(&surfaces, owner)?
    } else {
        recognize_capped_cylinder(&surfaces, owner)?
    };
    let Some(primitive) = primitive else {
        return Ok(false);
    };
    selection.add(&Handle::new(primitive).cast())?;
    Ok(true)
}
fn recognize_pointed_cone(
    surfaces: &[SolidSelectionSurface],
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Select3DSensitiveCylinder>, Failure> {
    let first = surfaces[0].surface.cast::<GeomConicalSurface>();
    let index = usize::from(first.is_null());
    let cone = surfaces[index].surface.cast::<GeomConicalSurface>();
    let plane = surfaces[1 - index].surface.cast::<GeomPlane>();
    if cone.is_null() || plane.is_null() {
        return Ok(None);
    }
    if !plane
        .position()
        .direction()
        .is_equal(&cone.position().direction(), precision::angular())
    {
        return Ok(None);
    }
    let placed = BRepAdaptorSurface::with_face(&surfaces[index].face, true)?.cone()?;
    let radius = placed.ref_radius();
    let height = pointed_cone_height(placed, &plane, &surfaces[1 - index].location);
    let top = if radius != 0. {
        0.
    } else {
        placed.semi_angle().tan() * height
    };
    Ok(Some(solid_cylinder_entity(
        owner,
        (radius, top),
        height,
        placed.position(),
    )))
}
fn pointed_cone_height(cone: Cone3, plane: &GeomPlane, location: &Location) -> f64 {
    if cone.ref_radius() != 0. {
        return cone.ref_radius() / cone.semi_angle().tan().abs();
    }
    cone.location()
        .distance(&plane.location().transformed(location.transformation()))
}
fn solid_cylinder_entity(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    radii: (f64, f64),
    height: f64,
    position: Axis3,
) -> Select3DSensitiveCylinder {
    let mut transform = Transform3::default();
    transform.set_transformation_between(position, Axis3::from_axis2(xoy()));
    Select3DSensitiveCylinder::new(owner, radii.0, radii.1, height, transform, false)
}
struct CappedSolidSurfaces<'a> {
    quadric: Option<&'a SolidSelectionSurface>,
    cone: Handle<GeomConicalSurface>,
    cylinder: Handle<GeomCylindricalSurface>,
    planes: Vec<(Handle<GeomPlane>, &'a Location)>,
}
fn classify_capped_surfaces(surfaces: &[SolidSelectionSurface]) -> CappedSolidSurfaces<'_> {
    let mut classified = CappedSolidSurfaces {
        quadric: None,
        cone: Handle::default(),
        cylinder: Handle::default(),
        planes: Vec::new(),
    };
    for surface in surfaces {
        if classified.quadric.is_none() && classify_solid_quadric(surface, &mut classified) {
            continue;
        }
        if classified.planes.len() == 2 {
            continue;
        }
        let plane = surface.surface.cast::<GeomPlane>();
        if !plane.is_null() {
            classified.planes.push((plane, &surface.location));
        }
    }
    classified
}
fn classify_solid_quadric<'a>(
    surface: &'a SolidSelectionSurface,
    classified: &mut CappedSolidSurfaces<'a>,
) -> bool {
    classified.cone = surface.surface.cast();
    if !classified.cone.is_null() {
        classified.quadric = Some(surface);
        return true;
    }
    classified.cylinder = surface.surface.cast();
    if !classified.cylinder.is_null() {
        classified.quadric = Some(surface);
        return true;
    }
    false
}
fn recognize_capped_cylinder(
    surfaces: &[SolidSelectionSurface],
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Select3DSensitiveCylinder>, Failure> {
    let classified = classify_capped_surfaces(surfaces);
    if classified.planes.len() != 2 {
        return Ok(None);
    }
    let Some(quadric) = classified.quadric else {
        return Ok(None);
    };
    if !classified.cone.is_null() {
        return recognize_capped_cone(&classified, quadric, owner);
    }
    recognize_capped_round_cylinder(&classified, quadric, owner)
}
fn capped_plane_height(classified: &CappedSolidSurfaces<'_>) -> f64 {
    let first = classified.planes[0]
        .0
        .location()
        .transformed(classified.planes[0].1.transformation());
    let second = classified.planes[1]
        .0
        .location()
        .transformed(classified.planes[1].1.transformation());
    first.distance(&second)
}
fn recognize_capped_cone(
    classified: &CappedSolidSurfaces<'_>,
    quadric: &SolidSelectionSurface,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Select3DSensitiveCylinder>, Failure> {
    let direction = classified.cone.position().direction();
    if classified.planes.iter().any(|(plane, _)| {
        !plane
            .position()
            .direction()
            .is_equal(&direction, precision::angular())
    }) {
        return Ok(None);
    }
    let cone = BRepAdaptorSurface::with_face(&quadric.face, true)?.cone()?;
    let radius = cone.ref_radius();
    let height = capped_plane_height(classified);
    let angle = cone.semi_angle();
    let triangle_height = if angle > 0. {
        radius / angle.tan()
    } else {
        radius / angle.abs().tan() - height
    };
    let top = if angle > 0. {
        radius * (triangle_height + height) / triangle_height
    } else {
        radius * triangle_height / (triangle_height + height)
    };
    Ok(Some(solid_cylinder_entity(
        owner,
        (radius, top),
        height,
        cone.position(),
    )))
}
fn recognize_capped_round_cylinder(
    classified: &CappedSolidSurfaces<'_>,
    quadric: &SolidSelectionSurface,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Select3DSensitiveCylinder>, Failure> {
    let direction = classified.cylinder.position().direction();
    if classified.planes.iter().any(|(plane, _)| {
        !plane
            .position()
            .direction()
            .is_parallel(direction, precision::angular())
    }) {
        return Ok(None);
    }
    let cylinder = BRepAdaptorSurface::with_face(&quadric.face, true)?.cylinder()?;
    let height = capped_plane_height(classified);
    let radius = cylinder.radius();
    let mut position = cylinder.position();
    if classified.planes[0].0.position().is_coplanar(
        classified.planes[1].0.position(),
        precision::angular(),
        precision::angular(),
    ) {
        // OCCT 3d097a0 deliberately compares the underlying planes before their locations.
        position.set_direction(position.direction().reversed())?;
    }
    Ok(Some(solid_cylinder_entity(
        owner,
        (radius, radius),
        height,
        position,
    )))
}
