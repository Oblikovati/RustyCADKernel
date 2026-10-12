//! Native analytic recognition inspects real seam/degenerated edges and preserves the face surface placement and scale.
use crate::{select_mgr::SelectMgrEntityOwnerSource, select3d::*};
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface},
    brep_tools, topexp,
    topods::{self, Face, ShapeIterator},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkg3d::{
    adaptor3d::{Adaptor3dCurve, Adaptor3dSurface},
    geom::*,
};
use rustycad_tkmath::{
    elslib,
    geom_abs::CurveType,
    gp::{Axis3, Transform3, xoy},
    toploc::Location,
};
pub(super) fn brep_face_analytic(
    face: &Face,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    interior: bool,
) -> Result<Option<Handle<dyn Select3DSensitiveEntity>>, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let sphere = surface.cast::<GeomSphericalSurface>();
    if !sphere.is_null() {
        return brep_full_sphere(face, owner);
    }
    let cone = surface.cast::<GeomConicalSurface>();
    if !cone.is_null() {
        return brep_full_cone(face, &cone, &location, owner);
    }
    let cylinder = surface.cast::<GeomCylindricalSurface>();
    if !cylinder.is_null() {
        return brep_full_cylinder(face, &cylinder, &location, owner);
    }
    let plane = surface.cast::<GeomPlane>();
    if !plane.is_null() {
        return brep_planar_circle(face, owner, interior);
    }
    Ok(None)
}
fn brep_full_sphere(
    face: &Face,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Handle<dyn Select3DSensitiveEntity>>, Failure> {
    let full = face.nb_children() == 0
        || (face.nb_children() == 1 && sphere_wire_counts(face)? == Some((2, 2)));
    if !full {
        return Ok(None);
    }
    let sphere = BRepAdaptorSurface::with_face(face, true)?.sphere()?;
    let sensitive =
        Select3DSensitiveSphere::new(owner, sphere.position().axis().location(), sphere.radius());
    Ok(Some(Handle::new(sensitive).cast()))
}
fn sphere_wire_counts(face: &Face) -> Result<Option<(usize, usize)>, Failure> {
    let wire = topods::wire(ShapeIterator::new(face, true, true).value()?)?;
    if wire.nb_children() != 4 {
        return Ok(None);
    }
    let mut seams = 0;
    let mut degenerated = 0;
    for shape in ShapeIterator::new(&wire, true, true) {
        let edge = topods::edge(&shape)?;
        seams += usize::from(BRepTool::is_closed_on_face(&edge, face)?);
        degenerated += usize::from(BRepTool::degenerated(&edge)?);
    }
    Ok(Some((seams, degenerated)))
}
fn is_full_cone_cylinder(face: &Face) -> Result<bool, Failure> {
    if face.nb_children() != 1 {
        return Ok(false);
    }
    let iterator = ShapeIterator::new(face, true, true);
    let wire = iterator.value()?;
    if wire.shape_type()? != ShapeEnum::Wire || wire.nb_children() != 4 {
        return Ok(false);
    }
    let mut seams = 0;
    let mut circles = 0;
    for shape in ShapeIterator::new(&wire, true, true) {
        let edge = topods::edge(&shape)?;
        seams += usize::from(BRepTool::is_closed_on_face(&edge, face)?);
        circles +=
            usize::from(BRepAdaptorCurve::with_edge(&edge)?.get_type()? == CurveType::Circle);
    }
    Ok(seams == 2 && circles == 2)
}
fn brep_full_cone(
    face: &Face,
    cone: &GeomConicalSurface,
    location: &Location,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Handle<dyn Select3DSensitiveEntity>>, Failure> {
    if !is_full_cone_cylinder(face)? {
        return Ok(None);
    }
    let (_, _, first, last) = brep_tools::uv_bounds(face)?;
    let bottom = elslib::cone_v_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), first);
    let top = elslib::cone_v_iso(cone.position(), cone.ref_radius(), cone.semi_angle(), last);
    let mut position = cone.position();
    position.set_location(bottom.location());
    let radii = (bottom.radius(), top.radius());
    let height = bottom.location().distance(&top.location());
    Ok(Some(position_brep_cylinder(
        owner, radii, height, position, location,
    )?))
}
fn brep_full_cylinder(
    face: &Face,
    cylinder: &GeomCylindricalSurface,
    location: &Location,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<Option<Handle<dyn Select3DSensitiveEntity>>, Failure> {
    if !is_full_cone_cylinder(face)? {
        return Ok(None);
    }
    let (_, _, first, last) = brep_tools::uv_bounds(face)?;
    let mut position = cylinder.position();
    position.set_location((cylinder.location().xyz() + position.direction().xyz() * first).into());
    let radii = (cylinder.radius(), cylinder.radius());
    Ok(Some(position_brep_cylinder(
        owner,
        radii,
        last - first,
        position,
        location,
    )?))
}
fn position_brep_cylinder(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    radii: (f64, f64),
    height: f64,
    position: Axis3,
    location: &Location,
) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
    let mut local = Transform3::default();
    local.set_transformation_between(position, Axis3::from_axis2(xoy()));
    let mut transform = location.transformation().multiplied(local);
    let scale = transform.scale_factor();
    transform.set_scale_factor(1.)?;
    let cylinder = Select3DSensitiveCylinder::new(
        owner,
        radii.0 * scale,
        radii.1 * scale,
        height * scale,
        transform,
        true,
    );
    Ok(Handle::new(cylinder).cast())
}
fn brep_planar_circle(
    face: &Face,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
    interior: bool,
) -> Result<Option<Handle<dyn Select3DSensitiveEntity>>, Failure> {
    let mut edges = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(face, ShapeEnum::Edge, &mut edges)?;
    if edges.extent() != 1 {
        return Ok(None);
    }
    let edge = topods::edge(edges.find_key(1)?)?;
    let adaptor = BRepAdaptorCurve::with_edge(&edge)?;
    if adaptor.get_type()? != CurveType::Circle || !BRepTool::is_closed(&edge)? {
        return Ok(None);
    }
    Ok(Some(
        Handle::new(Select3DSensitiveCircle::new(
            owner,
            &adaptor.circle()?,
            interior,
        ))
        .cast(),
    ))
}
