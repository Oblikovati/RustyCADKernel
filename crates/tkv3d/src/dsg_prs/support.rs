//! The drawing steps every DsgPrs builder repeats: the dimension aspect of a drawer, the
//! current/new group with the dimension line aspect, segment, polyline and point arrays,
//! texts and arrows.
use crate::prs3d::{Prs3dArrow, Prs3dDimensionAspect, Prs3dDrawer, Prs3dText};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkgeombase::gce;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::aspect::AspectTypeOfMarker;
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPoints, Graphic3dArrayOfPolylines, Graphic3dArrayOfSegments,
    Graphic3dAspectMarker3d, Graphic3dAspectsMembers as _, Graphic3dGroup,
    Graphic3dGroupMembers as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};

/// `Prs3d_Presentation` (a `Graphic3d_Structure`) as the builders receive it.
pub type DsgPrsPresentation = Handle<dyn Graphic3dStructureSource>;

/// `aDrawer->DimensionAspect()`; a null drawer is a native null dereference.
pub(super) fn dimension_aspect(
    drawer: &Handle<Prs3dDrawer>,
) -> Result<Handle<Prs3dDimensionAspect>, Failure> {
    let Some(drawer) = drawer.get() else {
        return Err(null_object(
            "DsgPrs: the drawer is null; expected a Prs3d_Drawer",
        ));
    };
    Ok(drawer.dimension_aspect())
}

pub(super) fn null_object(message: &str) -> Failure {
    Failure::with_type("Standard_NullObject", message, "")
}

/// The presentation as its structure source; a null handle is a native null dereference.
pub(super) fn structure(
    prs: &DsgPrsPresentation,
) -> Result<&dyn Graphic3dStructureSource, Failure> {
    prs.get().ok_or_else(|| {
        null_object("DsgPrs: the presentation is null; expected a Prs3d_Presentation")
    })
}

/// `aPresentation->CurrentGroup()`.
pub(super) fn current_group(
    prs: &DsgPrsPresentation,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    structure(prs)?.current_group()
}

/// `aPresentation->NewGroup()`.
pub(super) fn new_group(prs: &DsgPrsPresentation) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    structure(prs)?.new_group()
}

/// `group->SetPrimitivesAspect(LA->LineAspect()->Aspect())`.
pub(super) fn set_line_aspect(group: &Handle<dyn Graphic3dGroup>, aspect: &Prs3dDimensionAspect) {
    if let (Some(group), Some(line)) = (group.get(), aspect.line_aspect().get()) {
        group.set_primitives_aspect(&line.aspect().cast());
    }
}

/// `CurrentGroup()->SetPrimitivesAspect(LA->LineAspect()->Aspect())`: the usual first step.
pub(super) fn current_line_group(
    prs: &DsgPrsPresentation,
    aspect: &Prs3dDimensionAspect,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    let group = current_group(prs)?;
    set_line_aspect(&group, aspect);
    Ok(group)
}

/// `NewGroup()` then the dimension line aspect.
pub(super) fn new_line_group(
    prs: &DsgPrsPresentation,
    aspect: &Prs3dDimensionAspect,
) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
    new_group(prs)?;
    current_line_group(prs, aspect)
}

/// A `Graphic3d_ArrayOfSegments` of the given vertex capacity filled with `points`.
pub(super) fn segments(
    capacity: i32,
    points: &[Point3],
) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    let array = Graphic3dArrayOfSegments::new(capacity, 0, 0)?;
    for point in points {
        array.add_vertex_pnt(point)?;
    }
    Ok(Handle::new(array))
}

/// A one-bound `Graphic3d_ArrayOfPolylines(points.len())` of the points.
pub(super) fn polyline(points: &[Point3]) -> Result<Handle<Graphic3dArrayOfPolylines>, Failure> {
    let array = Graphic3dArrayOfPolylines::new(points.len() as i32, 0, 0, 0)?;
    for point in points {
        array.add_vertex_pnt(point)?;
    }
    Ok(Handle::new(array))
}

/// `AddPrimitiveArray` of an array handle on the current group.
pub(super) fn add_array<T: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    group: &Handle<dyn Graphic3dGroup>,
    array: &Handle<T>,
) {
    if let Some(group) = group.get() {
        group.add_primitive_array(&array.clone().cast(), true);
    }
}

/// `Prs3d_Text::Draw(group, LA->TextAspect(), text, position)`.
pub(super) fn draw_text(
    group: &Handle<dyn Graphic3dGroup>,
    aspect: &Prs3dDimensionAspect,
    text: &ExtendedString,
    position: &Point3,
) {
    Prs3dText::draw(group, &aspect.text_aspect(), text, position);
}

/// `Prs3d_Arrow::Draw(group, point, direction, LA->ArrowAspect()->Angle(), Length())`.
pub(super) fn draw_arrow(
    group: &Handle<dyn Graphic3dGroup>,
    aspect: &Prs3dDimensionAspect,
    point: &Point3,
    direction: &Direction3,
) -> Result<(), Failure> {
    let (angle, length) = arrow_angle_length(aspect);
    Prs3dArrow::draw(group, point, direction, angle, length)
}

/// The angle and length of the dimension arrow aspect.
pub(super) fn arrow_angle_length(aspect: &Prs3dDimensionAspect) -> (f64, f64) {
    aspect
        .arrow_aspect()
        .get()
        .map_or((0.0, 0.0), |arrow| (arrow.angle(), arrow.length()))
}

/// `LA->ArrowAspect()->Length()`.
pub(super) fn arrow_length(aspect: &Prs3dDimensionAspect) -> f64 {
    arrow_angle_length(aspect).1
}

/// A ring marker (`Aspect_TOM_O`, scale 1) of the dimension line color.
pub(super) fn ring_marker(
    aspect: &Prs3dDimensionAspect,
) -> Result<Handle<Graphic3dAspectMarker3d>, Failure> {
    let color = aspect
        .line_aspect()
        .get()
        .map(|line| line.aspect().interior_color())
        .unwrap_or_default();
    Ok(Handle::new(Graphic3dAspectMarker3d::new(
        AspectTypeOfMarker::O,
        color,
        1.0,
    )?))
}

/// A `Graphic3d_ArrayOfPoints` of the points.
pub(super) fn points(points: &[Point3]) -> Result<Handle<Graphic3dArrayOfPoints>, Failure> {
    let array = Graphic3dArrayOfPoints::new(points.len() as i32, 0)?;
    for point in points {
        array.add_vertex_f64(point.x(), point.y(), point.z())?;
    }
    Ok(Handle::new(array))
}

/// A new group with the line aspect, then a ring marker aspect and one point
/// (the "round" end symbol many presentations add).
pub(super) fn add_ring_point(
    prs: &DsgPrsPresentation,
    aspect: &Prs3dDimensionAspect,
    point: &Point3,
) -> Result<(), Failure> {
    let group = new_line_group(prs, aspect)?;
    let marker = ring_marker(aspect)?;
    if let Some(value) = group.get() {
        value.set_primitives_aspect(&marker.cast());
    }
    add_array(&group, &points(&[*point])?);
    Ok(())
}

/// `ElCLib::Value(ElCLib::Parameter(line, point), line)`: the projection on a line.
pub(super) fn project_on_line(line: &Line3, point: &Point3) -> Result<Point3, Failure> {
    Ok(elclib::value(elclib::parameter(line, *point)?, line))
}

/// `gp_Dir(gp_Vec(from, to))`.
pub(super) fn direction_between(from: &Point3, to: &Point3) -> Result<Direction3, Failure> {
    Direction3::from_vector(Vector3::between(*from, *to))
}

/// `gce_MakeDir(P1, P2)` through its `gp_Dir` conversion (StdFail_NotDone when the points
/// are confused within gp::Resolution()).
pub(super) fn make_dir(from: &Point3, to: &Point3) -> Result<Direction3, Failure> {
    gce::MakeDir::from_points(*from, *to)?.value()
}

/// `gce_MakeLin(P1, P2)` through its `gp_Lin` conversion.
pub(super) fn make_lin(from: &Point3, to: &Point3) -> Result<Line3, Failure> {
    gce::MakeLin::from_points(*from, *to)?.value()
}

/// `Precision::Confusion()`.
pub(super) fn confusion() -> f64 {
    precision::confusion()
}

/// `Precision::Angular()`.
pub(super) fn angular() -> f64 {
    precision::angular()
}
