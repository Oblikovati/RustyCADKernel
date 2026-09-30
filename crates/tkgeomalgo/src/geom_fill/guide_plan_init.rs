// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
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

/// `Init()` of the plane-guided law; `init_poles(&law)?`.
pub(crate) fn init_poles(law: &GeomFillGuideTrihedronPlan) -> Result<(), Failure> {
    let curve = law.fields.read().my_curve.clone();
    let c = live(&curve, "GeomFill_GuideTrihedronPlan::Init", "myCurve")?;
    law.fields.write().status = GeomFillPipeError::PipeOk;
    let (f, l) = (c.first_parameter()?, c.last_parameter()?);
    law.frenet().set_curve(&curve)?;
    let guide = law.guide_curve("GeomFill_GuideTrihedronPlan::Init")?;
    let g = guide.get().expect("checked");
    let mut delta_g = (g.last_parameter()? - g.first_parameter()?) / 2.0;
    {
        let mut fields = law.fields.write();
        fields.inf = g.first_parameter()? - delta_g;
        fields.sup = g.last_parameter()? + delta_g;
    }
    let trim_g = if g.is_periodic()? {
        guide.clone()
    } else {
        g.trim(
            g.first_parameter()? - delta_g / 100.0,
            g.last_parameter()? + delta_g / 100.0,
            delta_g * 1.0e-7,
        )?
    };
    law.set_trim_g(trim_g.clone());
    delta_g /= 3.0;
    let mut w = 0.0;
    for ii in 1..=NB_PTS {
        let t = sample_parameter(ii, f, l);
        let p = c.d0(t)?;
        let mut frame = TrihedronD0::default();
        law.frenet().d0(t, &mut frame)?;
        match nearest_intersection(&trim_g, p, &frame)? {
            Some(found) => w = found,
            None => {
                w = if (g.last_parameter()? - w).abs() > (g.first_parameter()? - w).abs() {
                    g.first_parameter()?
                } else {
                    g.last_parameter()?
                };
                law.fields.write().status = GeomFillPipeError::PlaneNotIntersectGuide;
            }
        }
        if ii > 1 {
            let previous = law.fields.read().pole[ii - 2].y();
            if (w - previous).abs() > delta_g && g.is_periodic()? {
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
