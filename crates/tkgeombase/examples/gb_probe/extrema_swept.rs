//! The Extrema requests on swept surfaces (ExtPExtS, ExtPRevS); the counterpart of the
//! swept part of tools/oracle/tkgeombase_probe_extrema_ps.cxx (same encoding and
//! token order).
#![allow(clippy::unit_arg, clippy::type_complexity)]
use super::common::{Reader, attempt, end, flag, int};
use super::extrema_ps::{PsView, ps_result};
use super::pc3d::curve3d;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurfaceOfLinearExtrusion, GeomAdaptorSurfaceOfRevolution,
};
use rustycad_tkgeombase::extrema::{ExtPExtS, ExtPRevS};
use rustycad_tkmath::gp::Point3;

/// gb_ex_basis_curve: a curve adaptor (gb_ex_adaptor) as a handle.
fn basis_curve(r: &mut Reader) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let c = curve3d(r)?;
    if r.flag() {
        let (u1, u2) = (r.real(), r.real());
        return Ok(Handle::new(GeomAdaptorCurve::with_range(&c, u1, u2)?).cast());
    }
    Ok(Handle::new(GeomAdaptorCurve::with_curve(&c)?).cast())
}

/// The members gb_ex_swept_runs uses.
trait Swept: PsView + Sized {
    type Surface: rustycad_tkernel::standard::RuntimeTyped;
    fn with_surface(
        p: Point3,
        s: &Handle<Self::Surface>,
        tu: f64,
        tv: f64,
    ) -> Result<Self, Failure>;
    fn with_range(
        p: Point3,
        s: &Handle<Self::Surface>,
        u: (f64, f64),
        v: (f64, f64),
        t: (f64, f64),
    ) -> Result<Self, Failure>;
    fn fresh() -> Self;
    fn init(
        &mut self,
        s: &Handle<Self::Surface>,
        u: (f64, f64),
        v: (f64, f64),
        t: (f64, f64),
    ) -> Result<(), Failure>;
    fn run(&mut self, p: Point3) -> Result<(), Failure>;
}

macro_rules! swept {
    ($t:ty, $s:ty) => {
        impl Swept for $t {
            type Surface = $s;
            fn with_surface(p: Point3, s: &Handle<$s>, tu: f64, tv: f64) -> Result<Self, Failure> {
                <$t>::with_surface(p, s, tu, tv)
            }
            fn with_range(
                p: Point3,
                s: &Handle<$s>,
                u: (f64, f64),
                v: (f64, f64),
                t: (f64, f64),
            ) -> Result<Self, Failure> {
                <$t>::with_range(p, s, u, v, t)
            }
            fn fresh() -> Self {
                <$t>::new()
            }
            fn init(
                &mut self,
                s: &Handle<$s>,
                u: (f64, f64),
                v: (f64, f64),
                t: (f64, f64),
            ) -> Result<(), Failure> {
                self.initialize(s, u, v, t)
            }
            fn run(&mut self, p: Point3) -> Result<(), Failure> {
                self.perform(p)
            }
        }
    };
}
swept!(ExtPExtS, GeomAdaptorSurfaceOfLinearExtrusion);
swept!(ExtPRevS, GeomAdaptorSurfaceOfRevolution);

fn swept_runs<E: Swept>(
    s: &Handle<E::Surface>,
    (p, p2): (Point3, Point3),
    u: (f64, f64),
    v: (f64, f64),
    t: (f64, f64),
) {
    attempt(|| {
        let mut e = E::with_surface(p, s, t.0, t.1)?;
        ps_result(&e);
        attempt(|| {
            e.run(p2)?;
            ps_result(&e);
            Ok(())
        });
        Ok(())
    });
    attempt(|| {
        let e = E::with_range(p, s, u, v, t)?;
        ps_result(&e);
        Ok(())
    });
    let mut d = E::fresh();
    flag(d.done());
    attempt(|| Ok(int(d.count()?)));
    attempt(|| {
        d.init(s, u, v, t)?;
        d.run(p)?;
        ps_result(&d);
        d.init(s, u, v, t)?;
        attempt(|| {
            d.run(p2)?;
            ps_result(&d);
            Ok(())
        });
        Ok(())
    });
}

// The common tail of ex_pexts and ex_prevs: P u1 u2 v1 v2 tolu tolv P2.
fn swept_tail(r: &mut Reader) -> ((Point3, Point3), (f64, f64), (f64, f64), (f64, f64)) {
    let p = r.pnt();
    let (u1, u2, v1, v2, tu, tv) = (r.real(), r.real(), r.real(), r.real(), r.real(), r.real());
    let p2 = r.pnt();
    ((p, p2), (u1, u2), (v1, v2), (tu, tv))
}

fn pexts_body(r: &mut Reader) -> Result<(), Failure> {
    let c = basis_curve(r)?;
    let dir = r.dir();
    let (points, u, v, t) = swept_tail(r);
    let s = Handle::new(GeomAdaptorSurfaceOfLinearExtrusion::with_curve_and_direction(&c, dir));
    swept_runs::<ExtPExtS>(&s, points, u, v, t);
    Ok(())
}

pub fn pexts_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| pexts_body(&mut r));
    end();
}

fn prevs_body(r: &mut Reader) -> Result<(), Failure> {
    let c = basis_curve(r)?;
    let ax = r.ax1();
    let (points, u, v, t) = swept_tail(r);
    let s = Handle::new(GeomAdaptorSurfaceOfRevolution::with_curve_and_axis(&c, ax)?);
    swept_runs::<ExtPRevS>(&s, points, u, v, t);
    Ok(())
}

pub fn prevs_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| prevs_body(&mut r));
    end();
}
