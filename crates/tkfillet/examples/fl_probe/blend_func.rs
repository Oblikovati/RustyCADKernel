//! The BlendFunc probes, the Rust twin of tools/oracle/tkfillet_probe_blendfunc.cxx.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, pnt2d, real};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::blend::{
    BlendAppFunction, BlendFuncInv, BlendFunction, BlendInverseBounds, BlendPoint, SectionPoles,
};
use rustycad_tkfillet::blend_func::{ConstRad, ConstRadInv, SectionShape};
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

pub fn vec3(v: Vector3) {
    real(v.x());
    real(v.y());
    real(v.z());
}
fn vec2(v: Vector2) {
    real(v.x());
    real(v.y());
}

/// `fl_surface`: 12 numbers.
fn surface(r: &mut Reader<'_>) -> Handle<dyn Adaptor3dSurface> {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let (a, b) = (r.real(), r.real());
    let frame = Axis3::new(p, n, x).expect("probe frame");
    let geom: Handle<dyn GeomSurface> = match kind {
        0 => Handle::new(GeomPlane::new(frame)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(frame, a).expect("cylinder")).cast(),
        2 => Handle::new(GeomSphericalSurface::new(frame, a).expect("sphere")).cast(),
        3 => Handle::new(GeomConicalSurface::new(frame, a, b).expect("cone")).cast(),
        _ => Handle::new(GeomToroidalSurface::new(frame, a, b).expect("torus")).cast(),
    };
    Handle::new(GeomAdaptorSurface::with_surface(&geom).expect("adaptor")).cast()
}

/// `fl_guide`: 11 numbers.
fn guide(r: &mut Reader<'_>) -> Handle<dyn Adaptor3dCurve> {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let a = r.real();
    let geom: Handle<dyn GeomCurve> = if kind == 0 {
        Handle::new(GeomLine::from_point(p, n)).cast()
    } else {
        let frame = Axis2::with_x_direction(p, n, x).expect("frame");
        Handle::new(GeomCircle::from_circ(
            Circle3::new(frame, a).expect("circle"),
        ))
        .cast()
    };
    Handle::new(GeomAdaptorCurve::with_curve(&geom).expect("adaptor")).cast()
}

pub fn vector(v: &Vector<'_>) {
    for i in v.lower()..=v.upper() {
        real(v[i]);
    }
}
fn matrix(m: &Matrix<'_>) {
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

fn tangents(f: &dyn BlendFunction) {
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
fn sections(f: &mut dyn BlendAppFunction, x: [f64; 4], t: f64) {
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
    let point = BlendPoint::on_surfaces(Point3::default(), Point3::default(), t, x);
    section_arrays(f, &point, np, np2);
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
fn app_queries(f: &dyn BlendAppFunction, tol: f64) {
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

fn read_x(r: &mut Reader<'_>) -> [f64; 4] {
    [r.real(), r.real(), r.real(), r.real()]
}

fn to_vector(x: [f64; 4]) -> Vector<'static> {
    let mut v = Vector::filled(1, 4, 0.0).expect("vector");
    for (i, value) in (1..).zip(x) {
        v[i] = value;
    }
    v
}

/// The Value, Derivatives and Values of a 4-variable function at `x`; each step prints
/// its flag and outputs, or the exception its callback kept.
fn equations(f: &mut dyn BlendAppFunction, x: &Vector<'_>) {
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

fn shape_of(code: i32) -> SectionShape {
    [
        SectionShape::Rational,
        SectionShape::QuasiAngular,
        SectionShape::Polynomial,
        SectionShape::Linear,
    ][code as usize]
}

/// `fl_const_rad S1 S2 C radius choix shape t trim first last X1..X4 tol`.
pub fn const_rad_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s1, s2, c) = (surface(&mut r), surface(&mut r), guide(&mut r));
    let (radius, choix, shape, t) = (r.real(), r.integer(), r.integer(), r.real());
    let (trim, first, last) = (r.flag(), r.real(), r.real());
    let (x, tol) = (read_x(&mut r), r.real());
    let mut f = ConstRad::new(&s1, &s2, &c).expect("function");
    f.set_radius(radius, choix);
    f.set_shape(shape_of(shape));
    if trim {
        attempt(|| f.set_interval(first, last));
    }
    let _ = f.set_param(t);
    let xv = to_vector(x);
    equations(&mut f, &xv);
    attempt(|| f.is_solution(&xv, tol).map(flag));
    tangents(&f);
    attempt(|| {
        let frames = f.tangent(x[0], x[1], x[2], x[3])?;
        [
            frames.tg_first,
            frames.tg_last,
            frames.norm_first,
            frames.norm_last,
        ]
        .into_iter()
        .for_each(vec3);
        Ok(())
    });
    attempt(|| circle_section(&mut f, t, x));
    sections(&mut f, x, t);
    app_queries(&f, tol);
    attempt(|| {
        let axis = f.axe_rot(t)?;
        pnt(axis.location());
        dir(axis.direction());
        Ok(())
    });
    end();
}

fn circle_section(f: &mut ConstRad, t: f64, x: [f64; 4]) -> Result<(), Failure> {
    let (pdeb, pfin, circle) = f.section_circle(t, x)?;
    real(pdeb);
    real(pfin);
    let frame = circle.position();
    pnt(frame.location());
    dir(frame.direction());
    dir(frame.x_direction());
    real(circle.radius());
    Ok(())
}

/// `fl_curve2d`: the line through `(u, v)` along `(du, dv)`.
fn curve2d(r: &mut Reader<'_>) -> Handle<dyn Adaptor2dCurve2d> {
    let (p, d) = (r.pnt2d(), r.dir2d());
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(p, d)).cast();
    Handle::new(Geom2dAdaptorCurve::with_curve(&line).expect("adaptor")).cast()
}

/// `fl_inverse`: the math members and the bounds of an inverse function at `x`.
fn inverse(f: &mut dyn BlendInverseBounds, x: &Vector<'_>, tol: f64) {
    int(f.variables());
    int(f.equations());
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
    attempt(|| f.is_solution(x, tol).map(flag));
    let mut tv = Vector::filled(1, 4, 0.0).expect("vector");
    attempt(|| f.get_tolerance(&mut tv, tol).map(|()| vector(&tv)));
    let mut inf = Vector::filled(1, 4, 0.0).expect("vector");
    let mut sup = Vector::filled(1, 4, 0.0).expect("vector");
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
