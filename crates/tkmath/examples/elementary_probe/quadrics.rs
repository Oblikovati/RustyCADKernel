//! gp_Sphere/Cylinder/Cone/Torus (gpquadric) and gp_Ax1/Ax2/Ax3 (gpaxis) member requests.
use super::conics3::{Motion3, build_motion3, motion3};
use super::{Fields, attempt, ax1, ax2, ax3, direction, end, flag, frame3, pnt, real, reals, text};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, dump::DumpStream};
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Cone3, Cylinder3, Point3, Sphere3, Torus3, Transform3,
};

/// The native `el_placed_surface` template.
macro_rules! placed_surface {
    ($s:expr, $k:expr, $p:expr) => {{
        let (s, k, p) = ($s, $k, $p);
        ax3(s.position());
        attempt(|| Ok(flag(s.direct()?)));
        ax1(s.x_axis());
        ax1(s.y_axis());
        ax3(s.mirrored_point(k.pivot).position());
        ax3(s.mirrored_axis(k.axis).position());
        ax3(s.mirrored_plane(k.plane).position());
        attempt(|| Ok(ax3(s.rotated(k.axis, k.angle)?.position())));
        ax3(s.translated(k.shift).position());
        ax3(s.translated_between(k.pivot, p).position());
        ax3(s.scaled(k.pivot, k.scale).position());
        ax3(s.transformed(k.transform).position());
        let mut u = s;
        u.u_reverse();
        ax3(u.position());
        let mut v = s;
        v.v_reverse();
        ax3(v.position());
        let mut l = s;
        l.set_location(p);
        ax3(l.position());
        reals(&s.coefficients());
    }};
}
/// The native `el_axis_surface` template.
macro_rules! axis_surface {
    ($s:expr, $k:expr) => {{
        let (s, k) = ($s, $k);
        ax1(s.axis());
        let mut a = s;
        attempt(|| {
            a.set_axis(k.axis)?;
            Ok(ax3(a.position()))
        });
    }};
}

fn sphere(a: Axis3, r: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let s = Sphere3::new(a, r)?;
    reals(&[s.area(), s.volume()]);
    placed_surface!(s, k, p);
    real(s.scaled(k.pivot, k.scale).radius());
    real(s.transformed(k.transform).radius());
    Ok(())
}
fn cylinder(a: Axis3, r: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let s = Cylinder3::new(a, r)?;
    placed_surface!(s, k, p);
    axis_surface!(s, k);
    real(s.scaled(k.pivot, k.scale).radius());
    real(s.transformed(k.transform).radius());
    Ok(())
}
fn cone(a: Axis3, angle: f64, r: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let s = Cone3::new(a, angle, r)?;
    pnt(s.apex());
    placed_surface!(s, k, p);
    axis_surface!(s, k);
    let mut v = s;
    v.v_reverse();
    real(v.semi_angle());
    real(s.scaled(k.pivot, k.scale).ref_radius());
    real(s.transformed(k.transform).ref_radius());
    let mut w = s;
    attempt(|| {
        w.set_semi_angle(2.0 * angle)?;
        Ok(real(w.semi_angle()))
    });
    Ok(())
}
fn torus_radii(t: Torus3) {
    reals(&[t.major_radius(), t.minor_radius()]);
}
fn torus(a: Axis3, big: f64, small: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let s = Torus3::new(a, big, small)?;
    reals(&[s.area(), s.volume()]);
    axis_surface!(s, k);
    ax3(s.mirrored_axis(k.axis).position());
    ax3(s.translated_between(k.pivot, p).position());
    attempt(|| Ok(flag(s.direct()?)));
    let mut values = Array1::with_bounds(3, 37).unwrap();
    s.coefficients(&mut values)?;
    reals(values.as_slice());
    let mut short = Array1::with_bounds(1, 34).unwrap();
    attempt(|| s.coefficients(&mut short));
    torus_radii(s.scaled(k.pivot, k.scale));
    let mut b = s;
    b.scale(k.pivot, k.scale);
    torus_radii(b);
    torus_radii(s.transformed(k.transform));
    let mut d = s;
    d.transform(k.transform);
    torus_radii(d);
    let mut e = s;
    attempt(|| {
        e.set_major_radius(small)?;
        Ok(real(e.major_radius()))
    });
    attempt(|| {
        e.set_minor_radius(big)?;
        Ok(real(e.minor_radius()))
    });
    Ok(())
}
pub fn gpquadric_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, normal, x) = frame3(&mut fields);
    let indirect = fields.int() != 0;
    let (r, angle, big, small) = (fields.real(), fields.real(), fields.real(), fields.real());
    let motion = motion3(&mut fields);
    let p = fields.point();
    attempt(|| {
        let mut a = Axis3::new(origin, direction(normal)?, direction(x)?)?;
        if indirect {
            a.y_reverse();
        }
        let k = build_motion3(motion)?;
        attempt(|| sphere(a, r, &k, p));
        attempt(|| cylinder(a, r, &k, p));
        attempt(|| cone(a, angle, r, &k, p));
        attempt(|| torus(a, big, small, &k, p));
        Ok(())
    });
    end();
}

fn dumped(write: impl FnOnce(&mut DumpStream) -> std::io::Result<()>) {
    let mut stream = DumpStream::default();
    write(&mut stream).unwrap();
    text(stream.text());
}
struct AxisRequest {
    origin: Point3,
    other: Point3,
    frame: Axis2,
    line: Axis1,
    plane: Axis2,
    angle: f64,
    scale: f64,
    linear: f64,
    angular: f64,
}
fn axis1_members(q: &AxisRequest, main: Axis1) -> Result<(), Failure> {
    let l = q.line;
    flag(main.is_coaxial(l, q.angular, q.linear));
    flag(main.is_normal(l, q.angular));
    flag(main.is_opposite(l, q.angular));
    flag(main.is_parallel(l, q.angular));
    real(main.angle(l));
    ax1(main.reversed());
    ax1(main.mirrored_point(q.other));
    ax1(main.mirrored_axis(l));
    ax1(main.mirrored_plane(q.plane));
    ax1(main.rotated(l, q.angle)?);
    ax1(main.scaled(q.other, q.scale));
    ax1(main.transformed(Transform3::rotation(l, q.angle)?));
    ax1(main.translated_between(q.origin, q.other));
    dumped(|stream| main.dump_json(stream));
    Ok(())
}
fn axis2_setters(q: &AxisRequest) {
    let a = q.frame;
    type Setter = fn(&mut Axis2, &AxisRequest) -> Result<(), Failure>;
    let setters: [Setter; 6] = [
        |s, q| s.set_axis(q.line),
        |s, q| s.set_direction(q.line.direction()),
        |s, q| s.set_direction(q.frame.x_direction()),
        |s, q| s.set_direction(q.frame.x_direction().reversed()),
        |s, q| s.set_x_direction(q.line.direction()),
        |s, q| s.set_y_direction(q.line.direction()),
    ];
    for setter in setters {
        let mut s = a;
        attempt(|| {
            setter(&mut s, q)?;
            Ok(ax2(s))
        });
    }
}
fn axis2_members(q: &AxisRequest) {
    let (a, b, l) = (q.frame, q.plane, q.line);
    axis2_setters(q);
    real(a.angle(b));
    flag(a.is_coplanar(b, q.linear, q.angular));
    flag(a.is_coplanar_axis(l, q.linear, q.angular));
    attempt(|| Ok(ax2(a.mirrored_axis(l)?)));
    attempt(|| Ok(ax2(a.mirrored_plane(b)?)));
    ax2(a.translated_between(q.origin, q.other));
    dumped(|stream| a.dump_json(stream));
}
fn axis3_members(q: &AxisRequest) {
    let c = Axis3::from_axis2(q.frame);
    let b = Axis3::from_axis2(q.plane);
    attempt(|| Ok(ax2(c.ax2()?)));
    let mut indirect = c;
    indirect.y_reverse();
    attempt(|| Ok(ax2(indirect.ax2()?)));
    real(c.angle(b));
    flag(c.is_coplanar(b, q.linear, q.angular));
    flag(c.is_coplanar_axis(q.line, q.linear, q.angular));
    ax3(c.mirrored_point(q.other));
    ax3(c.mirrored_axis(q.line));
    ax3(c.translated_between(q.origin, q.other));
}
pub fn gpaxis_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, normal, x) = frame3(&mut fields);
    let (other, line) = (fields.point(), fields.xyz());
    let (angle, scale, linear, angular) =
        (fields.real(), fields.real(), fields.real(), fields.real());
    attempt(|| {
        let frame = Axis2::with_x_direction(origin, direction(normal)?, direction(x)?)?;
        let line = Axis1::new(other, direction(line)?);
        let plane = Axis2::new(other, line.direction())?;
        let q = AxisRequest {
            origin,
            other,
            frame,
            line,
            plane,
            angle,
            scale,
            linear,
            angular,
        };
        axis1_members(&q, Axis1::new(origin, direction(normal)?))?;
        axis2_members(&q);
        axis3_members(&q);
        Ok(())
    });
    end();
}
