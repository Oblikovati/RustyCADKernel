// Rust translation of BRepPrim_OneAxis.cxx (Shell, LateralFace, TopFace, BottomFace,
// StartFace, EndFace) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shell and the faces of `BRepPrim_OneAxis`, each built once.
use super::BRepPrimOneAxis;
use super::one_axis::{
    EAXIS, EBOTEND, EBOTSTART, EBOTTOM, EEND, ESTART, ETOP, ETOPEND, ETOPSTART, FBOTTOM, FEND,
    FLATERAL, FSTART, FTOP, are_heights_equal, axis_vector, is_height_inverted,
    one_axis_domain_error,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face, Shell};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Circle2, Direction2, Line2, Plane3, Point2, StandardDirection2,
};

/// `THE_PCURVE_OFFSET_FACTOR`: the multiplier of Precision::Confusion() that separates
/// the p-curves of the top and bottom side edges when their heights are equal.
const THE_PCURVE_OFFSET_FACTOR: f64 = 10.0;

/// A 2d line through `(x, y)` along +X.
fn line_x(x: f64, y: f64) -> Line2 {
    Line2::new(
        Point2::new(x, y),
        Direction2::from_standard(StandardDirection2::X),
    )
}

/// A 2d line through `(x, y)` along +Y.
fn line_y(x: f64, y: f64) -> Line2 {
    Line2::new(
        Point2::new(x, y),
        Direction2::from_standard(StandardDirection2::Y),
    )
}

/// The built edge in slot `slot`.
fn edge_at<T: BRepPrimOneAxis + ?Sized>(prim: &T, slot: usize) -> Edge {
    prim.one_axis_base().my_edges[slot].clone()
}

/// Stores `face` in slot `slot` and marks it built.
fn store_face<T: BRepPrimOneAxis + ?Sized>(prim: &mut T, slot: usize, face: &Face) -> Face {
    let base = prim.one_axis_base_mut();
    base.my_faces[slot] = face.clone();
    base.faces_built[slot] = true;
    face.clone()
}

/// The built face in slot `slot`, or `None` when it is not built yet.
fn built_face<T: BRepPrimOneAxis + ?Sized>(prim: &T, slot: usize) -> Option<Face> {
    let base = prim.one_axis_base();
    base.faces_built[slot].then(|| base.my_faces[slot].clone())
}

/// `Shell()`: the lateral face, then the top, bottom, start and end faces that exist.
pub(crate) fn shell<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Shell, Failure> {
    if prim.one_axis_base().shell_built {
        return Ok(prim.one_axis_base().my_shell.clone());
    }
    let builder = prim.one_axis_base().my_builder;
    let mut shell = Shell::default();
    builder.make_shell(&mut shell)?;
    builder.add_shell_face(&mut shell, &prim.lateral_face()?)?;
    if prim.has_top()? {
        builder.add_shell_face(&mut shell, &prim.top_face()?)?;
    }
    if prim.has_bottom()? {
        builder.add_shell_face(&mut shell, &prim.bottom_face()?)?;
    }
    if prim.has_sides() {
        builder.add_shell_face(&mut shell, &prim.start_face()?)?;
        builder.add_shell_face(&mut shell, &prim.end_face()?)?;
    }
    shell.set_closed(BRepTool::is_closed(&shell)?)?;
    builder.complete_shell(&shell)?;
    let base = prim.one_axis_base_mut();
    base.my_shell = shell.clone();
    base.shell_built = true;
    Ok(shell)
}

/// `LateralFace()`: the empty lateral face with its wires (the start and end wires for
/// an infinite range) and the p-curves of its edges.
pub(crate) fn lateral_face<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Face, Failure> {
    if let Some(face) = built_face(prim, FLATERAL) {
        return Ok(face);
    }
    let builder = prim.one_axis_base().my_builder;
    let mut face = prim.make_empty_lateral_face()?;
    if prim.vmax_infinite() && prim.vmin_infinite() {
        builder.add_face_wire(&mut face, &prim.lateral_start_wire()?)?;
        builder.add_face_wire(&mut face, &prim.lateral_end_wire()?)?;
    } else {
        builder.add_face_wire(&mut face, &prim.lateral_wire()?)?;
    }
    lateral_circle_pcurves(prim, &face)?;
    lateral_meridian_pcurves(prim, &face)?;
    builder.complete_face(&face)?;
    Ok(store_face(prim, FLATERAL, &face))
}

/// The p-curves of the top and bottom edges on the lateral face: one seam for a closed
/// meridian, else a line at each finite end, and the range 0..Angle of a closed edge.
fn lateral_circle_pcurves<T: BRepPrimOneAxis + ?Sized>(
    prim: &mut T,
    face: &Face,
) -> Result<(), Failure> {
    let builder = prim.one_axis_base().my_builder;
    let (vmin, vmax, angle) = (prim.vmin(), prim.vmax(), prim.angle());
    if prim.meridian_closed()? {
        return builder.set_pcurve_lines(
            &edge_at(prim, ETOP),
            face,
            line_x(0.0, vmin),
            line_x(0.0, vmax),
        );
    }
    if !prim.vmax_infinite() {
        builder.set_pcurve_line(&edge_at(prim, ETOP), face, line_x(0.0, vmax))?;
        if !prim.has_sides() || prim.meridian_on_axis(vmax)? {
            let vertex = prim.top_end_vertex()?;
            builder.set_parameters(&edge_at(prim, ETOP), &vertex, 0.0, angle)?;
        }
    }
    if !prim.vmin_infinite() {
        builder.set_pcurve_line(&edge_at(prim, EBOTTOM), face, line_x(0.0, vmin))?;
        if !prim.has_sides() || prim.meridian_on_axis(vmin)? {
            let vertex = prim.bottom_end_vertex()?;
            builder.set_parameters(&edge_at(prim, EBOTTOM), &vertex, 0.0, angle)?;
        }
    }
    Ok(())
}

/// The p-curves of the start and end meridian edges on the lateral face: a seam at 0
/// and Angle without sides.
fn lateral_meridian_pcurves<T: BRepPrimOneAxis + ?Sized>(
    prim: &T,
    face: &Face,
) -> Result<(), Failure> {
    let builder = prim.one_axis_base().my_builder;
    let offset = prim.one_axis_base().my_meridian_offset;
    let angle = prim.angle();
    if prim.has_sides() {
        builder.set_pcurve_line(&edge_at(prim, ESTART), face, line_y(0.0, -offset))?;
        return builder.set_pcurve_line(&edge_at(prim, EEND), face, line_y(angle, -offset));
    }
    builder.set_pcurve_lines(
        &edge_at(prim, ESTART),
        face,
        line_y(angle, -offset),
        line_y(0.0, -offset),
    )
}

/// `TopFace()`: the plane at the top height with the top wire, its circle and, with
/// sides, its two lines as p-curves.
pub(crate) fn top_face<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Face, Failure> {
    if let Some(face) = built_face(prim, FTOP) {
        return Ok(face);
    }
    if !prim.has_top()? {
        return Err(one_axis_domain_error("TopFace", "No top face"));
    }
    let builder = prim.one_axis_base().my_builder;
    let vmax = prim.vmax();
    let mut face = Face::default();
    builder.make_face(&mut face, cap_plane(prim, vmax)?)?;
    builder.add_face_wire(&mut face, &prim.top_wire()?)?;
    cap_pcurves(prim, &face, vmax, [ETOP, ETOPSTART, ETOPEND])?;
    builder.complete_face(&face)?;
    Ok(store_face(prim, FTOP, &face))
}

/// `BottomFace()`: the reversed plane at the bottom height with the bottom wire and
/// the p-curves of its edges.
pub(crate) fn bottom_face<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Face, Failure> {
    if let Some(face) = built_face(prim, FBOTTOM) {
        return Ok(face);
    }
    if !prim.has_bottom()? {
        return Err(one_axis_domain_error("BottomFace", "No bottom face"));
    }
    let builder = prim.one_axis_base().my_builder;
    let vmin = prim.vmin();
    let mut face = Face::default();
    builder.make_face(&mut face, cap_plane(prim, vmin)?)?;
    builder.reverse_face(&mut face);
    builder.add_face_wire(&mut face, &prim.bottom_wire()?)?;
    cap_pcurves(prim, &face, vmin, [EBOTTOM, EBOTSTART, EBOTEND])?;
    builder.complete_face(&face)?;
    Ok(store_face(prim, FBOTTOM, &face))
}

/// The plane of a cap: the axes translated to the meridian height at `v`.
fn cap_plane<T: BRepPrimOneAxis + ?Sized>(prim: &T, v: f64) -> Result<Plane3, Failure> {
    let z = prim.meridian_value(v)?.y();
    let axes = prim.axes();
    Ok(Plane3::from_position(
        axes.translated(axis_vector(axes, z)).axis3(),
    ))
}

/// The p-curves of a cap: the circle of the meridian radius at `v` and, with sides,
/// the lines of the start and end edges; `slots` is `[circle, start, end]`.
fn cap_pcurves<T: BRepPrimOneAxis + ?Sized>(
    prim: &T,
    face: &Face,
    v: f64,
    slots: [usize; 3],
) -> Result<(), Failure> {
    let builder = prim.one_axis_base().my_builder;
    let x_axis = Axis2d::new(
        Point2::new(0.0, 0.0),
        Direction2::from_standard(StandardDirection2::X),
    );
    let circle = Circle2::from_x_axis(x_axis, prim.meridian_value(v)?.x(), true)?;
    builder.set_pcurve_circle(&edge_at(prim, slots[0]), face, circle)?;
    if prim.has_sides() {
        let angle = prim.angle();
        builder.set_pcurve_line(&edge_at(prim, slots[1]), face, line_x(0.0, 0.0))?;
        let end = Line2::new(
            Point2::new(0.0, 0.0),
            Direction2::new(angle.cos(), angle.sin())?,
        );
        builder.set_pcurve_line(&edge_at(prim, slots[2]), face, end)?;
    }
    Ok(())
}

/// The frame of the start face: the axes location, normal -Y, X direction X.
fn start_frame(axes: Axis2) -> Result<Axis2, Failure> {
    Axis2::with_x_direction(
        axes.location(),
        axes.y_direction().reversed(),
        axes.x_direction(),
    )
}

/// `StartFace()`: the plane at angle 0 with the start wires and the p-curves of its
/// edges; reversed when the heights are inverted, or equal on an open meridian.
pub(crate) fn start_face<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Face, Failure> {
    if let Some(face) = built_face(prim, FSTART) {
        return Ok(face);
    }
    if !prim.has_sides() {
        return Err(one_axis_domain_error("StartFace", "No side faces"));
    }
    let builder = prim.one_axis_base().my_builder;
    let mut face = Face::default();
    builder.make_face(
        &mut face,
        Plane3::from_position(start_frame(prim.axes())?.axis3()),
    )?;
    if is_height_inverted(prim)? || (are_heights_equal(prim)? && !prim.meridian_closed()?) {
        builder.reverse_face(&mut face);
    }
    if prim.vmax_infinite() && prim.vmin_infinite() {
        builder.add_face_wire(&mut face, &prim.axis_start_wire()?)?;
    }
    builder.add_face_wire(&mut face, &prim.start_wire()?)?;
    prim.set_meridian_pcurve(&edge_at(prim, ESTART), &face)?;
    side_pcurves(prim, &face, [ETOPSTART, EBOTSTART])?;
    builder.complete_face(&face)?;
    Ok(store_face(prim, FSTART, &face))
}

/// `EndFace()`: the start plane rotated by Angle with the end wires and the p-curves
/// of its edges; reversed unless the heights are inverted, or equal on an open meridian.
pub(crate) fn end_face<T: BRepPrimOneAxis + ?Sized>(prim: &mut T) -> Result<Face, Failure> {
    if let Some(face) = built_face(prim, FEND) {
        return Ok(face);
    }
    if !prim.has_sides() {
        return Err(one_axis_domain_error("EndFace", "No side faces"));
    }
    let builder = prim.one_axis_base().my_builder;
    let axes = start_frame(prim.axes())?.rotated(prim.axes().axis(), prim.angle())?;
    let mut face = Face::default();
    builder.make_face(&mut face, Plane3::from_position(axes.axis3()))?;
    if !is_height_inverted(prim)? && (!are_heights_equal(prim)? || prim.meridian_closed()?) {
        builder.reverse_face(&mut face);
    }
    if prim.vmax_infinite() && prim.vmin_infinite() {
        builder.add_face_wire(&mut face, &prim.axis_end_wire()?)?;
    }
    builder.add_face_wire(&mut face, &prim.end_wire()?)?;
    prim.set_meridian_pcurve(&edge_at(prim, EEND), &face)?;
    side_pcurves(prim, &face, [ETOPEND, EBOTEND])?;
    builder.complete_face(&face)?;
    Ok(store_face(prim, FEND, &face))
}

/// The p-curves of a side face: the axis edge (when built and not degenerated) and the
/// built top and bottom lines at their heights, moved apart when the heights are
/// equal; `slots` is `[top, bottom]`.
fn side_pcurves<T: BRepPrimOneAxis + ?Sized>(
    prim: &T,
    face: &Face,
    slots: [usize; 2],
) -> Result<(), Failure> {
    let builder = prim.one_axis_base().my_builder;
    let built = prim.one_axis_base().edges_built;
    if built[EAXIS] && !are_heights_equal(prim)? {
        builder.set_pcurve_line(&edge_at(prim, EAXIS), face, line_y(0.0, 0.0))?;
    }
    let offset = if are_heights_equal(prim)? {
        precision::confusion() * THE_PCURVE_OFFSET_FACTOR
    } else {
        0.0
    };
    if built[slots[0]] {
        let y = prim.meridian_value(prim.vmax())?.y() + offset;
        builder.set_pcurve_line(&edge_at(prim, slots[0]), face, line_x(0.0, y))?;
    }
    if built[slots[1]] {
        let y = prim.meridian_value(prim.vmin())?.y() - offset;
        builder.set_pcurve_line(&edge_at(prim, slots[1]), face, line_x(0.0, y))?;
    }
    Ok(())
}
