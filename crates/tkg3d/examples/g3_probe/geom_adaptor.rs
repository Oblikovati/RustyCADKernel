//! Adaptor3d and GeomAdaptor summaries and MakeCurve/MakeSurface; the counterpart of
//! the summary functions of tools/oracle/tkg3d_probe_geom_adaptor.cxx (a3_*_summary...).
use super::common::{
    attempt, ax1, ax2, ax3, curve_summary, dir, flag, int, name, pnt, real, show, surface_summary,
    vec,
};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom_adaptor::{make_curve, make_surface};
use rustycad_tkmath::geom_abs::Shape;

pub const SHAPES: [Shape; 7] = [
    Shape::C0,
    Shape::G1,
    Shape::C1,
    Shape::G2,
    Shape::C2,
    Shape::C3,
    Shape::CN,
];
const RESOLUTIONS: [f64; 4] = [1e-7, 1e-3, 0.5, 10.0];

/// a3_curve_intervals.
pub fn curve_intervals(a: &dyn Adaptor3dCurve) {
    for shape in SHAPES {
        let n = match a.nb_intervals(shape) {
            Ok(n) => n,
            Err(error) => {
                print!("{} ", error.exception_type());
                continue;
            }
        };
        int(n);
        let mut t = Array1::<f64>::with_bounds(1, n + 1).unwrap();
        attempt(|| {
            a.intervals(&mut t, shape)?;
            t.iter().for_each(|v| real(*v));
            Ok(())
        });
    }
}

fn named<T: ?Sized + rustycad_tkernel::standard::RuntimeTyped>(
    handle: Result<Handle<T>, Failure>,
    tail: impl FnOnce(&T),
) {
    show(handle, |h| {
        name(&h);
        if let Some(object) = h.get() {
            tail(object);
        }
    });
}

/// a3_curve_kinds.
pub fn curve_kinds(a: &dyn Adaptor3dCurve) {
    show(a.get_type(), |t| int(t as i32));
    show(a.line(), |l| ax1(l.position()));
    show(a.circle(), |c| {
        ax2(c.position());
        real(c.radius());
    });
    show(a.ellipse(), |e| {
        ax2(e.position());
        real(e.major_radius());
        real(e.minor_radius());
    });
    show(a.hyperbola(), |h| {
        ax2(h.position());
        real(h.major_radius());
        real(h.minor_radius());
    });
    show(a.parabola(), |p| {
        ax2(p.position());
        real(p.focal());
    });
    show(a.degree(), int);
    show(a.is_rational(), flag);
    show(a.nb_poles(), int);
    show(a.nb_knots(), int);
    named(a.bezier(), |b| int(b.nb_poles()));
    named(a.bspline(), |b| int(b.nb_poles()));
    named(a.offset_curve(), |o| real(o.offset()));
}

fn curve_evaluations(a: &dyn Adaptor3dCurve, u: f64) {
    show(a.eval_d0(u), pnt);
    show(a.eval_d1(u), |e| {
        pnt(e.point);
        vec(e.d1);
    });
    show(a.eval_d2(u), |e| {
        pnt(e.point);
        [e.d1, e.d2].into_iter().for_each(vec);
    });
    show(a.eval_d3(u), |e| {
        pnt(e.point);
        [e.d1, e.d2, e.d3].into_iter().for_each(vec);
    });
    for n in 0..=5 {
        show(a.eval_dn(u, n), vec);
    }
    show(a.value(u), pnt);
    show(a.d0(u), pnt);
    show(a.d1(u), |(p, v1)| {
        pnt(p);
        vec(v1);
    });
    show(a.d2(u), |(p, v1, v2)| {
        pnt(p);
        [v1, v2].into_iter().for_each(vec);
    });
    show(a.d3(u), |(p, v1, v2, v3)| {
        pnt(p);
        [v1, v2, v3].into_iter().for_each(vec);
    });
    show(a.dn(u, 2), vec);
}

/// a3_curve_summary.
pub fn curve_adaptor_summary(a: &Handle<dyn Adaptor3dCurve>, params: &[f64]) {
    name(a);
    show(a.first_parameter(), real);
    show(a.last_parameter(), real);
    show(a.continuity(), |s| int(s as i32));
    curve_intervals(&**a);
    show(a.is_closed(), flag);
    show(a.is_periodic(), flag);
    show(a.period(), real);
    for r in RESOLUTIONS {
        show(a.resolution(r), real);
    }
    curve_kinds(&**a);
    for &u in params {
        curve_evaluations(&**a, u);
    }
}

/// a3_make_curve.
pub fn make_curve_summary(a: &dyn Adaptor3dCurve, params: &[f64]) {
    show(make_curve(a), |c| {
        name(&c);
        if let Some(curve) = c.get() {
            curve_summary(curve, params);
        }
    });
}

fn one_direction_intervals(
    count: Result<i32, Failure>,
    fill: impl FnOnce(&mut Array1<'_, f64>) -> Result<(), Failure>,
) {
    show(count, |n| {
        int(n);
        let mut t = Array1::<f64>::with_bounds(1, n + 1).unwrap();
        attempt(|| {
            fill(&mut t)?;
            t.iter().for_each(|v| real(*v));
            Ok(())
        });
    });
}

/// a3_surface_intervals.
pub fn surface_intervals(a: &dyn Adaptor3dSurface) {
    for shape in SHAPES {
        one_direction_intervals(a.nb_u_intervals(shape), |t| a.u_intervals(t, shape));
        one_direction_intervals(a.nb_v_intervals(shape), |t| a.v_intervals(t, shape));
    }
}

fn elementary_kinds(a: &dyn Adaptor3dSurface) {
    show(a.get_type(), |t| int(t as i32));
    show(a.plane(), |p| ax3(p.position()));
    show(a.cylinder(), |c| {
        ax3(c.position());
        real(c.radius());
    });
    show(a.cone(), |c| {
        ax3(c.position());
        real(c.ref_radius());
        real(c.semi_angle());
    });
    show(a.sphere(), |s| {
        ax3(s.position());
        real(s.radius());
    });
    show(a.torus(), |t| {
        ax3(t.position());
        real(t.major_radius());
        real(t.minor_radius());
    });
}

/// a3_surface_kinds.
pub fn surface_kinds(a: &dyn Adaptor3dSurface) {
    elementary_kinds(a);
    for count in [a.u_degree(), a.nb_u_poles(), a.v_degree(), a.nb_v_poles()] {
        show(count, int);
    }
    show(a.nb_u_knots(), int);
    show(a.nb_v_knots(), int);
    show(a.is_u_rational(), flag);
    show(a.is_v_rational(), flag);
    named(a.bezier(), |b| int(b.nb_u_poles()));
    named(a.bspline(), |b| int(b.nb_u_poles()));
    show(a.axe_of_revolution(), ax1);
    show(a.direction(), dir);
    named(a.basis_curve(), |c| {
        attempt(|| {
            real(c.first_parameter()?);
            real(c.last_parameter()?);
            int(c.get_type()? as i32);
            Ok(())
        })
    });
    show(a.basis_surface(), |s| {
        name(&s);
        attempt(|| {
            real(s.first_u_parameter()?);
            real(s.last_u_parameter()?);
            real(s.first_v_parameter()?);
            real(s.last_v_parameter()?);
            int(s.get_type()? as i32);
            Ok(())
        })
    });
    show(a.offset_value(), real);
}

fn surface_evaluations(a: &dyn Adaptor3dSurface, u: f64, v: f64) {
    show(a.eval_d0(u, v), pnt);
    show(a.eval_d1(u, v), |r| {
        pnt(r.point);
        [r.d1u, r.d1v].into_iter().for_each(vec);
    });
    show(a.eval_d2(u, v), |r| {
        pnt(r.point);
        [r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv]
            .into_iter()
            .for_each(vec);
    });
    show(a.eval_d3(u, v), |r| {
        pnt(r.point);
        let d = [
            r.d1u, r.d1v, r.d2u, r.d2v, r.d2uv, r.d3u, r.d3v, r.d3uuv, r.d3uvv,
        ];
        d.into_iter().for_each(vec);
    });
    for nu in 0..=3 {
        for nv in 0..=3 {
            if nu + nv > 0 {
                show(a.eval_dn(u, v, nu, nv), vec);
            }
        }
    }
    surface_wrappers(a, u, v);
}

// The non-virtual Value, D0..D3 and DN.
fn surface_wrappers(a: &dyn Adaptor3dSurface, u: f64, v: f64) {
    show(a.value(u, v), pnt);
    show(a.d0(u, v), pnt);
    show(a.d1(u, v), |r| {
        pnt(r.point);
        [r.d1u, r.d1v].into_iter().for_each(vec);
    });
    show(a.d2(u, v), |r| {
        pnt(r.point);
        [r.d2u, r.d2v, r.d2uv].into_iter().for_each(vec);
    });
    show(a.d3(u, v), |r| {
        pnt(r.point);
        [r.d3u, r.d3v, r.d3uuv, r.d3uvv].into_iter().for_each(vec);
    });
    show(a.dn(u, v, 1, 1), vec);
}

/// a3_surface_summary.
pub fn surface_adaptor_summary(a: &Handle<dyn Adaptor3dSurface>, uv: &[f64]) {
    name(a);
    let bounds = [
        a.first_u_parameter(),
        a.last_u_parameter(),
        a.first_v_parameter(),
        a.last_v_parameter(),
    ];
    bounds.into_iter().for_each(|b| show(b, real));
    show(a.u_continuity(), |s| int(s as i32));
    show(a.v_continuity(), |s| int(s as i32));
    surface_intervals(&**a);
    for closure in [
        a.is_u_closed(),
        a.is_v_closed(),
        a.is_u_periodic(),
        a.is_v_periodic(),
    ] {
        show(closure, flag);
    }
    show(a.u_period(), real);
    show(a.v_period(), real);
    for r in RESOLUTIONS {
        show(a.u_resolution(r), real);
        show(a.v_resolution(r), real);
    }
    surface_kinds(&**a);
    for pair in uv.as_chunks::<2>().0 {
        surface_evaluations(&**a, pair[0], pair[1]);
    }
}

/// a3_make_surface.
pub fn make_surface_summary(a: &dyn Adaptor3dSurface, uv: &[f64]) {
    for trim in [true, false] {
        show(make_surface(a, trim), |s| {
            name(&s);
            if let Some(surface) = s.get() {
                surface_summary(surface, uv);
            }
        });
    }
}
