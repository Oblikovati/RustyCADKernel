//! BRepGProp probe operations on the face tool, the domain, the Gauss integrator and the
//! integrand functions, the Rust twin of tools/oracle/tktopalgo_probe_brepgprop_tools.cxx.
use super::brep_class3d::index_of;
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, show, xy, xyz};
use super::shape::tp_shape;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::gprop::ValueType;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::{Matrix3, Point3, Xyz};
use rustycad_tkmath::math::Function;
use rustycad_tktopalgo::brep_gprop::{
    BRepGPropDomain, BRepGPropFace, BRepGPropGauss, BRepGPropTFunction, BRepGPropUFunction,
    GaussProps, GaussType,
};

/// `tp_knots_out`: lower bound, length and values.
fn knots_out(k: &Array1<'_, f64>) {
    int(k.lower());
    int(k.length());
    k.iter().copied().for_each(real);
}

fn knot_array(n: i32) -> Result<Array1<'static, f64>, Failure> {
    Array1::with_bounds(1, n + 1).map_err(|e| e.into_failure("probe"))
}

/// `tp_face_orders`: the orders, subdivisions and knots of the loaded face.
fn face_orders(f: &BRepGPropFace, eps: f64) {
    show(f.u_integration_order(), int);
    show(f.v_integration_order(), int);
    show(f.s_int_order(eps), int);
    attempt(|| {
        let n = f.su_int_subs()?;
        int(n);
        let mut k = knot_array(n)?;
        f.u_knots(&mut k)?;
        k.iter().copied().for_each(real);
        Ok(())
    });
    attempt(|| {
        let n = f.sv_int_subs()?;
        int(n);
        let mut k = knot_array(n)?;
        f.v_knots(&mut k)?;
        k.iter().copied().for_each(real);
        Ok(())
    });
}

/// `tp_face_boundary`: the boundary members after a Load.
fn face_boundary(f: &BRepGPropFace, eps: f64) -> Result<(), Failure> {
    let (a, b) = (f.first_parameter()?, f.last_parameter()?);
    real(a);
    real(b);
    show(f.integration_order(), int);
    show(f.l_int_order(eps), int);
    attempt(|| {
        let n = f.l_int_subs()?;
        int(n);
        let mut k = knot_array(n)?;
        f.l_knots(&mut k)?;
        k.iter().copied().for_each(real);
        Ok(())
    });
    attempt(|| {
        pnt2d(f.value2d(0.5 * (a + b))?);
        let (p, v) = f.d12d(0.37 * a + 0.63 * b)?;
        pnt2d(p);
        xy(v.xy());
        Ok(())
    });
    show(f.get_t_knots(a, b), |k| knots_out(&k));
    Ok(())
}

/// `shape face span eps`: the face tool over its bounds, edges and iso boundaries.
pub fn face_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let face = s.faces[r.integer() as usize].clone();
        let (span, eps) = (r.flag(), r.real());
        let mut f = BRepGPropFace::with_face(&face, span)?;
        int(index_of(&s.faces, &f.get_face()));
        int(f.get_face().orientation() as i32);
        show(f.natural_restriction(), flag);
        let (u1, u2, v1, v2) = f.bounds()?;
        [u1, u2, v1, v2].into_iter().for_each(real);
        face_orders(&f, eps);
        if ![u1, u2, v1, v2].into_iter().any(precision::is_infinite) {
            face_samples(&f, (u1, u2, v1, v2));
        }
        let mut d = BRepGPropDomain::with_face(&face)?;
        while d.more() {
            let e = d.value()?;
            int(index_of(&s.edges, &e));
            int(e.orientation() as i32);
            attempt(|| {
                let ok = f.load_edge(&e)?;
                flag(ok);
                if ok {
                    face_boundary(&f, eps)?;
                }
                Ok(())
            });
            d.next()?;
        }
        for iso in [IsoType::IsoU, IsoType::IsoV, IsoType::NoneIso] {
            for first in [false, true] {
                attempt(|| {
                    f.load_iso(first, iso)?;
                    face_boundary(&f, eps)
                });
            }
        }
        Ok(())
    });
    end();
}

/// The normals at three points and the U knots of finite bounds.
fn face_samples(f: &BRepGPropFace, (u1, u2, v1, v2): (f64, f64, f64, f64)) {
    for (wu, wv) in [(0.25, 0.25), (0.5, 0.75), (0.9, 0.1)] {
        attempt(|| {
            let (p, n) = f.normal(u1 + wu * (u2 - u1), v1 + wv * (v2 - v1))?;
            pnt(p);
            xyz(*n.xyz());
            Ok(())
        });
    }
    show(f.get_u_knots(u1, u2), |k| knots_out(&k));
    let inner = f.get_u_knots(u1 + 0.3 * (u2 - u1), u2 - 0.1 * (u2 - u1));
    show(inner, |k| knots_out(&k));
}

/// `shape face span edge umin umax tmin tmax`: GetUKnots and GetTKnots on ranges.
pub fn knots_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let face = s.faces[r.integer() as usize].clone();
        let (span, e) = (r.flag(), r.integer());
        let (umin, umax, tmin, tmax) = (r.real(), r.real(), r.real(), r.real());
        let mut f = BRepGPropFace::with_face(&face, span)?;
        if e < 0 {
            f.load_iso(false, IsoType::IsoU)?;
        } else {
            flag(f.load_edge(&s.edges[e as usize])?);
        }
        show(f.get_u_knots(umin, umax), |k| knots_out(&k));
        show(f.get_t_knots(tmin, tmax), |k| knots_out(&k));
        let mut k = Array1::default();
        attempt(|| {
            f.get_u_knots_into(umin, umax, &mut k)?;
            knots_out(&k);
            Ok(())
        });
        attempt(|| {
            f.get_t_knots_into(tmin, tmax, &mut k)?;
            knots_out(&k);
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `shape face edge which lower n`: UKnots, VKnots or LKnots into a prefilled array.
pub fn arrays_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let face = s.faces[r.integer() as usize].clone();
        let (e, which, lower, n) = (r.integer(), r.integer(), r.integer(), r.integer());
        let mut f = BRepGPropFace::with_face(&face, false)?;
        if e >= 0 {
            f.load_edge(&s.edges[e as usize])?;
        }
        let mut k =
            Array1::with_bounds(lower, lower + n - 1).map_err(|e| e.into_failure("probe"))?;
        k.init(-1.);
        attempt(|| match which {
            0 => f.u_knots(&mut k),
            1 => f.v_knots(&mut k),
            _ => f.l_knots(&mut k),
        });
        k.iter().copied().for_each(real);
        Ok(())
    });
    end();
}

/// `shape face`: the domain, Value past the end, Init, then an empty domain.
pub fn domain_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let face = s.faces[r.integer() as usize].clone();
        let mut d = BRepGPropDomain::with_face(&face)?;
        while d.more() {
            let e = d.value()?;
            int(index_of(&s.edges, &e));
            int(e.orientation() as i32);
            d.next()?;
        }
        show(d.value(), |e| int(index_of(&s.edges, e.as_ref())));
        d.init()?;
        flag(d.more());
        if d.more() {
            int(index_of(&s.edges, d.value()?.as_ref()));
        }
        let empty = BRepGPropDomain::new();
        flag(empty.more());
        show(empty.value(), |e| flag(e.is_null()));
        let mut later = BRepGPropDomain::new();
        later.init_face(&face)?;
        flag(later.more());
        Ok(())
    });
    end();
}

/// `shape face type overload x y z eps bypoint n coeffs...`: one Gauss Compute on
/// prefilled outputs.
pub fn gauss_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let face = s.faces[r.integer() as usize].clone();
        let kind = if r.integer() == 0 {
            GaussType::Vinert
        } else {
            GaussType::Sinert
        };
        let mut g = BRepGPropGauss::new(kind);
        let (overload, loc, eps, by_point) = (r.integer(), r.pnt(), r.real(), r.flag());
        let coeff = r.reals();
        let mut f = BRepGPropFace::with_face(&face, false)?;
        let mut d = BRepGPropDomain::with_face(&face)?;
        let mut out = GaussProps {
            mass: -7.,
            gravity_center: Point3::new(1., 2., 3.),
            inertia: Matrix3::from_columns(
                Xyz::new(5., 0., 0.),
                Xyz::new(0., 5., 0.),
                Xyz::new(0., 0., 5.),
            ),
        };
        attempt(|| match overload {
            0 => g.compute_natural(&f, loc, &coeff, by_point, &mut out),
            1 => g.compute_natural_surface(&f, loc, &mut out),
            2 => g.compute_domain(&mut f, &mut d, loc, &coeff, by_point, &mut out),
            3 => g.compute_domain_surface(&mut f, &mut d, loc, &mut out),
            4 => g
                .compute_adaptive(&mut f, &mut d, loc, eps, &coeff, by_point, &mut out)
                .map(real),
            _ => g
                .compute_adaptive_surface(&mut f, &mut d, loc, eps, &mut out)
                .map(real),
        });
        real(out.mass);
        pnt(out.gravity_center);
        for i in 1..=3 {
            for j in 1..=3 {
                real(out.inertia.value(i, j)?);
            }
        }
        Ok(())
    });
    end();
}

/// The `GProp_ValueType` of its native index.
fn value_type(index: i32) -> ValueType {
    [
        ValueType::Mass,
        ValueType::CenterMassX,
        ValueType::CenterMassY,
        ValueType::CenterMassZ,
        ValueType::InertiaXX,
        ValueType::InertiaYY,
        ValueType::InertiaZZ,
        ValueType::InertiaXY,
        ValueType::InertiaXZ,
        ValueType::InertiaYZ,
    ]
    .get(index as usize)
    .copied()
    .unwrap_or(ValueType::Unknown)
}

/// `shape face edge span bypoint x y z n coeffs... type umin tol nu (x v)... nt (t)...`:
/// the U and T functions of the face with a boundary.
pub fn functions_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let face = s.faces[r.integer() as usize].clone();
        let (e, span, by_point) = (r.integer(), r.flag(), r.flag());
        let vertex = r.pnt();
        let coeff = r.reals();
        let kind = value_type(r.integer());
        let (umin, tol) = (r.real(), r.real());
        let mut f = BRepGPropFace::with_face(&face, span)?;
        if e < 0 {
            f.load_iso(false, IsoType::IsoU)?;
        } else {
            flag(f.load_edge(&s.edges[e as usize])?);
        }
        let mut u = BRepGPropUFunction::new(&f, vertex, by_point, &coeff);
        u.set_value_type(kind);
        for _ in 0..r.integer() {
            let x = r.real();
            u.set_v_param(r.real());
            evaluate(&mut u, x, |u| u.take_failure());
        }
        let mut t = BRepGPropTFunction::new(&f, vertex, by_point, &coeff, umin, tol);
        t.set_value_type(kind);
        t.set_nb_kronrod_points(4);
        t.init();
        t.set_tolerance(tol);
        for _ in 0..r.integer() {
            let x = r.real();
            evaluate(&mut t, x, |t| t.take_failure());
        }
        real(t.error_reached());
        real(t.absolut_error());
        int(t.state_number());
        Ok(())
    });
    end();
}

/// One `Value(X, F)` of an integrand: its flag and value, or the kept exception.
fn evaluate<F: Function>(f: &mut F, x: f64, failure: impl FnOnce(&mut F) -> Option<Failure>) {
    let mut v = 0.;
    let ok = f.value(x, &mut v);
    if let Some(error) = failure(f) {
        print!("{} ", error.exception_type());
        return;
    }
    flag(ok);
    real(v);
}
