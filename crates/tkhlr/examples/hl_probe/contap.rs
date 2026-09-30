//! Contap record mutations; native twin: tkhlr_probe_contap.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dHVertexBase;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_start::{TheArc, TheVertex};
use rustycad_tkgeomalgo::int_surf::{
    IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTransition, IntSurfTypeTrans,
};
use rustycad_tkhlr::contap::{ContapLine, ContapPoint};
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point2, Point3};

fn transition(value: i32) -> IntSurfTypeTrans {
    [
        IntSurfTypeTrans::In,
        IntSurfTypeTrans::Out,
        IntSurfTypeTrans::Touch,
        IntSurfTypeTrans::Undecided,
    ][value as usize]
}
pub(super) fn point_snapshot(point: &ContapPoint) {
    pnt(point.value());
    let (u, v) = point.parameters();
    real(u);
    real(v);
    real(point.parameter_on_line());
    flag(point.is_multiple());
    flag(point.is_internal());
    flag(point.is_on_arc());
    flag(point.is_vertex());
    point_arc_snapshot(point);
    point_transition_snapshot(point);
}
fn point_arc_snapshot(point: &ContapPoint) {
    attempt(|| {
        flag(point.arc()?.is_null());
        Ok(())
    });
    attempt(|| {
        real(point.parameter_on_arc()?);
        Ok(())
    });
}
fn point_transition_snapshot(point: &ContapPoint) {
    attempt(|| {
        int(point.transition_on_line()?.transition_type() as i32);
        Ok(())
    });
    attempt(|| {
        int(point.transition_on_arc()?.transition_type() as i32);
        Ok(())
    });
    attempt(|| {
        flag(point.vertex()?.is_null());
        Ok(())
    });
}
fn set_point_flags(point: &mut ContapPoint, r: &mut Reader) {
    let mask = r.integer();
    let null = r.integer() != 0;
    let t1 = IntSurfTransition::with_type(true, transition(r.integer()));
    let t2 = IntSurfTransition::with_type(false, transition(r.integer()));
    let arc = probe_arc(null);
    let vertex = probe_vertex(null);
    if mask & 1 != 0 {
        point.set_arc(&arc, r.real(), t1, t2);
    } else {
        r.real();
    }
    if mask & 2 != 0 {
        point.set_vertex(&vertex);
    }
    set_point_kind(point, mask);
}
fn set_point_kind(point: &mut ContapPoint, mask: i32) {
    if mask & 4 != 0 {
        point.set_multiple();
    }
    if mask & 8 != 0 {
        point.set_internal();
    }
}
/// Constructors, all flags, transition copies, and SetValue reset with preserved line parameter.
pub fn point_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let position = r.pnt();
    let (u, v) = (r.real(), r.real());
    let mut point = if mode == 0 {
        ContapPoint::new()
    } else {
        ContapPoint::with_value(position, u, v)
    };
    point_snapshot(&point);
    point.set_parameter(r.real());
    set_point_flags(&mut point, &mut r);
    point_snapshot(&point);
    let copy = point.clone();
    point.set_value(position, u, v);
    point_snapshot(&point);
    point_snapshot(&copy);
    end();
}
pub(super) fn line_geometry(line: &ContapLine) {
    int(line.type_contour() as i32);
    int(line.transition_on_s() as i32);
    attempt(|| {
        let l = line.line()?;
        pnt(l.location());
        dir(l.direction());
        Ok(())
    });
    line_circle_geometry(line);
    line_other_geometry(line);
}
fn line_circle_geometry(line: &ContapLine) {
    attempt(|| {
        let c = line.circle()?;
        pnt(c.location());
        dir(c.position().direction());
        dir(c.position().x_direction());
        real(c.radius());
        Ok(())
    });
}
fn line_other_geometry(line: &ContapLine) {
    attempt(|| {
        flag(line.arc()?.is_null());
        Ok(())
    });
    attempt(|| {
        int(line.nb_pnts()?);
        Ok(())
    });
    attempt(|| {
        pnt(line.point(1)?.value());
        Ok(())
    });
}
fn line_vertices(line: &ContapLine) {
    int(line.nb_vertex());
    for i in 1..=line.nb_vertex() {
        let v = line.vertex(i).unwrap();
        real(v.parameter_on_line());
        pnt(v.value());
        flag(v.is_multiple());
    }
    for i in [-1, 0, line.nb_vertex() + 1] {
        attempt(|| {
            line.vertex(i)?;
            Ok(())
        });
    }
}
fn configure_line(line: &mut ContapLine, mode: i32) {
    match mode {
        0 => line.set_line(Line3::new(Point3::new(1.0, 2.0, 3.0), Direction3::X)),
        1 => line.set_circle(Circle3::new(Axis2::default(), 5.0).unwrap()),
        2 => line.set_arc(&Handle::new(Geom2dAdaptorCurve::default()).cast()),
        _ => line.set_line_on_2s(Handle::new(IntSurfLineOn2S::new(None))),
    }
}
/// Geometry dispatch, ordered vertices, shared edits, reset detachment and shared walking clear.
pub fn line_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut line = ContapLine::new();
    configure_line(&mut line, r.integer());
    line.set_transition_on_s(transition(r.integer()));
    if line.type_contour() as i32 == 2 {
        line.add_point(&IntSurfPntOn2S::new()).unwrap();
    }
    populate_vertices(&line, &mut r);
    line_geometry(&line);
    line_vertices(&line);
    line_sharing_case(&mut line);
    end();
}
fn populate_vertices(line: &ContapLine, r: &mut Reader) {
    for i in 0..r.integer() {
        let mut vertex = ContapPoint::with_value(Point3::new(f64::from(i), 0.0, 0.0), 0.0, 0.0);
        vertex.set_parameter(r.real());
        line.add_vertex(&vertex);
    }
}
fn line_sharing_case(line: &mut ContapLine) {
    let mut copy = line.clone();
    if copy.nb_vertex() > 0 {
        copy.change_vertex(1, ContapPoint::set_multiple).unwrap();
    }
    line_vertices(line);
    copy.reset_seq_of_vertex();
    int(copy.nb_vertex());
    line_vertices(line);
    let shared = line.clone();
    line.clear();
    int(line.nb_vertex());
    line_vertices(&shared);
    if !shared.line_on_2s().is_null() {
        int(shared.line_on_2s().nb_points());
    }
}

fn probe_arc(null: bool) -> TheArc {
    if null {
        return Handle::default();
    }
    Handle::new(Geom2dAdaptorCurve::default()).cast()
}
fn probe_vertex(null: bool) -> TheVertex {
    if null {
        return Handle::default();
    }
    Handle::new(Adaptor3dHVertexBase::new(
        Point2::new(1.0, 2.0),
        Orientation::Forward,
        1e-7,
    ))
    .cast()
}
