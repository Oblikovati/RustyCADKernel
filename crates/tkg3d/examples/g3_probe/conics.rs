//! Conic getters, setters and gp constructors; the counterpart of the conic
//! functions of tools/oracle/tkg3d_probe_curves.cxx.
use super::common::{
    Reader, attempt, ax1, ax2, built_curve, curve_summary, dump, end, pnt, real, show,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomConic, GeomEllipse, GeomHyperbola, GeomParabola};
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Ellipse3, Hyperbola3, Parabola3, Point3};

fn conic_common(c: &dyn GeomConic) {
    ax1(c.x_axis());
    ax1(c.y_axis());
    ax1(c.axis());
    pnt(c.location());
    ax2(c.position());
    show(c.eccentricity(), real);
}

fn circle_specific(e: &GeomCircle, value: f64) {
    real(e.radius());
    attempt(|| e.set_radius(value));
    real(e.radius());
    show(e.circ(), |g| {
        ax2(g.position());
        real(g.radius());
        e.set_circ(g);
    });
}

fn ellipse_specific(e: &GeomEllipse, value: f64) {
    real(e.major_radius());
    real(e.minor_radius());
    show(e.directrix1(), ax1);
    show(e.directrix2(), ax1);
    real(e.focal());
    pnt(e.focus1());
    pnt(e.focus2());
    real(e.parameter());
    attempt(|| e.set_minor_radius(value));
    attempt(|| e.set_major_radius(value));
    real(e.major_radius());
    real(e.minor_radius());
    show(e.elips(), |g| {
        ax2(g.position());
        e.set_elips(g);
    });
}

fn hyperbola_specific(e: &GeomHyperbola, value: f64) {
    real(e.major_radius());
    real(e.minor_radius());
    show(e.asymptote1(), ax1);
    show(e.asymptote2(), ax1);
    show(e.conjugate_branch1(), |g| ax2(g.position()));
    show(e.conjugate_branch2(), |g| ax2(g.position()));
    show(e.directrix1(), ax1);
    show(e.directrix2(), ax1);
    real(e.focal());
    pnt(e.focus1());
    pnt(e.focus2());
    show(e.other_branch(), |g| ax2(g.position()));
    show(e.parameter(), real);
    attempt(|| e.set_major_radius(value));
    attempt(|| e.set_minor_radius(-value));
    real(e.major_radius());
    real(e.minor_radius());
    show(e.hypr(), |g| {
        ax2(g.position());
        e.set_hypr(g);
    });
}

fn parabola_specific(e: &GeomParabola, value: f64) {
    real(e.focal());
    pnt(e.focus());
    real(e.parameter());
    show(e.directrix(), ax1);
    attempt(|| e.set_focal(value));
    real(e.focal());
    show(e.parab(), |g| {
        ax2(g.position());
        e.set_parab(g);
    });
}

fn conic_specific(c: &Handle<dyn GeomConic>, value: f64) {
    if let Some(e) = c.cast::<GeomCircle>().get() {
        circle_specific(e, value);
    }
    if let Some(e) = c.cast::<GeomEllipse>().get() {
        ellipse_specific(e, value);
    }
    if let Some(e) = c.cast::<GeomHyperbola>().get() {
        hyperbola_specific(e, value);
    }
    if let Some(e) = c.cast::<GeomParabola>().get() {
        parabola_specific(e, value);
    }
}

pub fn conic_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(curve) = built_curve(&mut r) else {
        return;
    };
    let u = r.reals();
    let c: Handle<dyn GeomConic> = curve.cast();
    let (value, a, p, f) = (r.real(), r.ax1(), r.pnt(), r.ax2());
    conic_common(&*c);
    conic_specific(&c, value);
    curve_summary(&*c, &u);
    c.set_location(p);
    conic_common(&*c);
    attempt(|| c.set_axis(a));
    conic_common(&*c);
    c.set_position(f);
    conic_common(&*c);
    c.reverse().unwrap();
    conic_common(&*c);
    curve_summary(&*c, &u);
    dump(&*c, -1);
    end();
}

/// The conic built from gp objects, one kind per request (g3_conic_build).
fn build_conic(
    kind: i32,
    f: Axis2,
    radii: (f64, f64),
    directrix: (Axis1, Point3),
) -> Result<Handle<dyn GeomConic>, Failure> {
    let (a, b) = radii;
    Ok(match kind {
        1 => Handle::new(GeomCircle::from_circ(Circle3::new(f, a)?)).cast(),
        2 => Handle::new(GeomEllipse::from_elips(Ellipse3::new(f, a, b)?)).cast(),
        3 => Handle::new(GeomHyperbola::from_hypr(Hyperbola3::new(f, a, b)?)).cast(),
        4 => Handle::new(GeomParabola::from_parab(Parabola3::new(f, a)?)).cast(),
        _ => Handle::new(GeomParabola::from_directrix(directrix.0, directrix.1)?).cast(),
    })
}

pub fn conic_build_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    let (f, a, b) = (r.ax2(), r.real(), r.real());
    let (d, p) = (r.ax1(), r.pnt());
    match build_conic(kind, f, (a, b), (d, p)) {
        Ok(c) => {
            conic_common(&*c);
            dump(&*c, -1);
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    end();
}
