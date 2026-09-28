//! gp_Circ2d/Elips2d/Hypr2d/Parab2d, gp_Ax22d and gp_Ax2d member requests (gpconic2).
use super::{Fields, attempt, ax2d, ax22d, end, flag, pnt2, real, reals, text};
use rustycad_tkernel::standard::{Failure, dump::DumpStream};
use rustycad_tkmath::gp::{
    Axis2d, Axis22d, Circle2, Direction2, Ellipse2, Hyperbola2, Parabola2, Point2, Transform2,
    Vector2,
};

struct Motion2 {
    pivot: Point2,
    axis: Axis2d,
    angle: f64,
    scale: f64,
    shift: Vector2,
    transform: Transform2,
}

/// The shared positional block of the native `el_placed2` template.
macro_rules! placed2 {
    ($c:expr, $k:expr, $p:expr) => {{
        let (c, k, p) = ($c, $k, $p);
        ax22d(c.axis());
        flag(c.is_direct());
        ax22d(c.reversed().axis());
        ax22d(c.mirrored_point(k.pivot).axis());
        ax22d(c.mirrored_axis(k.axis).axis());
        ax22d(c.rotated(k.pivot, k.angle).axis());
        ax22d(c.translated(k.shift).axis());
        ax22d(c.translated_between(k.pivot, p).axis());
        ax22d(c.scaled(k.pivot, k.scale).axis());
        ax22d(c.transformed(k.transform).axis());
        reals(&c.coefficients());
    }};
}
/// The native `el_axes2` template.
macro_rules! axes2 {
    ($c:expr, $k:expr) => {{
        let (c, k) = ($c, $k);
        ax2d(c.x_axis());
        ax2d(c.y_axis());
        let mut x = c;
        x.set_x_axis(k.axis);
        ax22d(x.axis());
        let mut y = c;
        y.set_y_axis(k.axis);
        ax22d(y.axis());
    }};
}
/// The native `el_radii2` template, including the in-place Transform.
macro_rules! radii2 {
    ($c:expr, $k:expr) => {{
        let (c, k) = ($c, $k);
        let s = c.scaled(k.pivot, k.scale);
        reals(&[s.major_radius(), s.minor_radius()]);
        let t = c.transformed(k.transform);
        reals(&[t.major_radius(), t.minor_radius()]);
        let mut u = c;
        u.transform(k.transform);
        reals(&[u.major_radius(), u.minor_radius()]);
    }};
}

fn circle2(a: Axis22d, r: f64, k: &Motion2, p: Point2) -> Result<(), Failure> {
    let c = Circle2::new(a, r)?;
    reals(&[c.area(), c.length(), c.distance(p), c.square_distance(p)]);
    flag(c.contains(p, 1e-7));
    placed2!(c, k, p);
    axes2!(c, k);
    real(c.scaled(k.pivot, k.scale).radius());
    real(c.transformed(k.transform).radius());
    Ok(())
}
fn ellipse2(a: Axis22d, big: f64, small: f64, k: &Motion2, p: Point2) -> Result<(), Failure> {
    let e = Ellipse2::new(a, big, small)?;
    reals(&[e.area(), e.eccentricity(), e.focal()]);
    pnt2(e.focus1());
    pnt2(e.focus2());
    real(e.parameter());
    attempt(|| {
        ax2d(e.directrix1()?);
        Ok(())
    });
    attempt(|| {
        ax2d(e.directrix2()?);
        Ok(())
    });
    placed2!(e, k, p);
    axes2!(e, k);
    radii2!(e, k);
    Ok(())
}
fn hyperbola2(a: Axis22d, big: f64, small: f64, k: &Motion2, p: Point2) -> Result<(), Failure> {
    let h = Hyperbola2::new(a, big, small)?;
    real(h.focal());
    pnt2(h.focus1());
    pnt2(h.focus2());
    attempt(|| {
        real(h.eccentricity()?);
        Ok(())
    });
    attempt(|| {
        real(h.parameter()?);
        Ok(())
    });
    for axis in [
        h.asymptote1(),
        h.asymptote2(),
        h.directrix1(),
        h.directrix2(),
    ] {
        attempt(|| {
            ax2d(axis?);
            Ok(())
        });
    }
    ax22d(h.conjugate_branch1().axis());
    ax22d(h.conjugate_branch2().axis());
    ax22d(h.other_branch().axis());
    real(h.conjugate_branch1().major_radius());
    placed2!(h, k, p);
    axes2!(h, k);
    radii2!(h, k);
    Ok(())
}
fn parabola2(a: Axis22d, focal: f64, sense: bool, k: &Motion2, p: Point2) -> Result<(), Failure> {
    let c = Parabola2::new(a, focal)?;
    real(c.parameter());
    pnt2(c.focus());
    ax2d(c.directrix());
    ax2d(c.axis_of_symmetry());
    placed2!(c, k, p);
    let mut x = c;
    x.set_mirror_axis(k.axis);
    ax22d(x.axis());
    real(c.scaled(k.pivot, k.scale).focal());
    attempt(|| {
        let d = Parabola2::from_directrix(k.axis, p, sense)?;
        ax22d(d.axis());
        real(d.focal());
        Ok(())
    });
    Ok(())
}
fn dumped(write: impl FnOnce(&mut DumpStream) -> std::io::Result<()>) {
    let mut stream = DumpStream::default();
    write(&mut stream).unwrap();
    text(stream.text());
}
fn frame2(a: Axis22d, k: &Motion2, p: Point2) {
    ax22d(Axis22d::with_directions(
        a.location(),
        a.x_direction(),
        k.axis.direction(),
    ));
    let mut g = a;
    g.set_x_direction(k.axis.direction());
    ax22d(g);
    g = a;
    g.set_y_direction(k.axis.direction());
    ax22d(g);
    let l = Axis2d::new(a.location(), a.x_direction());
    flag(l.is_coaxial(k.axis, 1e-6, 1e-6));
    real(l.angle(k.axis));
    ax2d(l.mirrored_axis(k.axis));
    ax2d(l.rotated(k.pivot, k.angle));
    ax2d(l.scaled(k.pivot, k.scale));
    ax2d(l.translated_between(k.pivot, p));
    ax2d(l.transformed(k.transform));
    dumped(|stream| l.dump_json(stream, -1));
    dumped(|stream| l.dump_json(stream, 0));
    dumped(|stream| a.dump_json(stream));
}
pub fn gpconic2_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, x) = (fields.point2(), fields.xy());
    let sense = fields.int() != 0;
    let (big, small, focal, r) = (fields.real(), fields.real(), fields.real(), fields.real());
    let (pivot, axis) = (fields.point2(), fields.xy());
    let (angle, scale) = (fields.real(), fields.real());
    let (shift, p) = (Vector2::from(fields.xy()), fields.point2());
    attempt(|| {
        let a = Axis22d::new(origin, Direction2::from_xy(x)?, sense);
        let axis = Axis2d::new(pivot, Direction2::from_xy(axis)?);
        let transform =
            Transform2::rotation(pivot, angle).multiplied(Transform2::mirror_axis(axis));
        let k = Motion2 {
            pivot,
            axis,
            angle,
            scale,
            shift,
            transform,
        };
        frame2(a, &k, p);
        attempt(|| circle2(a, r, &k, p));
        attempt(|| ellipse2(a, big, small, &k, p));
        attempt(|| hyperbola2(a, big, small, &k, p));
        attempt(|| parabola2(a, focal, sense, &k, p));
        Ok(())
    });
    end();
}
