// Rust translation of GeomFill_GuideTrihedronPlan::Init and the static InGoodPeriod of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_GuideTrihedronPlan.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_GuideTrihedronPlan::Init`: the guide parameter where the normal plane
//! of the path cuts the guide, at twenty path parameters.
use super::guide_trihedron_plan::{GeomFillGuideTrihedronPlan, NB_PTS};
use super::pipe_error::GeomFillPipeError;
use super::sweep_failures::live;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0};
use crate::int_curve_surface::HInter;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Direction3, Point2, Point3};

/// `InGoodPeriod(Prec, Period, Current)`: `current` moved by whole periods to within
/// half a period of `prec`.
fn in_good_period(prec: f64, period: f64, current: &mut f64) {
    let mut diff = *current - prec;
    let nb = (diff / period).trunc() as i32;
    *current -= f64::from(nb) * period;
    diff = *current - prec;
    if diff > period / 2.0 {
        *current -= period;
    } else if diff < -period / 2.0 {
        *current += period;
    }
}

/// The guide parameter `W` of the intersection nearest to `p`, or `None` when the
/// plane misses the guide.
fn nearest_intersection(
    trim_g: &Handle<dyn Adaptor3dCurve>,
    p: Point3,
    frame: &TrihedronD0,
) -> Result<Option<f64>, Failure> {
    let plan: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_point(
        p,
        Direction3::from_vector(frame.tangent)?,
    )?)
    .cast();
    let pl: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plan)?).cast();
    let mut int = HInter::new();
    int.perform(trim_g, &pl)?;
    if int.nb_points()? == 0 {
        return Ok(None);
    }
    let mut p_int = int.point(1)?;
    let mut dmin = p.distance(&p_int.pnt());
    for jj in 2..=int.nb_points()? {
        let pmin = int.point(jj)?.pnt();
        if p.distance(&pmin) < dmin {
            p_int = int.point(jj)?;
            dmin = p.distance(&pmin);
        }
    }
    Ok(Some(p_int.w()))
}

/// The guide trimmed a hundredth of its half range beyond its ends (the guide itself
/// when periodic), with the solver bounds half a range beyond; returns the trimmed
/// guide and the half range.
fn trimmed_guide(
    law: &GeomFillGuideTrihedronPlan,
    guide: &Handle<dyn Adaptor3dCurve>,
) -> Result<(Handle<dyn Adaptor3dCurve>, f64), Failure> {
    let g = guide.get().expect("checked");
    let delta_g = (g.last_parameter()? - g.first_parameter()?) / 2.0;
    {
        let mut fields = law.fields.write();
        fields.inf = g.first_parameter()? - delta_g;
        fields.sup = g.last_parameter()? + delta_g;
    }
    let trim_g = if g.is_periodic()? {
        guide.clone()
    } else {
        let (f, l) = (g.first_parameter()?, g.last_parameter()?);
        g.trim(f - delta_g / 100.0, l + delta_g / 100.0, delta_g * 1.0e-7)?
    };
    Ok((trim_g, delta_g))
}

/// The guide parameter of sample `ii` at the path point `p`: the nearest plane
/// intersection, else the guide end nearer to the previous parameter `w`.
fn sample_guide_parameter(
    law: &GeomFillGuideTrihedronPlan,
    (g, trim_g): (&dyn Adaptor3dCurve, &Handle<dyn Adaptor3dCurve>),
    (t, p): (f64, Point3),
    w: f64,
) -> Result<f64, Failure> {
    let mut frame = TrihedronD0::default();
    law.frenet().d0(t, &mut frame)?;
    if let Some(found) = nearest_intersection(trim_g, p, &frame)? {
        return Ok(found);
    }
    law.fields.write().status = GeomFillPipeError::PlaneNotIntersectGuide;
    if (g.last_parameter()? - w).abs() > (g.first_parameter()? - w).abs() {
        return g.first_parameter();
    }
    g.last_parameter()
}

/// `Init()` of the plane-guided law; `init_poles(&law)?`.
pub(crate) fn init_poles(law: &GeomFillGuideTrihedronPlan) -> Result<(), Failure> {
    let curve = law.fields.read().my_curve.clone();
    let c = live(&curve, "GeomFill_GuideTrihedronPlan::Init", "myCurve")?;
    law.fields.write().status = GeomFillPipeError::PipeOk;
    let (f, l) = (c.first_parameter()?, c.last_parameter()?);
    law.frenet().set_curve(&curve)?;
    let guide = law.guide_curve("GeomFill_GuideTrihedronPlan::Init")?;
    let g = guide.get().expect("checked");
    let (trim_g, delta_g) = trimmed_guide(law, &guide)?;
    law.set_trim_g(trim_g.clone());
    let mut w = 0.0;
    for ii in 1..=NB_PTS {
        let t = sample_parameter(ii, f, l);
        w = sample_guide_parameter(law, (g, &trim_g), (t, c.d0(t)?), w)?;
        if ii > 1 {
            let previous = law.fields.read().pole[ii - 2].y();
            if (w - previous).abs() > delta_g / 3.0 && g.is_periodic()? {
                in_good_period(previous, g.period()?, &mut w);
            }
        }
        law.fields.write().pole[ii - 1] = Point2::new(t, w);
    }
    Ok(())
}

/// `t = ((myNbPts - ii) f + (ii - 1) l) / (myNbPts - 1)`.
fn sample_parameter(ii: usize, f: f64, l: f64) -> f64 {
    let mut t = (NB_PTS - ii) as f64 * f + (ii - 1) as f64 * l;
    t /= (NB_PTS - 1) as f64;
    t
}
