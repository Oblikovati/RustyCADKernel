//! GeomFill trihedron law probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_geomfill_trihedron.cxx.
use super::common::{Reader, end, flag, int, pnt, real, show, xyz};
use super::curve2d::curve2d;
use super::ics_geometry::{curve, surface};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface};
use rustycad_tkgeomalgo::geom_fill::*;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Vector3;

pub const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];

/// `ga_gft_curve`: `0 curve` or `1 curve2d surface`.
pub fn gft_curve(r: &mut Reader) -> Handle<dyn Adaptor3dCurve> {
    if r.integer() == 0 {
        return curve(r);
    }
    let c = curve2d(r).expect("probe 2d curve");
    let s = surface(r);
    let c2d: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_curve(&c).expect("probe adaptor")).cast();
    Handle::new(Adaptor3dCurveOnSurface::with_curve(&c2d, &s).expect("probe curve on surface"))
        .cast()
}

/// `ga_gft_law`.
pub fn gft_law(r: &mut Reader) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
    Ok(match r.integer() {
        0 => {
            let t = Vector3::from(r.xyz());
            let n = Vector3::from(r.xyz());
            Handle::new(GeomFillFixed::new(t, n)?).cast()
        }
        1 => Handle::new(GeomFillFrenet::new()).cast(),
        2 => match r.integer() {
            2 => Handle::new(GeomFillCorrectedFrenet::new()).cast(),
            m => Handle::new(GeomFillCorrectedFrenet::with_evaluation(m != 0)).cast(),
        },
        3 => Handle::new(GeomFillConstantBiNormal::new(r.dir())).cast(),
        4 => Handle::new(GeomFillDarboux::new()).cast(),
        5 => Handle::new(GeomFillDiscreteTrihedron::new()).cast(),
        6 => {
            let b = Vector3::from(r.xyz());
            Handle::new(GeomFillDraftTrihedron::new(b, r.real())?).cast()
        }
        7 => Handle::new(GeomFillGuideTrihedronAC::new(&gft_curve(r))?).cast(),
        _ => Handle::new(GeomFillGuideTrihedronPlan::new(&gft_curve(r))?).cast(),
    })
}

pub fn vec(v: Vector3) {
    xyz(*v.xyz());
}

/// `ga_gft_eval`.
pub fn gft_eval(law: &dyn GeomFillTrihedronLaw, u: f64) {
    let s = Vector3::new(-1.0, -1.0, -1.0);
    let mut f0 = TrihedronD0 {
        tangent: s,
        normal: s,
        binormal: s,
    };
    show(law.d0(u, &mut f0), flag);
    [f0.tangent, f0.normal, f0.binormal]
        .into_iter()
        .for_each(vec);
    let mut f1 = TrihedronD1 {
        tangent: s,
        d_tangent: s,
        normal: s,
        d_normal: s,
        binormal: s,
        d_binormal: s,
    };
    show(law.d1(u, &mut f1), flag);
    [
        f1.tangent,
        f1.d_tangent,
        f1.normal,
        f1.d_normal,
        f1.binormal,
        f1.d_binormal,
    ]
    .into_iter()
    .for_each(vec);
    let mut f2 = TrihedronD2 {
        tangent: s,
        d_tangent: s,
        d2_tangent: s,
        normal: s,
        d_normal: s,
        d2_normal: s,
        binormal: s,
        d_binormal: s,
        d2_binormal: s,
    };
    show(law.d2(u, &mut f2), flag);
    [
        f2.tangent,
        f2.d_tangent,
        f2.d2_tangent,
        f2.normal,
        f2.d_normal,
        f2.d2_normal,
        f2.binormal,
        f2.d_binormal,
        f2.d2_binormal,
    ]
    .into_iter()
    .for_each(vec);
}

fn queries(law: &dyn GeomFillTrihedronLaw) {
    int(law.error_status() as i32);
    show(law.is_constant(), flag);
    show(law.is_only_by_3d_curve(), flag);
    for shape in SHAPES {
        show(law.nb_intervals(shape), |n| {
            int(n);
            let mut t: Array1<'static, f64> = Array1::with_bounds(1, n + 1).expect("probe array");
            t.init(-1.0);
            show(law.intervals(&mut t, shape), |()| ());
            t.as_slice().iter().for_each(|v| real(*v));
        });
    }
    show(law.get_average_law(), |f| {
        vec(f.tangent);
        vec(f.normal);
        vec(f.binormal);
    });
}

fn guide(law: &Handle<dyn GeomFillTrihedronLaw>, o1: f64, o2: f64) {
    let g: Handle<dyn GeomFillTrihedronWithGuide> = law.cast();
    let Some(g) = g.get() else { return };
    let c = g.guide();
    flag(c.is_null());
    if let Some(c) = c.get() {
        real(c.first_parameter().expect("probe guide"));
        real(c.last_parameter().expect("probe guide"));
    }
    pnt(g.current_point_on_guide());
    show(g.origine(o1, o2), |()| ());
}

/// `law curve nparams params... a b o1 o2`.
pub fn law_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let law = match gft_law(&mut r) {
        Ok(law) => law,
        Err(error) => {
            print!("{} ", error.exception_type());
            return end();
        }
    };
    let l = law.get().expect("probe law");
    let c = gft_curve(&mut r);
    let params = r.reals();
    let (a, b, o1, o2) = (r.real(), r.real(), r.real(), r.real());
    show(l.set_curve(&c), flag);
    queries(l);
    params.iter().for_each(|u| gft_eval(l, *u));
    show(l.copy(), |copy| {
        if let Some(u) = params.first() {
            gft_eval(copy.get().expect("probe copy"), *u);
        }
    });
    show(l.set_interval(a, b), |()| ());
    if law.cast::<GeomFillGuideTrihedronPlan>().is_null() {
        show(l.get_interval(), |(f, last)| {
            real(f);
            real(last);
        });
    }
    params.iter().for_each(|u| gft_eval(l, *u));
    guide(&law, o1, o2);
    let cf: Handle<GeomFillCorrectedFrenet> = law.cast();
    if let Some(cf) = cf.get() {
        show(cf.evaluate_best_mode(), |m| int(m as i32));
    }
    end();
}
