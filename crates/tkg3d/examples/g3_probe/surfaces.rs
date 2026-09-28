//! Surface evaluation, surface moves and the elementary surfaces; the counterpart of
//! the first half of tools/oracle/tkg3d_probe_surfaces.cxx.
use super::common::{
    Reader, attempt, ax1, ax3, built_surface, dump, end, gtrsf2d, isos, name, pnt, real, show,
    surface_summary, trsf,
};
use super::simple::moves;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface, GeomGeometry, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Sphere3, Torus3};
use std::cell::RefCell;

pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let (uv, iso_params, params) = (r.reals(), r.reals(), r.reals());
    name(&s);
    surface_summary(&*s, &uv);
    isos(&*s, &iso_params, &params);
    for depth in [-1, 1, 0] {
        dump(&*s, depth);
    }
    end();
}

thread_local! {
    static PROBE_UV: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

/// Prints the summary of a moved surface at the (u, v) pairs of the request (g3_show_surface).
pub fn show_surface(g: &Handle<dyn GeomGeometry>) {
    let s: Handle<dyn GeomSurface> = g.cast();
    PROBE_UV.with(|uv| surface_summary(&*s, &uv.borrow()));
}

pub fn surface_moves_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let uv = r.reals();
    PROBE_UV.with(|p| *p.borrow_mut() = uv.clone());
    let t = trsf(&mut r);
    for pair in uv.as_chunks::<2>().0 {
        let (u, v) = s.transform_parameters(pair[0], pair[1], t);
        real(u);
        real(v);
    }
    show(s.parametric_transformation(t), gtrsf2d);
    show(s.u_reversed(), |c| show_surface(&c.cast()));
    show(s.v_reversed(), |c| show_surface(&c.cast()));
    show(s.copy(), |c| show_surface(&c));
    show(s.u_reverse(), |()| show_surface(&s.cast()));
    show(s.v_reverse(), |()| show_surface(&s.cast()));
    moves(&s.cast(), &mut r, show_surface);
    dump(&*s, -1);
    end();
}

fn elementary_common(e: &dyn GeomElementarySurface) {
    ax1(e.axis());
    pnt(e.location());
    ax3(e.position());
}

fn plane_specific(p: &GeomPlane) {
    show(p.coefficients(), |c| c.into_iter().for_each(real));
    ax3(p.pln().position());
    p.set_pln(Plane3::from_position(p.position()));
}

fn cylinder_specific(p: &GeomCylindricalSurface, a: f64) {
    real(p.radius());
    p.coefficients().into_iter().for_each(real);
    show(p.cylinder(), |g: Cylinder3| {
        ax3(g.position());
        real(g.radius());
        p.set_cylinder(g);
    });
    attempt(|| p.set_radius(a));
    real(p.radius());
}

fn cone_specific(p: &GeomConicalSurface, a: f64, b: f64) {
    real(p.ref_radius());
    real(p.semi_angle());
    pnt(p.apex());
    p.coefficients().into_iter().for_each(real);
    show(p.cone(), |g: Cone3| {
        ax3(g.position());
        real(g.semi_angle());
        p.set_cone(g);
    });
    attempt(|| p.set_radius(a));
    attempt(|| p.set_semi_angle(b));
    real(p.ref_radius());
    real(p.semi_angle());
    pnt(p.apex());
}

fn sphere_specific(p: &GeomSphericalSurface, a: f64) {
    [p.radius(), p.area(), p.volume()]
        .into_iter()
        .for_each(real);
    p.coefficients().into_iter().for_each(real);
    show(p.sphere(), |g: Sphere3| {
        ax3(g.position());
        p.set_sphere(g);
    });
    attempt(|| p.set_radius(a));
    real(p.radius());
}

fn torus_specific(p: &GeomToroidalSurface, a: f64, b: f64) {
    [p.major_radius(), p.minor_radius(), p.area(), p.volume()]
        .into_iter()
        .for_each(real);
    let mut coefficients = Array1::<f64>::with_bounds(1, 35).unwrap();
    let mut small = Array1::<f64>::with_bounds(1, 30).unwrap();
    attempt(|| {
        p.coefficients(&mut coefficients)?;
        coefficients.as_slice().iter().copied().for_each(real);
        Ok(())
    });
    attempt(|| p.coefficients(&mut small));
    show(p.torus(), |g: Torus3| {
        ax3(g.position());
        p.set_torus(g);
    });
    attempt(|| p.set_major_radius(a));
    attempt(|| p.set_minor_radius(b));
    real(p.major_radius());
    real(p.minor_radius());
}

// The class getters and setters of g3_elementary_specific.
fn elementary_specific(e: &Handle<dyn GeomElementarySurface>, a: f64, b: f64) {
    if let Some(p) = e.cast::<GeomPlane>().get() {
        plane_specific(p);
    }
    if let Some(p) = e.cast::<GeomCylindricalSurface>().get() {
        cylinder_specific(p, a);
    }
    if let Some(p) = e.cast::<GeomConicalSurface>().get() {
        cone_specific(p, a, b);
    }
    if let Some(p) = e.cast::<GeomSphericalSurface>().get() {
        sphere_specific(p, a);
    }
    if let Some(p) = e.cast::<GeomToroidalSurface>().get() {
        torus_specific(p, a, b);
    }
}

pub fn elementary_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = built_surface(&mut r) else {
        return;
    };
    let uv = r.reals();
    let e: Handle<dyn GeomElementarySurface> = s.cast();
    let (a, b) = (r.real(), r.real());
    let (axis, p, f) = (r.ax1(), r.pnt(), r.ax3());
    elementary_common(&*e);
    elementary_specific(&e, a, b);
    surface_summary(&*e, &uv);
    e.set_location(p);
    elementary_common(&*e);
    attempt(|| e.set_axis(axis));
    elementary_common(&*e);
    e.set_position(f);
    elementary_common(&*e);
    surface_summary(&*e, &uv);
    dump(&*e, -1);
    end();
}

// The constructor of g3_elementary_build for `kind`, the error printed when it raises.
fn build_elementary(kind: i32, r: &mut Reader) -> Option<Handle<dyn GeomElementarySurface>> {
    let f = r.ax3();
    let (a, b, p, n) = (r.real(), r.real(), r.pnt(), r.dir());
    let c: Vec<f64> = (0..4).map(|_| r.real()).collect();
    let built: Result<Handle<dyn GeomElementarySurface>, Failure> = match kind {
        0 => Ok(Handle::new(GeomPlane::from_pln(Plane3::from_position(f))).cast()),
        1 => Cylinder3::new(f, a)
            .map(|g| Handle::new(GeomCylindricalSurface::from_cylinder(g)).cast()),
        2 => Cone3::new(f, b, a).map(|g| Handle::new(GeomConicalSurface::from_cone(g)).cast()),
        3 => Sphere3::new(f, a).map(|g| Handle::new(GeomSphericalSurface::from_sphere(g)).cast()),
        4 => Torus3::new(f, a, b).map(|g| Handle::new(GeomToroidalSurface::from_torus(g)).cast()),
        5 => GeomPlane::from_point(p, n).map(|g| Handle::new(g).cast()),
        _ => GeomPlane::from_coefficients(c[0], c[1], c[2], c[3]).map(|g| Handle::new(g).cast()),
    };
    match built {
        Ok(e) => Some(e),
        Err(error) => {
            print!("{} ", error.exception_type());
            None
        }
    }
}

pub fn elementary_build_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let kind = r.integer();
    if let Some(e) = build_elementary(kind, &mut r) {
        elementary_common(&*e);
        dump(&*e, -1);
    }
    end();
}
