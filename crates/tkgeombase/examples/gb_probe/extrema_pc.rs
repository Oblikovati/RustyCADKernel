//! The classic point-curve Extrema requests; the counterpart of
//! tools/oracle/tkgeombase_probe_extrema_pc_classic.cxx (same encoding and token order).
#![allow(clippy::unit_arg, clippy::type_complexity)]
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real};
use super::extrema_el::{poncurv, poncurv2d};
use super::pc2d::curve2d;
use super::pc3d::curve3d;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::extrema::{
    Curve2dTool, CurveTool, ExtremaCurveTool, GCurveLocator, GFuncExtPC, GGExtPC, GGenExtPC,
    GLocateExtPC, GenLocateExtPC, POnCurve,
};
use rustycad_tkmath::gp::{Point2, Point3};

/// The 3D or 2D side of a request family (GbSpace3 / GbSpace2).
pub trait Space {
    type Tool: ExtremaCurveTool<Curve: RuntimeTyped + Send + Sync>;
    fn adaptor(r: &mut Reader) -> Result<Handle<<Self::Tool as ExtremaCurveTool>::Curve>, Failure>;
    fn point(r: &mut Reader) -> <Self::Tool as ExtremaCurveTool>::Point;
    fn pnt(p: <Self::Tool as ExtremaCurveTool>::Point);
    fn pon(p: POnCurve<<Self::Tool as ExtremaCurveTool>::Point>);
}
pub struct Space3;
pub struct Space2;
impl Space for Space3 {
    type Tool = CurveTool;
    fn adaptor(r: &mut Reader) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let c = curve3d(r)?;
        if r.flag() {
            let (u1, u2) = (r.real(), r.real());
            return Ok(Handle::new(GeomAdaptorCurve::with_range(&c, u1, u2)?).cast());
        }
        Ok(Handle::new(GeomAdaptorCurve::with_curve(&c)?).cast())
    }
    fn point(r: &mut Reader) -> Point3 {
        r.pnt()
    }
    fn pnt(p: Point3) {
        pnt(p);
    }
    fn pon(p: POnCurve<Point3>) {
        poncurv(p);
    }
}
impl Space for Space2 {
    type Tool = Curve2dTool;
    fn adaptor(r: &mut Reader) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let c = curve2d(r)?;
        if r.flag() {
            let (u1, u2) = (r.real(), r.real());
            return Ok(Handle::new(Geom2dAdaptorCurve::with_range(&c, u1, u2)?).cast());
        }
        Ok(Handle::new(Geom2dAdaptorCurve::with_curve(&c)?).cast())
    }
    fn point(r: &mut Reader) -> Point2 {
        r.pnt2d()
    }
    fn pnt(p: Point2) {
        pnt2d(p);
    }
    fn pon(p: POnCurve<Point2>) {
        poncurv2d(p);
    }
}

type P<S> = <<S as Space>::Tool as ExtremaCurveTool>::Point;

fn extpc_result<S: Space>(e: &GGExtPC<'_, S::Tool>) {
    flag(e.is_done());
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            flag(e.is_min(i)?);
            S::pon(e.point(i)?);
        }
        Ok(())
    });
    let (d1, d2, p1, p2) = e.trimmed_square_distances();
    real(d1);
    real(d2);
    S::pnt(p1);
    S::pnt(p2);
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn extpc_body<S: Space>(r: &mut Reader) -> Result<(), Failure> {
    let a = S::adaptor(r)?;
    let (tolf, s1, s2) = (r.real(), r.real(), r.real());
    let n = r.integer();
    let mut reused = GGExtPC::<S::Tool>::new();
    attempt(|| reused.initialize(&*a, s1, s2, tolf));
    for _ in 0..n {
        let p = S::point(r);
        attempt(|| {
            Ok(extpc_result::<S>(&GGExtPC::<S::Tool>::with_curve(
                p, &*a, tolf,
            )?))
        });
        attempt(|| {
            reused.perform(p)?;
            extpc_result::<S>(&reused);
            Ok(())
        });
    }
    Ok(())
}

pub fn pc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| extpc_body::<Space3>(&mut r));
    end();
}
pub fn pc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| extpc_body::<Space2>(&mut r));
    end();
}

/// The accessors of a locate result (Extrema_LocateExtPC or Extrema_GenLocateExtPC).
trait LocView<S: Space> {
    fn done(&self) -> bool;
    fn sq(&self) -> Result<f64, Failure>;
    fn min(&self) -> Result<bool, Failure>;
    fn pon(&self) -> Result<POnCurve<P<S>>, Failure>;
}
impl<S: Space> LocView<S> for GLocateExtPC<'_, S::Tool> {
    fn done(&self) -> bool {
        self.is_done()
    }
    fn sq(&self) -> Result<f64, Failure> {
        self.square_distance()
    }
    fn min(&self) -> Result<bool, Failure> {
        self.is_min()
    }
    fn pon(&self) -> Result<POnCurve<P<S>>, Failure> {
        self.point()
    }
}
impl<S: Space> LocView<S> for GenLocateExtPC<'_, S::Tool> {
    fn done(&self) -> bool {
        self.is_done()
    }
    fn sq(&self) -> Result<f64, Failure> {
        self.square_distance()
    }
    fn min(&self) -> Result<bool, Failure> {
        self.is_min()
    }
    fn pon(&self) -> Result<POnCurve<P<S>>, Failure> {
        self.point()
    }
}
fn loc_result<S: Space>(e: &impl LocView<S>) {
    flag(e.done());
    attempt(|| Ok(real(e.sq()?)));
    attempt(|| Ok(flag(e.min()?)));
    attempt(|| Ok(S::pon(e.pon()?)));
}

fn locpc_body<S: Space>(r: &mut Reader, owned: bool) -> Result<(), Failure> {
    let a = S::adaptor(r)?;
    let (tolf, s1, s2) = (r.real(), r.real(), r.real());
    let n = r.integer();
    let mut reused = GLocateExtPC::<S::Tool>::new();
    if owned {
        attempt(|| reused.initialize_owned(a.clone(), (s1, s2), tolf));
    } else {
        attempt(|| reused.initialize(&*a, (s1, s2), tolf));
    }
    for _ in 0..n {
        let p = S::point(r);
        let u0 = r.real();
        attempt(|| {
            Ok(loc_result::<S>(&GLocateExtPC::<S::Tool>::with_curve(
                p, &*a, u0, tolf,
            )?))
        });
        attempt(|| {
            reused.perform(p, u0)?;
            loc_result::<S>(&reused);
            Ok(())
        });
    }
    Ok(())
}

pub fn locpc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| locpc_body::<Space3>(&mut r, false));
    end();
}
pub fn locpc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| locpc_body::<Space2>(&mut r, false));
    end();
}

fn epc_result<S: Space>(e: &GGenExtPC<'_, S::Tool>) {
    flag(e.is_done());
    attempt(|| {
        let n = e.nb_ext()?;
        int(n);
        for i in 1..=n {
            real(e.square_distance(i)?);
            flag(e.is_min(i)?);
            S::pon(e.point(i)?);
        }
        Ok(())
    });
    attempt(|| Ok(real(e.square_distance(0)?)));
}

fn genpc_body<S: Space>(r: &mut Reader) -> Result<(), Failure> {
    let a = S::adaptor(r)?;
    let nb = r.integer();
    let (umin, usup, tolu, tolf) = (r.real(), r.real(), r.real(), r.real());
    let n = r.integer();
    for _ in 0..n {
        let p = S::point(r);
        attempt(|| {
            Ok(epc_result::<S>(&GGenExtPC::<S::Tool>::with_curve(
                p, &*a, nb, tolu, tolf,
            )?))
        });
        attempt(|| {
            let e = GGenExtPC::<S::Tool>::with_range(p, &*a, nb, (umin, usup), tolu, tolf)?;
            epc_result::<S>(&e);
            Ok(())
        });
        attempt(|| {
            let l = GenLocateExtPC::<S::Tool>::with_range(
                p,
                &*a,
                0.5 * (umin + usup),
                (umin, usup),
                tolu,
            )?;
            loc_result::<S>(&l);
            Ok(())
        });
        attempt(|| {
            Ok(loc_result::<S>(&GenLocateExtPC::<S::Tool>::with_curve(
                p, &*a, umin, tolu,
            )?))
        });
    }
    Ok(())
}

pub fn genpc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| genpc_body::<Space3>(&mut r));
    end();
}
pub fn genpc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| genpc_body::<Space2>(&mut r));
    end();
}

fn pcf_step<S: Space>(f: &mut GFuncExtPC<'_, S::Tool>, u: f64) {
    let (mut v, mut d) = (0.0, 0.0);
    attempt(|| {
        flag(f.value(u, &mut v)?);
        real(v);
        Ok(())
    });
    attempt(|| {
        flag(f.derivative(u, &mut d)?);
        real(d);
        Ok(())
    });
    attempt(|| {
        let (mut v, mut d) = (0.0, 0.0);
        flag(f.values(u, &mut v, &mut d)?);
        real(v);
        real(d);
        Ok(())
    });
    attempt(|| Ok(int(f.get_state_number()?)));
}

fn pcf_body<S: Space>(r: &mut Reader) -> Result<(), Failure> {
    let a = S::adaptor(r)?;
    let p = S::point(r);
    let us = r.reals();
    let mut f = GFuncExtPC::<S::Tool>::with_point_curve(p, &*a)?;
    attempt(|| Ok(real(f.search_of_tolerance()?)));
    for u in us {
        pcf_step::<S>(&mut f, u);
    }
    int(f.nb_ext());
    for i in 0..=f.nb_ext() + 1 {
        attempt(|| {
            real(f.square_distance(i)?);
            flag(f.is_min(i)?);
            S::pon(f.point(i)?);
            Ok(())
        });
    }
    let mut g = GFuncExtPC::<S::Tool>::new();
    let mut v = 0.0;
    attempt(|| Ok(flag(g.value(0.5, &mut v)?)));
    g.set_point(p);
    attempt(|| Ok(flag(g.value(0.5, &mut v)?)));
    g.initialize(&*a)?;
    g.sub_interval_initialize(0.0, 1.0);
    attempt(|| {
        flag(g.value(0.5, &mut v)?);
        real(v);
        Ok(())
    });
    Ok(())
}

pub fn pcf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| pcf_body::<Space3>(&mut r));
    end();
}
pub fn pcf2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| pcf_body::<Space2>(&mut r));
    end();
}

fn cloc_body<S: Space>(r: &mut Reader) -> Result<(), Failure> {
    let a = S::adaptor(r)?;
    let n = r.integer();
    for _ in 0..n {
        let p = S::point(r);
        let nb = r.integer();
        let (umin, usup) = (r.real(), r.real());
        attempt(|| Ok(S::pon(GCurveLocator::<S::Tool>::locate(p, &*a, nb)?)));
        attempt(|| {
            Ok(S::pon(GCurveLocator::<S::Tool>::locate_range(
                p, &*a, nb, umin, usup,
            )?))
        });
    }
    Ok(())
}

pub fn cloc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| cloc_body::<Space3>(&mut r));
    end();
}
pub fn cloc2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| cloc_body::<Space2>(&mut r));
    end();
}

pub fn ctool_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = Space3::adaptor(&mut r)?;
        flag(CurveTool::is_periodic(&*a)?);
        attempt(|| Ok(real(CurveTool::period(&*a)?)));
        real(CurveTool::resolution(&*a, 1.0e-7)?);
        int(CurveTool::get_type(&*a)? as i32);
        int(CurveTool::continuity(&*a)? as i32);
        Ok(())
    });
    end();
}
pub fn ctool2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = Space2::adaptor(&mut r)?;
        flag(Curve2dTool::is_periodic(&*a)?);
        flag(Curve2dTool::is_closed(&*a)?);
        attempt(|| Ok(real(Curve2dTool::period(&*a)?)));
        real(Curve2dTool::resolution(&*a, 1.0e-7)?);
        int(Curve2dTool::get_type(&*a)? as i32);
        int(Curve2dTool::continuity(&*a)? as i32);
        Ok(())
    });
    end();
}

pub fn locpc_owned_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| locpc_body::<Space3>(&mut reader, true));
    end();
}
pub fn locpc2d_owned_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    attempt(|| locpc_body::<Space2>(&mut reader, true));
    end();
}
