//! `gp3d_lin`: every gp_Lin member, mirroring `gpc_lin_case`.
use super::line;
use crate::elementary_probe::{Fields, attempt, dir, end, flag, pnt, real, reals};
use crate::gp_transform_probe::read_transform;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Line3, Point3, Transform3, Vector3, Xyz};

/// The native case's shared inputs: three points, two raw directions, angle and scale.
struct LineInputs {
    o: Point3,
    p: Point3,
    q: Point3,
    n: Xyz,
    m: Xyz,
    angle: f64,
    s: f64,
    transform: Transform3,
}
pub fn gp3_lin_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (o, p, q, n, m) = (f.point(), f.point(), f.point(), f.xyz(), f.xyz());
    let (angle, s) = (f.real(), f.real());
    let transform = read_transform(&mut f);
    let inputs = LineInputs {
        o,
        p,
        q,
        n,
        m,
        angle,
        s,
        transform,
    };
    attempt(|| members(&inputs));
    end();
}
fn members(i: &LineInputs) -> Result<(), Failure> {
    let l = Line3::new(i.o, Direction3::from_xyz(i.n)?);
    let l2 = Line3::from(Axis1::new(i.q, Direction3::from_xyz(i.m)?));
    [Line3::default(), l, l2].into_iter().for_each(line);
    pnt(l.location());
    dir(l.direction());
    reals(&[l.distance(i.p), l.square_distance(i.p)]);
    attempt(|| {
        real(l.distance_line(l2)?);
        Ok(())
    });
    attempt(|| {
        real(l.square_distance_line(l2)?);
        Ok(())
    });
    flag(l.contains(i.p, i.s));
    real(l.angle(l2));
    attempt(|| {
        line(l.normal(i.p)?);
        Ok(())
    });
    line(l.reversed());
    let mut e = l;
    e.reverse();
    e.set_location(i.q);
    e.set_direction(Direction3::from_xyz(i.m)?);
    line(e);
    e.set_position(l.position());
    line(e);
    let axis = Axis1::new(i.q, Direction3::from_xyz(i.m)?);
    let plane = Axis2::new(i.q, Direction3::from_xyz(i.m)?)?;
    motions(l, i, axis, plane)
}
fn motions(l: Line3, i: &LineInputs, axis: Axis1, plane: Axis2) -> Result<(), Failure> {
    [
        l.mirrored_point(i.p),
        l.mirrored_axis(axis),
        l.mirrored_plane(plane),
    ]
    .into_iter()
    .for_each(line);
    let mut e = l;
    e.mirror_point(i.p);
    e.mirror_axis(axis);
    e.mirror_plane(plane);
    line(e);
    line(l.rotated(axis, i.angle)?);
    e = l;
    e.rotate(axis, i.angle)?;
    line(e);
    line(l.scaled(i.p, i.s));
    e = l;
    e.scale(i.p, i.s);
    line(e);
    line(l.transformed(i.transform));
    e = l;
    e.transform(i.transform);
    line(e);
    line(l.translated(Vector3::from(i.p.xyz())));
    e = l;
    e.translate(Vector3::from(i.q.xyz()));
    line(e);
    line(l.translated_between(i.p, i.q));
    e = l;
    e.translate_between(i.p, i.q);
    line(e);
    Ok(())
}
