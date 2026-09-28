//! The LProp and GeomLProp probe; the counterpart of tools/oracle/tkgeombase_probe_lprop.cxx.
pub mod build;

use super::common::{self as p, Reader, attempt, end};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::geom_lprop::{self, CLPropsBase, CurAndInf2d, SLProps};
use rustycad_tkgeombase::lprop::CurAndInf;
use rustycad_tkgeombase::lprop::curve_access::{CurveAccess, DirectAccess, PropsVector};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Vector2, Vector3};

/// The overloaded gb_lp printers.
pub trait Out {
    fn out(self);
}
impl Out for Point3 {
    fn out(self) {
        p::pnt(self);
    }
}
impl Out for Point2 {
    fn out(self) {
        p::pnt2d(self);
    }
}
impl Out for Vector3 {
    fn out(self) {
        p::vec(self);
    }
}
impl Out for Vector2 {
    fn out(self) {
        p::vec2d(self);
    }
}
impl Out for Direction3 {
    fn out(self) {
        p::dir(self);
    }
}
impl Out for Direction2 {
    fn out(self) {
        p::dir2d(self);
    }
}

/// A printable Result, or the exception type (the body of GB_TRY).
fn out<T: Out>(value: Result<T, Failure>) {
    p::show(value, T::out);
}

type Props<V, C> = CLPropsBase<V, C, DirectAccess>;

/// gb_lp_clprops_members.
fn clprops_members<V, C>(props: &mut Props<V, C>, mode: i32)
where
    V: PropsVector + Out,
    V::Point: Out,
    V::Direction: Out,
    DirectAccess: CurveAccess<C, V>,
{
    if mode == 1 {
        out(props.centre_of_curvature());
        out(props.normal());
        p::show(props.curvature(), p::real);
        out(props.centre_of_curvature());
        out(props.tangent());
        out(props.d3());
        props.value().out();
        return;
    }
    props.value().out();
    p::show(props.is_tangent_defined(), p::flag);
    out(props.tangent());
    p::show(props.curvature(), p::real);
    out(props.normal());
    out(props.centre_of_curvature());
    out(props.d1());
    out(props.d2());
    out(props.d3());
    props.value().out();
}

/// gb_lp_clprops_run.
fn clprops_run<V, C>(curve: C, r: &mut Reader<'_>)
where
    V: PropsVector + Out,
    V::Point: Out,
    V::Direction: Out,
    C: Clone + Default,
    DirectAccess: CurveAccess<C, V>,
{
    let (mode, n, tol, np) = (r.integer(), r.integer(), r.real(), r.integer());
    let mut shared: Option<Props<V, C>> = None;
    if mode == 2 {
        match Props::<V, C>::new(curve.clone(), n, tol) {
            Ok(props) => shared = Some(props),
            Err(error) => {
                print!("{} ", error.exception_type());
                return end();
            }
        }
    }
    for _ in 0..np {
        let u = r.real();
        if let Some(props) = shared.as_mut() {
            attempt(|| props.set_parameter(u));
            p::show(props.curvature(), p::real);
            out(props.centre_of_curvature());
            out(props.tangent());
            out(props.d1());
            continue;
        }
        if mode == 3 {
            attempt(|| {
                let mut props = Props::<V, C>::without_curve(n, tol)?;
                p::flag(props.is_tangent_defined()?);
                out(props.tangent());
                props.set_curve(curve.clone());
                props.set_parameter(u)?;
                clprops_members(&mut props, 0);
                Ok(())
            });
            continue;
        }
        attempt(|| {
            let mut props = Props::<V, C>::with_parameter(curve.clone(), u, n, tol)?;
            clprops_members(&mut props, mode);
            Ok(())
        });
    }
    end();
}

/// Build a curve or print the construction exception and end the line.
fn built<T>(value: Result<T, Failure>) -> Option<T> {
    match value {
        Ok(value) => Some(value),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

pub fn cl2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(curve) = built(build::curve2d(&mut r)) else {
        return;
    };
    clprops_run::<Vector2, _>(curve, &mut r);
}

pub fn cl3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(curve) = built(build::curve3d(&mut r)) else {
        return;
    };
    clprops_run::<Vector3, _>(curve, &mut r);
}

pub fn cl3d_adaptor_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let adaptor = build::curve3d(&mut r).and_then(|c| GeomAdaptorCurve::with_curve(&c));
    let Some(adaptor) = built(adaptor) else {
        return;
    };
    let curve: Handle<dyn Adaptor3dCurve> = Handle::new(adaptor).cast();
    clprops_run::<Vector3, _>(curve, &mut r);
}

/// gb_lp_slprops_members.
fn slprops_members(props: &mut SLProps, mode: i32) {
    p::pnt(props.value());
    if mode == 1 {
        curvature_getters(props);
    }
    out(props.tangent_u());
    out(props.tangent_v());
    p::show(props.is_tangent_u_defined(), p::flag);
    p::show(props.is_tangent_v_defined(), p::flag);
    p::show(props.is_normal_defined(), p::flag);
    out(props.normal());
    p::show(props.is_curvature_defined(), p::flag);
    curvature_getters(props);
    p::show(props.curvature_directions(), |(max, min)| {
        p::dir(max);
        p::dir(min);
    });
    out(props.tangent_u());
    out(props.tangent_v());
    out(props.d1u());
    out(props.d1v());
    out(props.d2u());
    out(props.d2v());
    out(props.duv());
}

fn curvature_getters(props: &mut SLProps) {
    p::show(props.max_curvature(), p::real);
    p::show(props.min_curvature(), p::real);
    p::show(props.mean_curvature(), p::real);
    p::show(props.gaussian_curvature(), p::real);
    p::show(props.is_umbilic(), p::flag);
}

pub fn sl_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(surface) = built(build::surface(&mut r)) else {
        return;
    };
    let (mode, n, tol, np) = (r.integer(), r.integer(), r.real(), r.integer());
    let mut shared: Option<SLProps> = None;
    if mode == 2 {
        match SLProps::new(surface.clone(), n, tol) {
            Ok(props) => shared = Some(props),
            Err(error) => {
                print!("{} ", error.exception_type());
                return end();
            }
        }
    }
    for _ in 0..np {
        let (u, v) = (r.real(), r.real());
        if let Some(props) = shared.as_mut() {
            attempt(|| {
                props.set_parameters(u, v)?;
                slprops_members(props, 1);
                Ok(())
            });
            continue;
        }
        if mode == 3 {
            attempt(|| {
                let mut props = SLProps::without_surface(n, tol)?;
                p::flag(props.is_normal_defined()?);
                p::flag(props.is_curvature_defined()?);
                props.set_surface(surface.clone());
                props.set_parameters(u, v)?;
                slprops_members(&mut props, 0);
                Ok(())
            });
            continue;
        }
        attempt(|| {
            let mut props = SLProps::with_parameters(surface.clone(), u, v, n, tol)?;
            slprops_members(&mut props, mode);
            Ok(())
        });
    }
    end();
}

/// gb_lp_points.
fn points(points: &CurAndInf) {
    p::flag(points.is_empty());
    p::int(points.nb_points());
    for i in 1..=points.nb_points() {
        p::real(points.parameter(i).unwrap());
        p::int(points.type_of(i).unwrap() as i32);
    }
}

pub fn curinf2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Some(curve) = built(build::curve2d(&mut r)) else {
        return;
    };
    let mode = r.integer();
    let mut analyzer = CurAndInf2d::new();
    p::flag(analyzer.is_done());
    attempt(|| {
        match mode {
            0 => analyzer.perform(&curve)?,
            1 => analyzer.perform_cur_ext(&curve)?,
            2 => analyzer.perform_inf(&curve)?,
            _ => {
                analyzer.perform_cur_ext(&curve)?;
                p::flag(analyzer.is_done());
                points(&analyzer);
                analyzer.perform_inf(&curve)?;
            }
        }
        Ok(())
    });
    p::flag(analyzer.is_done());
    points(&analyzer);
    end();
}

pub fn curandinf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut store = CurAndInf::new();
    for _ in 0..r.integer() {
        let (op, u) = (r.integer(), r.real());
        match op {
            0 => store.add_inflection(u),
            3 => store.clear(),
            _ => store.add_ext_cur(u, op == 1),
        }
    }
    points(&store);
    for _ in 0..r.integer() {
        let q = r.integer();
        p::show(store.parameter(q), p::real);
        p::show(store.type_of(q), |kind| p::int(kind as i32));
    }
    end();
}

pub fn continuity_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let curves = build::curve3d(&mut r).and_then(|c1| Ok((c1, build::curve3d(&mut r)?)));
    let Some((c1, c2)) = built(curves) else {
        return;
    };
    let (u1, u2, r1, r2, with_tol) = (r.real(), r.real(), r.flag(), r.flag(), r.flag());
    let (tl, ta) = (r.real(), r.real());
    let shape = if with_tol {
        geom_lprop::continuity_with_tolerances(&c1, &c2, u1, u2, r1, r2, tl, ta)
    } else {
        geom_lprop::continuity(&c1, &c2, u1, u2, r1, r2)
    };
    p::show(shape, |shape| p::int(shape as i32));
    end();
}
