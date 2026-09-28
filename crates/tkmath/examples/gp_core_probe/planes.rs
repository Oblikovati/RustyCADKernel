//! `gp3d_pln`: every gp_Pln member, mirroring `gpc_pln_case`.
use super::plane;
use crate::elementary_probe::{Fields, attempt, ax1, end, flag, pnt, reals, text};
use crate::gp_transform_probe::read_transform;
use rustycad_tkernel::standard::{Failure, dump::DumpStream};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Direction3, Line3, Plane3, Point3, Transform3, Vector3, Xyz,
};

/// The native case's inputs after the equation coefficients.
struct PlaneInputs {
    o: Point3,
    p: Point3,
    q: Point3,
    n: Xyz,
    m: Xyz,
    angle: f64,
    s: f64,
    linear: f64,
    angular: f64,
    transform: Transform3,
}
pub fn gp3_pln_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (o, p, q, n, m) = (f.point(), f.point(), f.point(), f.xyz(), f.xyz());
    let equation: [f64; 4] = std::array::from_fn(|_| f.real());
    let (angle, s, linear, angular) = (f.real(), f.real(), f.real(), f.real());
    let transform = read_transform(&mut f);
    let i = PlaneInputs {
        o,
        p,
        q,
        n,
        m,
        angle,
        s,
        linear,
        angular,
        transform,
    };
    attempt(|| {
        let [a, b, c, d] = equation;
        plane(Plane3::from_equation(a, b, c, d)?);
        Ok(())
    });
    attempt(|| members(&i));
    end();
}
fn members(i: &PlaneInputs) -> Result<(), Failure> {
    let pl = Plane3::new(i.o, Direction3::from_xyz(i.n)?)?;
    let axis = Axis1::new(i.q, Direction3::from_xyz(i.m)?);
    let other = Plane3::new(i.q, Direction3::from_xyz(i.m)?)?;
    [Plane3::default(), pl, Plane3::from_position(pl.position())]
        .into_iter()
        .for_each(plane);
    metrics(pl, i, Line3::from(axis), other)?;
    edits(pl, i.p, axis, other)?;
    motions(pl, i, axis, Axis2::new(i.q, Direction3::from_xyz(i.m)?)?)
}
fn metrics(pl: Plane3, i: &PlaneInputs, line: Line3, other: Plane3) -> Result<(), Failure> {
    reals(&[
        pl.distance(i.p),
        pl.signed_distance(i.p),
        pl.square_distance(i.p),
    ]);
    reals(&[
        pl.distance_line(line),
        pl.signed_distance_line(line),
        pl.square_distance_line(line),
    ]);
    reals(&[
        pl.distance_plane(other),
        pl.signed_distance_plane(other),
        pl.square_distance_plane(other),
    ]);
    flag(pl.contains(i.p, i.linear));
    flag(pl.contains_line(line, i.linear, i.angular));
    [pl.axis(), pl.x_axis(), pl.y_axis()]
        .into_iter()
        .for_each(ax1);
    flag(pl.direct()?);
    pnt(pl.location());
    reals(&pl.coefficients()?);
    Ok(())
}
fn edits(pl: Plane3, p: Point3, axis: Axis1, other: Plane3) -> Result<(), Failure> {
    let mut e = pl;
    e.u_reverse();
    flag(e.direct()?);
    reals(&e.coefficients()?);
    e.v_reverse();
    plane(e);
    e = pl;
    attempt(|| e.set_axis(axis));
    plane(e);
    e.set_location(p);
    plane(e);
    e.set_position(other.position());
    plane(e);
    Ok(())
}
fn motions(pl: Plane3, i: &PlaneInputs, axis: Axis1, frame: Axis2) -> Result<(), Failure> {
    [
        pl.mirrored_point(i.p),
        pl.mirrored_axis(axis),
        pl.mirrored_plane(frame),
    ]
    .into_iter()
    .for_each(plane);
    let mut e = pl;
    e.mirror_point(i.p);
    e.mirror_axis(axis);
    e.mirror_plane(frame);
    plane(e);
    plane(pl.rotated(axis, i.angle)?);
    e = pl;
    e.rotate(axis, i.angle)?;
    plane(e);
    plane(pl.scaled(i.p, i.s));
    e = pl;
    e.scale(i.p, i.s);
    plane(e);
    plane(pl.transformed(i.transform));
    e = pl;
    e.transform(i.transform);
    plane(e);
    plane(pl.translated(Vector3::from(i.q.xyz())));
    e = pl;
    e.translate(Vector3::from(i.p.xyz()));
    plane(e);
    plane(pl.translated_between(i.p, i.q));
    e = pl;
    e.translate_between(i.p, i.q);
    plane(e);
    let (mut full, mut none) = (DumpStream::default(), DumpStream::default());
    pl.dump_json(&mut full, -1).unwrap();
    pl.dump_json(&mut none, 0).unwrap();
    text(&format!("{} | {} |", full.text(), none.text()));
    Ok(())
}
