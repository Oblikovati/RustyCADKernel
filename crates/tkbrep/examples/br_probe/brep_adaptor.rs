//! BRepAdaptor and BRepLProp over a scene; the Rust twin of
//! tools/oracle/tkbrep_probe_brepadaptor.cxx.
use super::brep::scene;
use super::common::{Reader, attempt, end, flag, int, real};
use super::graph::{Graph, desc};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{
    BRepAdaptorCompCurve, BRepAdaptorCurve, BRepAdaptorCurve2d, BRepAdaptorSurface,
};
use rustycad_tkbrep::brep_lprop::{self, BRepLPropCLProps, BRepLPropSLProps, BRepLPropSurfaceTool};
use rustycad_tkbrep::brep_tools::WireExplorer;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Wire};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::{Point3, Vector3};

fn pnt(p: Point3) {
    [p.x(), p.y(), p.z()].into_iter().for_each(real);
}
fn vec3(v: Vector3) {
    [v.x(), v.y(), v.z()].into_iter().for_each(real);
}
fn xy(x: f64, y: f64) {
    real(x);
    real(y);
}

/// `ba_samples`.
fn samples(f: f64, l: f64) -> Vec<f64> {
    if precision::is_infinite(f) || precision::is_infinite(l) {
        return Vec::new();
    }
    (0..=4).map(|i| f + (l - f) * f64::from(i) / 4.0).collect()
}

const SHAPES: [Continuity; 5] = [
    Continuity::C0,
    Continuity::G1,
    Continuity::C1,
    Continuity::G2,
    Continuity::C2,
];

/// `ba_curve3d`: the members shared by every 3D curve adaptor.
fn curve3d(c: &dyn Adaptor3dCurve) {
    let (mut f, mut l) = (0.0, 0.0);
    attempt(|| {
        (f, l) = (c.first_parameter()?, c.last_parameter()?);
        real(f);
        real(l);
        Ok(())
    });
    attempt(|| c.get_type().map(|t| int(t as i32)));
    attempt(|| c.continuity().map(|t| int(t as i32)));
    attempt(|| c.is_closed().map(flag));
    attempt(|| c.is_periodic().map(flag));
    attempt(|| c.period().map(real));
    attempt(|| c.resolution(1e-3).map(real));
    for s in SHAPES {
        attempt(|| c.nb_intervals(s).map(int));
    }
    attempt(|| {
        let n = c.nb_intervals(Continuity::C2)?;
        let mut t = Array1::with_bounds(1, n + 1).unwrap();
        c.intervals(&mut t, Continuity::C2)?;
        t.iter().copied().for_each(real);
        Ok(())
    });
    curve3d_kind(c);
    for u in samples(f, l) {
        attempt(|| c.value(u).map(pnt));
        attempt(|| {
            c.d1(u).map(|(p, v)| {
                pnt(p);
                vec3(v);
            })
        });
        attempt(|| c.d2(u).map(|(_, _, v2)| vec3(v2)));
        attempt(|| c.dn(u, 3).map(vec3));
    }
    curve3d_copies(c, &samples(f, l));
}

/// The degree, poles and conic accessors of a 3D curve adaptor.
fn curve3d_kind(c: &dyn Adaptor3dCurve) {
    attempt(|| c.degree().map(int));
    attempt(|| c.is_rational().map(flag));
    attempt(|| c.nb_poles().map(int));
    attempt(|| c.nb_knots().map(int));
    attempt(|| c.line().map(|x| pnt(x.location())));
    attempt(|| c.circle().map(|x| real(x.radius())));
    attempt(|| c.ellipse().map(|x| real(x.major_radius())));
    attempt(|| c.hyperbola().map(|x| real(x.major_radius())));
    attempt(|| c.parabola().map(|x| real(x.focal())));
    attempt(|| {
        let kind = c.get_type()?;
        if kind == CurveType::BezierCurve {
            int(c.bezier()?.get().unwrap().nb_poles());
        }
        if kind == CurveType::BSplineCurve {
            int(c.bspline()?.get().unwrap().nb_poles());
        }
        Ok(())
    });
}

/// The trimmed and shallow copies at the samples `s`.
fn curve3d_copies(c: &dyn Adaptor3dCurve, s: &[f64]) {
    if s.is_empty() {
        return;
    }
    attempt(|| {
        let t = c.trim(s[1], s[3], 1e-7)?;
        let t = t.get().unwrap();
        real(t.first_parameter()?);
        real(t.last_parameter()?);
        pnt(t.value(s[2])?);
        Ok(())
    });
    attempt(|| {
        let t = c.shallow_copy()?;
        let t = t.get().unwrap();
        real(t.first_parameter()?);
        pnt(t.value(s[2])?);
        Ok(())
    });
}

/// `ba_curve2d`.
fn curve2d(c: &dyn Adaptor2dCurve2d) {
    let (mut f, mut l) = (0.0, 0.0);
    attempt(|| {
        (f, l) = (c.first_parameter()?, c.last_parameter()?);
        real(f);
        real(l);
        Ok(())
    });
    attempt(|| c.get_type().map(|t| int(t as i32)));
    attempt(|| c.continuity().map(|t| int(t as i32)));
    attempt(|| c.is_closed().map(flag));
    attempt(|| c.is_periodic().map(flag));
    attempt(|| c.resolution(1e-3).map(real));
    attempt(|| c.nb_intervals(Continuity::C1).map(int));
    attempt(|| c.nb_samples().map(int));
    let s = samples(f, l);
    for &u in &s {
        attempt(|| c.value(u).map(|p| xy(p.x(), p.y())));
        attempt(|| c.d1(u).map(|(_, v)| xy(v.x(), v.y())));
    }
    if s.is_empty() {
        return;
    }
    attempt(|| {
        let t = c.trim(s[1], s[3], 1e-7)?;
        let t = t.get().unwrap();
        real(t.first_parameter()?);
        let p = t.value(s[2])?;
        xy(p.x(), p.y());
        Ok(())
    });
    attempt(|| {
        c.shallow_copy()
            .and_then(|t| t.get().unwrap().last_parameter())
            .map(real)
    });
}

/// `ba_clprops`.
fn clprops(c: &BRepAdaptorCurve) {
    let (mut f, mut l) = (0.0, 0.0);
    attempt(|| {
        (f, l) = (c.first_parameter()?, c.last_parameter()?);
        Ok(())
    });
    for u in samples(f, l) {
        attempt(|| {
            let mut p = BRepLPropCLProps::with_parameter(c, u, 2, 1e-7)?;
            pnt(p.value());
            let t = p.is_tangent_defined()?;
            flag(t);
            if t {
                vec3(Vector3::from(p.tangent()?));
            }
            attempt(|| p.curvature().map(real));
            attempt(|| p.normal().map(|n| vec3(Vector3::from(n))));
            attempt(|| p.centre_of_curvature().map(pnt));
            Ok(())
        });
    }
}

fn edge(g: &Graph, e: &Edge) {
    attempt(|| {
        let c = BRepAdaptorCurve::with_edge(e)?;
        real(c.tolerance()?);
        flag(c.transformed_curve().is_3d_curve());
        flag(c.transformed_curve().is_curve_on_surface());
        desc(g, &c.edge());
        curve3d(&c);
        clprops(&c);
        Ok(())
    });
}

fn edge_on_face(g: &Graph, e: &Edge, f: &Face) {
    attempt(|| {
        let c = BRepAdaptorCurve::with_edge_on_face(e, f)?;
        flag(c.transformed_curve().is_3d_curve());
        flag(c.transformed_curve().is_curve_on_surface());
        curve3d(&c);
        Ok(())
    });
    attempt(|| {
        let c = BRepAdaptorCurve2d::with_edge_on_face(e, f)?;
        desc(g, &c.edge());
        desc(g, &c.face());
        curve2d(&c);
        Ok(())
    });
}

/// `ba_surface_props`.
fn surface_props(s: &BRepAdaptorSurface, u: f64, v: f64) {
    attempt(|| {
        let mut p = BRepLPropSLProps::with_parameters(s, u, v, 2, 1e-7)?;
        pnt(p.value());
        let n = p.is_normal_defined()?;
        flag(n);
        if n {
            vec3(Vector3::from(p.normal()?));
        }
        let k = p.is_curvature_defined()?;
        flag(k);
        if k {
            real(p.max_curvature()?);
            real(p.min_curvature()?);
            real(p.mean_curvature()?);
            real(p.gaussian_curvature()?);
            flag(p.is_umbilic()?);
        }
        Ok(())
    });
    attempt(|| BRepLPropSurfaceTool::value(s, u, v).map(pnt));
    attempt(|| BRepLPropSurfaceTool::d2(s, u, v).map(|d| vec3(d.d2uv)));
}

/// `ba_surface`.
fn surface(s: &BRepAdaptorSurface) {
    let (mut u1, mut u2, mut v1, mut v2) = (0.0, 0.0, 0.0, 0.0);
    attempt(|| {
        u1 = s.first_u_parameter()?;
        u2 = s.last_u_parameter()?;
        v1 = s.first_v_parameter()?;
        v2 = s.last_v_parameter()?;
        [u1, u2, v1, v2].into_iter().for_each(real);
        Ok(())
    });
    surface_flags(s);
    let (us, vs) = (samples(u1, u2), samples(v1, v2));
    for (&u, &v) in us.iter().zip(vs.iter()) {
        attempt(|| {
            let d = s.d1(u, v)?;
            pnt(d.point);
            vec3(d.d1u);
            vec3(d.d1v);
            Ok(())
        });
        attempt(|| s.dn(u, v, 1, 1).map(vec3));
        surface_props(s, u, v);
    }
}

/// The type, continuity, closure, resolution and tool queries of `ba_surface`.
fn surface_flags(s: &BRepAdaptorSurface) {
    attempt(|| s.get_type().map(|t| int(t as i32)));
    attempt(|| s.u_continuity().map(|t| int(t as i32)));
    attempt(|| s.v_continuity().map(|t| int(t as i32)));
    attempt(|| s.is_u_closed().map(flag));
    attempt(|| s.is_v_closed().map(flag));
    attempt(|| s.is_u_periodic().map(flag));
    attempt(|| s.is_v_periodic().map(flag));
    attempt(|| s.nb_u_intervals(Continuity::C1).map(int));
    attempt(|| s.nb_v_intervals(Continuity::C1).map(int));
    attempt(|| s.u_resolution(1e-3).map(real));
    attempt(|| s.v_resolution(1e-3).map(real));
    attempt(|| s.tolerance().map(real));
    attempt(|| s.cylinder().map(|c| real(c.radius())));
    attempt(|| BRepLPropSurfaceTool::continuity(s).map(int));
    attempt(|| {
        let (a, b, c, d) = BRepLPropSurfaceTool::bounds(s)?;
        [a, b, c, d].into_iter().for_each(real);
        Ok(())
    });
}

fn face(g: &Graph, f: &Face) {
    for restriction in [true, false] {
        attempt(|| {
            let s = BRepAdaptorSurface::with_face(f, restriction)?;
            desc(g, &s.face());
            surface(&s);
            Ok(())
        });
    }
    for e in Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        edge_on_face(g, &topods::edge(&e).unwrap(), f);
    }
}

/// `ba_comp`.
fn comp(g: &Graph, c: &BRepAdaptorCompCurve) {
    desc(g, &c.wire());
    curve3d(c);
    let (mut f, mut l) = (0.0, 0.0);
    attempt(|| {
        (f, l) = (c.first_parameter()?, c.last_parameter()?);
        Ok(())
    });
    for u in samples(f, l) {
        attempt(|| {
            let found = c.edge(u)?;
            desc(g, &found.edge);
            real(found.parameter);
            Ok(())
        });
    }
}

/// `ba_junctions`.
fn junctions(w: &Wire) -> Result<(), Failure> {
    let mut edges = Vec::new();
    let mut x = WireExplorer::with_wire(w)?;
    while x.more() {
        edges.push(x.current().clone());
        x.next()?;
    }
    for pair in edges.windows(2) {
        attempt(|| {
            let (a, b) = (
                BRepAdaptorCurve::with_edge(&pair[0])?,
                BRepAdaptorCurve::with_edge(&pair[1])?,
            );
            let u1 = if pair[0].orientation() == Orientation::Reversed {
                a.first_parameter()?
            } else {
                a.last_parameter()?
            };
            let u2 = if pair[1].orientation() == Orientation::Reversed {
                b.last_parameter()?
            } else {
                b.first_parameter()?
            };
            attempt(|| brep_lprop::continuity(&a, &b, u1, u2).map(|c| int(c as i32)));
            attempt(|| {
                brep_lprop::continuity_with_tolerances(&a, &b, u1, u2, 1e-7, 1e-3)
                    .map(|c| int(c as i32))
            });
            Ok(())
        });
    }
    Ok(())
}

fn wire(g: &Graph, w: &Wire) {
    for ac in [false, true] {
        attempt(|| BRepAdaptorCompCurve::with_wire(w, ac).map(|c| comp(g, &c)));
    }
    attempt(|| {
        let c = BRepAdaptorCompCurve::with_wire(w, false)?;
        let (f, l) = (c.first_parameter()?, c.last_parameter()?);
        if !precision::is_infinite(f) && !precision::is_infinite(l) {
            let t = BRepAdaptorCompCurve::with_range(
                w,
                false,
                f + (l - f) / 8.0,
                l - (l - f) / 8.0,
                1e-7,
            )?;
            comp(g, &t);
        }
        Ok(())
    });
    attempt(|| junctions(w));
}

/// `br_adaptor script`.
pub fn adaptor_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let g = &s.graph;
    for sh in g.shapes.iter().filter(|sh| !sh.is_null()) {
        match sh.shape_type().unwrap() {
            ShapeEnum::Edge => {
                let e = topods::edge(sh).unwrap();
                if BRepTool::is_geometric_edge(&e).unwrap() {
                    edge(g, &e);
                }
            }
            ShapeEnum::Face => {
                let f = topods::face(sh).unwrap();
                if BRepTool::is_geometric_face(&f).unwrap() {
                    face(g, &f);
                }
            }
            ShapeEnum::Wire => wire(g, &topods::wire(sh).unwrap()),
            _ => {}
        }
    }
    desc(g, &BRepAdaptorCurve::default().edge());
    end();
}
