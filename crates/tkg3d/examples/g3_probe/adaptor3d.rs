//! The Adaptor3d probe operations of tools/oracle/tkg3d_probe_adaptor3d.cxx:
//! curve on surface, iso curve, InterFunc and HSurfaceTool.
use super::adaptor3d_inputs::{adaptor2d, curve_summary, inputs, surface};
use super::common::{Reader, attempt, ax1, ax3, dir, end, flag, int, name, pnt, real, show, vec};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool as tool;
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dInterFunc, Adaptor3dIsoCurve,
    Adaptor3dSurface,
};
use rustycad_tkmath::geom_abs::{IsoType, Shape};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

fn report<T>(built: Result<T, Failure>) -> Option<T> {
    match built {
        Ok(value) => Some(value),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

// Intervals into an array of a wrong length (a3_interval_arrays).
fn interval_arrays(a: &dyn Adaptor3dCurve) {
    let n = match a.nb_intervals(Shape::C2) {
        Ok(n) => n,
        Err(e) => return print!("{} ", e.exception_type()),
    };
    let mut long: Array1<'static, f64> = Array1::with_bounds(1, n + 2).unwrap();
    attempt(|| {
        a.intervals(&mut long, Shape::C2)?;
        real(*long.value(1).unwrap());
        Ok(())
    });
}

/// `a3_cos`: summary, interval arrays, Trim and ShallowCopy of a curve on a surface.
pub fn cos_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some((c, s)) = inputs(&mut r) else { return };
    let u = r.reals();
    let (t1, t2) = (r.real(), r.real());
    let Some(a) = report(Adaptor3dCurveOnSurface::with_curve(&c, &s)) else {
        return;
    };
    flag(a.get_curve() == c);
    flag(a.get_surface() == s);
    curve_summary(&a, &u);
    interval_arrays(&a);
    attempt(|| {
        curve_summary(&*a.trim(t1, t2, 1e-7)?, &u);
        Ok(())
    });
    attempt(|| {
        curve_summary(&*a.shallow_copy()?, &u);
        Ok(())
    });
    end();
}

/// `a3_cos_load`: the Load sequences of a curve on a surface.
pub fn cos_load_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some((c1, s1)) = inputs(&mut r) else {
        return;
    };
    let Some((c2, s2)) = inputs(&mut r) else {
        return;
    };
    let u = r.reals();
    let a = Adaptor3dCurveOnSurface::default();
    flag(a.get_curve().is_null());
    flag(a.get_surface().is_null());
    show(a.get_type(), |k| int(k as i32));
    attempt(|| {
        a.load_surface(&s1)?;
        int(a.get_type()? as i32);
        Ok(())
    });
    attempt(|| {
        a.load_curve(&c1)?;
        curve_summary(&a, &u);
        Ok(())
    });
    attempt(|| {
        a.load_surface(&s2)?;
        curve_summary(&a, &u);
        Ok(())
    });
    attempt(|| {
        a.load(&c2, &s1)?;
        curve_summary(&a, &u);
        Ok(())
    });
    show(a.nb_intervals(Shape::C1), int);
    a.change_curve(|curve| *curve = c1.clone());
    a.change_surface(|surface| *surface = s2.clone());
    flag(a.get_curve() == c1);
    curve_summary(&a, &u);
    attempt(|| {
        let b = Adaptor3dCurveOnSurface::with_surface(&s2)?;
        int(b.get_type()? as i32);
        b.load_curve(&c2)?;
        curve_summary(&b, &u);
        Ok(())
    });
    end();
}

fn iso_of(value: i32) -> IsoType {
    [IsoType::IsoU, IsoType::IsoV, IsoType::NoneIso][value as usize]
}

fn iso_summary(a: &Adaptor3dIsoCurve, u: &[f64]) {
    int(a.iso() as i32);
    real(a.parameter());
    flag(a.surface().is_null());
    curve_summary(a, u);
}

// The empty iso curve: the members that do not dereference the null surface.
fn empty_iso() {
    let e = Adaptor3dIsoCurve::default();
    int(e.iso() as i32);
    real(e.parameter());
    flag(e.surface().is_null());
    real(e.first_parameter().unwrap());
    real(e.last_parameter().unwrap());
    show(e.continuity(), |c| int(c as i32));
    show(e.nb_intervals(Shape::C1), int);
    show(e.is_closed(), flag);
    show(e.resolution(0.5), real);
    show(e.value(0.), pnt);
}

/// `a3_iso`: the constructors, Load members, Trim and ShallowCopy of an iso curve.
pub fn iso_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = report(surface(&mut r)) else {
        return;
    };
    let iso = iso_of(r.integer());
    let (p, w1, w2) = (r.real(), r.real(), r.real());
    let u = r.reals();
    let (t1, t2) = (r.real(), r.real());
    empty_iso();
    let b = Adaptor3dIsoCurve::with_surface(&s);
    iso_summary(&b, &u);
    attempt(|| {
        let a = Adaptor3dIsoCurve::with_iso(&s, iso, p)?;
        iso_summary(&a, &u);
        attempt(|| {
            curve_summary(&*a.trim(t1, t2, 1e-7)?, &u);
            Ok(())
        });
        attempt(|| {
            curve_summary(&*a.shallow_copy()?, &u);
            Ok(())
        });
        Ok(())
    });
    attempt(|| {
        iso_summary(
            &Adaptor3dIsoCurve::with_iso_range(&s, iso, p, (w1, w2))?,
            &u,
        );
        Ok(())
    });
    attempt(|| {
        b.load_iso_range(iso, p, (w1, w2))?;
        iso_summary(&b, &u);
        Ok(())
    });
    attempt(|| {
        b.load_iso(iso, p)?;
        iso_summary(&b, &u);
        Ok(())
    });
    b.load_surface(&s);
    iso_summary(&b, &u);
    end();
}

/// `a3_interfunc`: Value, Derivative and Values of Adaptor3d_InterFunc.
pub fn interfunc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(c) = report(adaptor2d(&mut r)) else {
        return;
    };
    let (v, fix) = (r.real(), r.integer());
    let x = r.reals();
    match Adaptor3dInterFunc::new(&c, v, fix) {
        Ok(mut f) => {
            for t in x {
                interfunc_at(&mut f, t);
            }
        }
        Err(e) => print!("{} ", e.exception_type()),
    }
    end();
}

// One parameter: a failed evaluation prints the kept exception, as the native throw.
fn interfunc_at(f: &mut Adaptor3dInterFunc, t: f64) {
    let (mut a, mut b) = (0., 0.);
    let ok = f.value(t, &mut a);
    print_or_failure(f, || {
        flag(ok);
        real(a);
    });
    let ok = f.derivative(t, &mut b);
    print_or_failure(f, || {
        flag(ok);
        real(b);
    });
    (a, b) = (0., 0.);
    let ok = f.values(t, &mut a, &mut b);
    print_or_failure(f, || {
        flag(ok);
        real(a);
        real(b);
    });
}

fn print_or_failure(f: &mut Adaptor3dInterFunc, print: impl FnOnce()) {
    match f.take_failure() {
        Some(failure) => print!("{} ", failure.exception_type()),
        None => print(),
    }
}

// Every continuity: NbU/VIntervals and U/VIntervals.
fn tool_intervals(s: &Handle<dyn Adaptor3dSurface>) {
    for k in SHAPES {
        attempt(|| {
            let n = tool::nb_u_intervals(s, k)?;
            int(n);
            let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).unwrap();
            tool::u_intervals(s, &mut t, k)?;
            t.as_slice().iter().for_each(|v| real(*v));
            Ok(())
        });
        attempt(|| {
            let n = tool::nb_v_intervals(s, k)?;
            int(n);
            let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).unwrap();
            tool::v_intervals(s, &mut t, k)?;
            t.as_slice().iter().for_each(|v| real(*v));
            Ok(())
        });
    }
}

fn tool_evaluations(s: &Handle<dyn Adaptor3dSurface>, u: f64, v: f64) {
    show(tool::value(s, u, v), pnt);
    show(tool::d0(s, u, v), pnt);
    show(tool::d1(s, u, v), |r| {
        pnt(r.point);
        vec(r.d1u);
        vec(r.d1v);
    });
    show(tool::d2(s, u, v), |r| {
        pnt(r.point);
        [r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv]
            .into_iter()
            .for_each(vec);
    });
    show(tool::d3(s, u, v), |r| {
        pnt(r.point);
        [
            r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv, r.d3u, r.d3v, r.d3uuv, r.d3uvv,
        ]
        .into_iter()
        .for_each(vec);
    });
    show(tool::dn(s, u, v, 1, 1), vec);
}

fn tool_kinds(s: &Handle<dyn Adaptor3dSurface>) {
    show(tool::get_type(s), |k| int(k as i32));
    show(tool::plane(s), |p| ax3(p.position()));
    show(tool::cylinder(s), |c| {
        ax3(c.position());
        real(c.radius());
    });
    show(tool::cone(s), |c| {
        ax3(c.position());
        real(c.ref_radius());
        real(c.semi_angle());
    });
    show(tool::torus(s), |c| {
        ax3(c.position());
        real(c.major_radius());
        real(c.minor_radius());
    });
    show(tool::sphere(s), |c| {
        ax3(c.position());
        real(c.radius());
    });
    show(tool::bezier(s), |b| name(&b));
    show(tool::bspline(s), |b| name(&b));
    show(tool::axe_of_revolution(s), ax1);
    show(tool::direction(s), dir);
    show(tool::basis_curve(s), |c| name(&c));
    show(tool::basis_surface(s), |c| name(&c));
    show(tool::offset_value(s), real);
}

/// `a3_hstool`: every Adaptor3d_HSurfaceTool function.
pub fn hstool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(s) = report(surface(&mut r)) else {
        return;
    };
    let (uv, ranges) = (r.reals(), r.reals());
    show(tool::first_u_parameter(&s), real);
    show(tool::last_u_parameter(&s), real);
    show(tool::first_v_parameter(&s), real);
    show(tool::last_v_parameter(&s), real);
    tool_intervals(&s);
    show(
        tool::u_trim(&s, 0., 0.5, 1e-7).and_then(|t| t.last_u_parameter()),
        real,
    );
    show(
        tool::v_trim(&s, 0., 0.5, 1e-7).and_then(|t| t.last_v_parameter()),
        real,
    );
    show(tool::is_u_closed(&s), flag);
    show(tool::is_v_closed(&s), flag);
    show(tool::is_u_periodic(&s), flag);
    show(tool::u_period(&s), real);
    show(tool::is_v_periodic(&s), flag);
    show(tool::v_period(&s), real);
    for pair in uv.as_chunks::<2>().0 {
        tool_evaluations(&s, pair[0], pair[1]);
    }
    for x in [1e-7, 0.5] {
        attempt(|| {
            real(tool::u_resolution(&s, x)?);
            real(tool::v_resolution(&s, x)?);
            Ok(())
        });
    }
    tool_kinds(&s);
    show(tool::nb_samples_u(&s), int);
    show(tool::nb_samples_v(&s), int);
    for pair in ranges.as_chunks::<2>().0 {
        show(tool::nb_samples_u_range(&s, pair[0], pair[1]), int);
        show(tool::nb_samples_v_range(&s, pair[0], pair[1]), int);
    }
    for t in [precision::angular(), 1e-3, 0.5] {
        show(tool::is_surf_g1(&s, true, t), flag);
        show(tool::is_surf_g1(&s, false, t), flag);
    }
    end();
}
