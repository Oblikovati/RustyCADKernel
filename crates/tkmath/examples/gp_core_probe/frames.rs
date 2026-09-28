//! `gp3d_frame`: gp_Ax2 and gp_Ax3 constructors, accessors, edits and motions, mirroring
//! `gpc_frame_case`.
use super::json;
use crate::elementary_probe::{Fields, attempt, ax2, ax3, dir, end, flag, pnt};
use crate::gp_planar_probe::show;
use crate::gp_transform_probe::read_transform;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Transform3, Vector3, Xyz};

/// gp_Ax3::SetDirection / SetXDirection / SetYDirection.
type DirectionSetter = fn(&mut Axis3, Direction3) -> Result<(), Failure>;
/// The native case's inputs: two points, three raw directions, angle, scale and transform.
struct FrameInputs {
    o: Point3,
    p: Point3,
    n: Xyz,
    x: Xyz,
    m: Xyz,
    angle: f64,
    s: f64,
    transform: Transform3,
}
pub fn gp3_frame_case(values: &[f64]) {
    let mut f = Fields::new(values);
    let (o, p, n, x, m) = (f.point(), f.point(), f.xyz(), f.xyz(), f.xyz());
    let (angle, s) = (f.real(), f.real());
    let transform = read_transform(&mut f);
    let i = FrameInputs {
        o,
        p,
        n,
        x,
        m,
        angle,
        s,
        transform,
    };
    ax2(Axis2::default());
    ax3(Axis3::default());
    attempt(|| axis2_members(&i));
    attempt(|| axis3_members(&i));
    end();
}
fn unit(xyz: Xyz) -> Result<Direction3, Failure> {
    Direction3::from_xyz(xyz)
}
fn axis2_members(i: &FrameInputs) -> Result<(), Failure> {
    attempt(|| {
        ax2(Axis2::with_x_direction(i.o, unit(i.n)?, unit(i.x)?)?);
        Ok(())
    });
    let a = Axis2::new(i.o, unit(i.n)?)?;
    ax2(a);
    pnt(a.location());
    [a.direction(), a.x_direction(), a.y_direction()]
        .into_iter()
        .for_each(dir);
    let axis = Axis1::new(i.p, unit(i.m)?);
    ax2(a.mirrored_point(i.p));
    let mut e = a;
    e.mirror_point(i.p);
    ax2(e);
    show(a.rotated(axis, i.angle), ax2);
    e = a;
    attempt(|| e.rotate(axis, i.angle));
    ax2(e);
    ax2(a.scaled(i.p, i.s));
    e = a;
    e.scale(i.p, i.s);
    ax2(e);
    show(a.transformed(i.transform), ax2);
    e = a;
    attempt(|| e.transform(i.transform));
    ax2(e);
    ax2(a.translated(Vector3::from(i.m)));
    e = a;
    e.translate(Vector3::from(i.m));
    ax2(e);
    json(
        a,
        |a, s| a.dump_json(s),
        |a, t, c| a.init_from_json(t, c),
        ax2,
    );
    Ok(())
}
fn axis3_edits(a: Axis3, p: Point3, m: Xyz) -> Result<(), Failure> {
    let mut e = a;
    e.x_reverse();
    ax3(e);
    show(e.direct(), flag);
    e.y_reverse();
    e.z_reverse();
    ax3(e);
    show(e.direct(), flag);
    e = a;
    attempt(|| e.set_axis(Axis1::new(p, unit(m)?)));
    ax3(e);
    let setters: [DirectionSetter; 3] = [
        Axis3::set_direction,
        Axis3::set_x_direction,
        Axis3::set_y_direction,
    ];
    for set in setters {
        e = a;
        attempt(|| set(&mut e, unit(m)?));
        ax3(e);
    }
    e = a;
    e.set_location(p);
    ax3(e);
    Ok(())
}
fn axis3_members(i: &FrameInputs) -> Result<(), Failure> {
    attempt(|| {
        ax3(Axis3::new(i.o, unit(i.n)?, unit(i.x)?)?);
        Ok(())
    });
    let a = Axis3::from_normal(i.o, unit(i.n)?)?;
    ax3(a);
    ax3(Axis3::from_axis2(Axis2::new(i.o, unit(i.n)?)?));
    pnt(a.location());
    [a.direction(), a.x_direction(), a.y_direction()]
        .into_iter()
        .for_each(dir);
    flag(a.direct()?);
    axis3_edits(a, i.p, i.m)?;
    axis3_motions(a, i)?;
    show(a.ax2(), ax2);
    json(
        a,
        |a, s| a.dump_json(s),
        |a, t, c| a.init_from_json(t, c),
        ax3,
    );
    Ok(())
}
fn axis3_motions(a: Axis3, i: &FrameInputs) -> Result<(), Failure> {
    let axis = Axis1::new(i.p, unit(i.m)?);
    let plane = Axis2::new(i.p, unit(i.m)?)?;
    ax3(a.mirrored_plane(plane));
    let mut e = a;
    e.mirror_plane(plane);
    ax3(e);
    ax3(a.mirrored_point(i.p));
    ax3(a.mirrored_axis(axis));
    ax3(a.rotated(axis, i.angle)?);
    e = a;
    e.rotate(axis, i.angle)?;
    ax3(e);
    ax3(a.scaled(i.p, i.s));
    e = a;
    e.scale(i.p, i.s);
    ax3(e);
    ax3(a.transformed(i.transform));
    e = a;
    e.transform(i.transform);
    ax3(e);
    ax3(a.translated(Vector3::from(i.m)));
    e = a;
    e.translate(Vector3::from(i.m));
    ax3(e);
    ax3(a.translated_between(i.o, i.p));
    e = a;
    e.translate_between(i.o, i.p);
    ax3(e);
    Ok(())
}
