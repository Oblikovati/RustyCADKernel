//! The BlendFunc probes, the Rust twin of tools/oracle/tkfillet_probe_blendfunc.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::blend::{
    BlendAppFunction, BlendFuncInv, BlendFunction, BlendInverseBounds, BlendPoint, SectionPoles,
};
use rustycad_tkfillet::blend_func::{ConstRadInv, EvolRadInv, SectionShape};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkgeombase::approx::{SweepD1, SweepD2};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Axis3, Circle3, Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

pub(super) fn vec3(v: Vector3) {
    real(v.x());
    real(v.y());
    real(v.z());
}
fn vec2(v: Vector2) {
    real(v.x());
    real(v.y());
}

/// `fl_surface`: 12 numbers.
pub(super) fn surface(r: &mut Reader<'_>) -> Handle<dyn Adaptor3dSurface> {
    let geom = geom_surface(r);
    Handle::new(GeomAdaptorSurface::with_surface(&geom).expect("adaptor")).cast()
}

/// The surface of `fl_surface`.
pub(super) fn geom_surface(r: &mut Reader<'_>) -> Handle<dyn GeomSurface> {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let (a, b) = (r.real(), r.real());
    let frame = Axis3::new(p, n, x).expect("probe frame");
    match kind {
        0 => Handle::new(GeomPlane::new(frame)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(frame, a).expect("cylinder")).cast(),
        2 => Handle::new(GeomSphericalSurface::new(frame, a).expect("sphere")).cast(),
        3 => Handle::new(GeomConicalSurface::new(frame, a, b).expect("cone")).cast(),
        _ => Handle::new(GeomToroidalSurface::new(frame, a, b).expect("torus")).cast(),
    }
}

/// `fl_guide`: 11 numbers.
pub(super) fn guide(r: &mut Reader<'_>) -> Handle<dyn Adaptor3dCurve> {
    let geom = geom_guide(r);
    Handle::new(GeomAdaptorCurve::with_curve(&geom).expect("adaptor")).cast()
}

/// The curve of `fl_guide`.
pub(super) fn geom_guide(r: &mut Reader<'_>) -> Handle<dyn GeomCurve> {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let a = r.real();
    if kind == 0 {
        Handle::new(GeomLine::from_point(p, n)).cast()
    } else {
        let frame = Axis2::with_x_direction(p, n, x).expect("frame");
        Handle::new(GeomCircle::from_circ(
            Circle3::new(frame, a).expect("circle"),
        ))
        .cast()
    }
}

pub fn vector(v: &Vector<'_>) {
    for i in v.lower()..=v.upper() {
        real(v[i]);
    }
}
pub(super) fn matrix(m: &Matrix<'_>) {
    for i in m.lower_row()..=m.upper_row() {
        for j in m.lower_col()..=m.upper_col() {
            real(m[(i, j)]);
        }
    }
}
fn each<T: Copy>(a: &Array1<'_, T>, print: impl Fn(T)) {
    for i in a.lower()..=a.upper() {
        print(*a.value(i).expect("in bounds"));
    }
}

pub(super) fn tangents(f: &dyn BlendFunction) {
    flag(f.is_tangency_point());
    attempt(|| f.tangent_on_s1().map(vec3));
    attempt(|| f.tangent2d_on_s1().map(vec2));
    attempt(|| f.tangent_on_s2().map(vec3));
    attempt(|| f.tangent2d_on_s2().map(vec2));
    attempt(|| f.twist_on_s1().map(flag));
    attempt(|| f.twist_on_s2().map(flag));
}

fn filled<T: Clone + Default>(n: i32, value: T) -> Array1<'static, T> {
    let mut a = Array1::with_bounds(1, n).expect("bounds");
    a.init(value);
    a
}

/// `fl_sections`.
pub(super) fn sections(f: &mut dyn BlendAppFunction, x: [f64; 4], t: f64) {
    sections_at(
        f,
        &BlendPoint::on_surfaces(Point3::default(), Point3::default(), t, x),
    );
}

/// `fl_sections_at`: the sections of every order at `point`.
pub(super) fn sections_at(f: &mut dyn BlendAppFunction, point: &BlendPoint) {
    let mut shape = (0, 0, 0, 0);
    attempt(|| {
        shape = f.get_shape()?;
        let (a, b, c, d) = shape;
        [a, b, c, d].into_iter().for_each(int);
        Ok(())
    });
    let (np, nk, _, np2) = shape;
    if !(1..=64).contains(&np) || !(1..=64).contains(&nk) {
        return;
    }
    let mut knots = filled(nk, 0.0);
    let mut mults = filled(nk, 0);
    let mut weights = filled(np, 0.0);
    attempt(|| f.knots(&mut knots).map(|()| each(&knots, real)));
    attempt(|| {
        f.mults(&mut mults)
            .map(|()| each(&mults, |m| real(f64::from(m))))
    });
    attempt(|| {
        f.get_minimal_weight(&mut weights)
            .map(|()| each(&weights, real))
    });
    let mut tol3d = Vector::filled(1, np, 0.0).expect("vector");
    let mut tol1d = Vector::filled(1, np2, 0.0).expect("vector");
    attempt(|| {
        f.get_tolerance_3d(1.0e-4, 1.0e-5, 1.0e-3, &mut tol3d, &mut tol1d)?;
        vector(&tol3d);
        vector(&tol1d);
        Ok(())
    });
    section_arrays(f, point, np, np2);
}

fn section_arrays(f: &mut dyn BlendAppFunction, point: &BlendPoint, np: i32, np2: i32) {
    let mut poles = filled(np, Point3::default());
    let mut dp = filled(np, Vector3::default());
    let mut d2p = filled(np, Vector3::default());
    let mut p2 = filled(np2, Point2::default());
    let mut dp2 = filled(np2, Vector2::default());
    let mut d2p2 = filled(np2, Vector2::default());
    let mut weights = filled(np, 0.0);
    let mut dw = filled(np, 0.0);
    let mut d2w = filled(np, 0.0);
    attempt(|| {
        f.section(
            point,
            SectionPoles {
                poles: &mut poles,
                poles2d: &mut p2,
                weights: &mut weights,
            },
        )?;
        each(&poles, pnt);
        each(&p2, pnt2d);
        each(&weights, real);
        Ok(())
    });
    attempt(|| {
        let done = f.section_d1(
            point,
            SweepD1 {
                poles: &mut poles,
                d_poles: &mut dp,
                poles2d: &mut p2,
                d_poles2d: &mut dp2,
                weights: &mut weights,
                d_weights: &mut dw,
            },
        )?;
        flag(done);
        each(&poles, pnt);
        each(&dp, vec3);
        each(&p2, pnt2d);
        each(&dp2, vec2);
        each(&weights, real);
        each(&dw, real);
        Ok(())
    });
    attempt(|| {
        let d1 = SweepD1 {
            poles: &mut poles,
            d_poles: &mut dp,
            poles2d: &mut p2,
            d_poles2d: &mut dp2,
            weights: &mut weights,
            d_weights: &mut dw,
        };
        let done = f.section_d2(
            point,
            SweepD2 {
                d1,
                d2_poles: &mut d2p,
                d2_poles2d: &mut d2p2,
                d2_weights: &mut d2w,
            },
        )?;
        flag(done);
        each(&poles, pnt);
        each(&dp, vec3);
        each(&d2p, vec3);
        each(&p2, pnt2d);
        each(&dp2, vec2);
        each(&d2p2, vec2);
        each(&weights, real);
        each(&dw, real);
        each(&d2w, real);
        Ok(())
    });
}

/// `fl_app_queries`.
pub(super) fn app_queries(f: &dyn BlendAppFunction, tol: f64) {
    let mut inf = Vector::filled(1, 4, 0.0).expect("vector");
    let mut sup = Vector::filled(1, 4, 0.0).expect("vector");
    let mut tv = Vector::filled(1, 4, 0.0).expect("vector");
    attempt(|| {
        f.get_bounds(&mut inf, &mut sup)?;
        vector(&inf);
        vector(&sup);
        Ok(())
    });
    attempt(|| f.get_tolerance(&mut tv, tol).map(|()| vector(&tv)));
    attempt(|| {
        let n = f.nb_intervals(Shape::C1)?;
        int(n);
        let mut iv = filled(n + 1, 0.0);
        f.intervals(&mut iv, Shape::C1)?;
        each(&iv, real);
        Ok(())
    });
    for index in [1, 2] {
        attempt(|| {
            f.resolution(index, tol).map(|(u, v)| {
                real(u);
                real(v)
            })
        });
    }
    attempt(|| f.get_minimal_distance().map(real));
    attempt(|| f.get_section_size().map(real));
    attempt(|| f.is_rational().map(flag));
}

pub(super) fn read_x(r: &mut Reader<'_>) -> [f64; 4] {
    [r.real(), r.real(), r.real(), r.real()]
}

pub(super) fn to_vector(x: [f64; 4]) -> Vector<'static> {
    let mut v = Vector::filled(1, 4, 0.0).expect("vector");
    for (i, value) in (1..).zip(x) {
        v[i] = value;
    }
    v
}

/// The Value, Derivatives and Values of a 4-variable function at `x`; each step prints
/// its flag and outputs, or the exception its callback kept.
pub(super) fn equations(f: &mut dyn BlendAppFunction, x: &Vector<'_>) {
    let mut fv = Vector::filled(1, 4, 0.0).expect("vector");
    let mut d = Matrix::filled(1, 4, 1, 4, 0.0).expect("matrix");
    let ok = f.value(x, &mut fv);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            vector(&fv)
        })
    });
    let ok = f.derivatives(x, &mut d);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            matrix(&d)
        })
    });
    let ok = f.values(x, &mut fv, &mut d);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            vector(&fv);
            matrix(&d)
        })
    });
}

pub(super) fn shape_of(code: i32) -> SectionShape {
    [
        SectionShape::Rational,
        SectionShape::QuasiAngular,
        SectionShape::Polynomial,
        SectionShape::Linear,
    ][code as usize]
}

/// `fl_curve2d`: the line through `(u, v)` along `(du, dv)`.
pub(super) fn curve2d(r: &mut Reader<'_>) -> Handle<dyn Adaptor2dCurve2d> {
    let (p, d) = (r.pnt2d(), r.dir2d());
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p, d)).cast();
    Handle::new(Geom2dAdaptorCurve::with_curve(&line).expect("adaptor")).cast()
}

/// `fl_inverse`: the math members and the bounds of an inverse function at `x`.
pub(super) fn inverse(f: &mut dyn BlendInverseBounds, x: &Vector<'_>, tol: f64) {
    let (nv, ne) = (f.variables(), f.equations());
    int(nv);
    int(ne);
    let mut fv = Vector::filled(1, ne, 0.0).expect("vector");
    let mut d = Matrix::filled(1, ne, 1, nv, 0.0).expect("matrix");
    let ok = f.value(x, &mut fv);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            vector(&fv)
        })
    });
    let ok = f.derivatives(x, &mut d);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            matrix(&d)
        })
    });
    let ok = f.values(x, &mut fv, &mut d);
    attempt(|| {
        f.take_failure().map(|()| {
            flag(ok);
            vector(&fv);
            matrix(&d)
        })
    });
    attempt(|| f.is_solution(x, tol).map(flag));
    let mut tv = Vector::filled(1, nv, 0.0).expect("vector");
    attempt(|| f.get_tolerance(&mut tv, tol).map(|()| vector(&tv)));
    let mut inf = Vector::filled(1, nv, 0.0).expect("vector");
    let mut sup = Vector::filled(1, nv, 0.0).expect("vector");
    attempt(|| {
        f.get_bounds(&mut inf, &mut sup)?;
        vector(&inf);
        vector(&sup);
        Ok(())
    });
}

/// `fl_const_rad_inv S1 S2 C P2d D2d radius choix first X1..X4 tol`.
pub fn const_rad_inv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let cs = curve2d(&mut r);
    let (radius, choix, first) = (r.real(), r.integer(), r.flag());
    let (x, tol) = (read_x(&mut r), r.real());
    let mut f = ConstRadInv::new(&s1, &s2, &c);
    f.set_radius(radius, choix);
    let _ = f.set_curve(first, &cs);
    inverse(&mut f, &to_vector(x), tol);
    end();
}

/// `fl_evol_rad_inv S1 S2 C P2d D2d Law choix first X1..X4 tol`.
pub fn evol_rad_inv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let (cs, radius_law) = (curve2d(&mut r), super::fillet::law(&mut r));
    let (choix, first) = (r.integer(), r.flag());
    let (x, tol) = (read_x(&mut r), r.real());
    let mut f = EvolRadInv::new(&s1, &s2, &c, &radius_law);
    f.set_choix(choix);
    let _ = f.set_curve(first, &cs);
    inverse(&mut f, &to_vector(x), tol);
    end();
}
