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
// Rust translation of GeomFill_LocationGuide::SetRotation and the static InGoodPeriod
// of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocationGuide.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `SetRotation`: at `myNbPts` path parameters, the surface of revolution of the
//! placed section around the tangent and its nearest point to the guide, giving the
//! guide parameter, the contact angle and the section parameter of the contact.
use super::function_guide::GeomFillFunctionGuide;
use super::location_guide::{GeomFillLocationGuide, NB_PTS, sample_parameter};
use super::location_guide_eval::vector3;
use super::location_helpers::columns;
use super::pipe_error::GeomFillPipeError;
use super::section_law::GeomFillSectionLaw;
use super::sweep_failures::{construction_error, live};
use super::trihedron_law::TrihedronD0;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomSurface, GeomSurfaceOfRevolution, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::extrema::ExtCS;
use rustycad_tkmath::gp::{self, Axis1, Axis3, Direction3, Point2, Point3, Transform3};
use rustycad_tkmath::math::FunctionSetRoot;
use std::f64::consts::PI;

/// `InGoodPeriod(Prec, Period, Current)`.
pub(crate) fn in_good_period(prec: f64, period: f64, current: &mut f64) {
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

/// The section as `SetRotation` builds it: the constant section, or the arrays of a
/// varying one (`Mult`, `Knots`, `Poles`, `Weights`, `Deg`, `israt`).
enum RotationSection {
    Constant(Handle<dyn GeomCurve>),
    Varying {
        mult: Array1<'static, i32>,
        knots: Array1<'static, f64>,
        nb_poles: i32,
        deg: i32,
        israt: bool,
    },
}

/// The section, and its parameter range `(Uf, Ul)`.
fn rotation_section(sec: &dyn GeomFillSectionLaw) -> Result<(RotationSection, f64, f64), Failure> {
    let (nb_poles, nb_knots, deg) = sec.section_shape()?;
    let mut tol = 1.0e-9;
    if sec.is_constant(&mut tol)? {
        let section = sec.constant_section()?;
        let c = live(&section, "GeomFill_LocationGuide::SetRotation", "mySection")?;
        let (uf, ul) = (c.first_parameter(), c.last_parameter());
        return Ok((RotationSection::Constant(section), uf, ul));
    }
    let israt = sec.is_rational()?;
    let mut mult = one_based(nb_knots)?;
    sec.mults(&mut mult)?;
    let mut knots: Array1<'static, f64> = one_based(nb_knots)?;
    sec.knots(&mut knots)?;
    let (uf, ul) = (knots[1], knots[nb_knots]);
    Ok((
        RotationSection::Varying {
            mult,
            knots,
            nb_poles,
            deg,
            israt,
        },
        uf,
        ul,
    ))
}

/// The contact of one sample: the guide parameter `w` and the surface parameters
/// `(theU, theV)`.
struct Contact {
    w: f64,
    the_u: f64,
    the_v: f64,
}

impl GeomFillLocationGuide {
    /// The trimmed section `S` at the path parameter `t` (section parameter `u`).
    fn placed_section(
        &self,
        section: &RotationSection,
        sec: &dyn GeomFillSectionLaw,
        u: f64,
        (uf, ul): (f64, f64),
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        let curve: Handle<dyn GeomCurve> = match section {
            RotationSection::Constant(c) => live(c, "SetRotation", "mySection")?.copy()?.cast(),
            RotationSection::Varying {
                mult,
                knots,
                nb_poles,
                deg,
                israt,
            } => {
                let mut poles: Array1<'static, Point3> = one_based(*nb_poles)?;
                let mut weights: Array1<'static, f64> = one_based(*nb_poles)?;
                sec.d0(u, &mut poles, &mut weights)?;
                let periodic = sec.is_u_periodic()?;
                let bspline = if *israt {
                    GeomBSplineCurve::with_weights(&poles, &weights, knots, mult, *deg, periodic)?
                } else {
                    GeomBSplineCurve::new(&poles, knots, mult, *deg, periodic)?
                };
                Handle::new(bspline).cast()
            }
        };
        Ok(Handle::new(GeomTrimmedCurve::new(&curve, uf, ul, true, true)?).cast())
    }

    /// The axis direction `D` (the tangent, or the transformed third column).
    fn axis_direction(&self, frame: &TrihedronD0) -> Result<Direction3, Failure> {
        let f = self.fields.read();
        if !f.with_trans {
            return Direction3::from_vector(frame.tangent);
        }
        let mut m = columns(frame.normal, frame.binormal, frame.tangent);
        m *= f.trans;
        Direction3::from_xyz(m.column(3)?)
    }

    /// The contact on the surface of revolution `revol`, or the rescue search from
    /// the previous sample (`None` when both fail).
    fn find_contact(
        &self,
        revol: &Handle<dyn GeomSurface>,
        ii: usize,
        (p, frame, u): (Point3, &TrihedronD0, f64),
    ) -> Result<Option<Contact>, Failure> {
        let ga_revol = GeomAdaptorSurface::with_surface(revol)?;
        let dist_mini = ExtCS::with_curve(
            self.guide_curve()?,
            &ga_revol,
            precision::confusion(),
            precision::confusion(),
        )?;
        if !dist_mini.is_done() || dist_mini.nb_ext()? == 0 {
            return self.rescue_contact(ii, (p, frame, u));
        }
        let (mut min_dist, mut jref) = (f64::MAX, 0);
        for j in 1..=dist_mini.nb_ext()? {
            let a_dist = dist_mini.square_distance(j)?;
            if a_dist < min_dist {
                min_dist = a_dist;
                jref = j;
            }
        }
        let (pc, ps) = dist_mini.points(jref)?;
        let (the_u, the_v) = ps.parameter();
        Ok(Some(Contact {
            w: pc.parameter(),
            the_u,
            the_v,
        }))
    }

    /// The rescue of a failed extrema search: the guide function solved from the
    /// previous sample; the guide parameter stays 0 (the native `Pc` is never set).
    fn rescue_contact(
        &self,
        ii: usize,
        (p, frame, u): (Point3, &TrihedronD0, f64),
    ) -> Result<Option<Contact>, Failure> {
        if ii <= 1 {
            return Ok(None);
        }
        let f = self.fields.read().clone();
        let x = [
            f.poles2d[0][ii - 2].y(),
            f.poles2d[1][ii - 2].x(),
            f.poles2d[1][ii - 2].y(),
        ];
        self.fields.write().x = x;
        let mut e = GeomFillFunctionGuide::new(&f.my_sec, &self.my_guide, u)?;
        e.set_param(u, p, *frame.tangent.xyz(), *frame.normal.xyz())?;
        let mut result = FunctionSetRoot::new(&e, &vector3(f.tol_res)?, 100)?;
        result.perform_bounded(
            &mut e,
            &vector3(x)?,
            &vector3(f.inf)?,
            &vector3(f.sup)?,
            false,
        )?;
        e.take_failure()?;
        if !(result.is_done()
            && result.function_set_errors()?.norm()? < f.tol_res[0] * f.tol_res[0])
        {
            return Ok(None);
        }
        let rr = result.root()?;
        Ok(Some(Contact {
            w: 0.0,
            the_u: rr[2],
            the_v: rr[3],
        }))
    }

    /// The search bounds of `(w, angle, v)`.
    fn set_bounds(&self, (uf, ul): (f64, f64)) -> Result<(), Failure> {
        let guide = self.guide_curve()?;
        let (gf, gl) = (guide.first_parameter()?, guide.last_parameter()?);
        let delta = gl - gf;
        let mut f = self.fields.write();
        (f.uf, f.ul) = (uf, ul);
        f.inf[0] = gf - delta / 10.0;
        f.sup[0] = gl + delta / 10.0;
        f.inf[1] = -PI;
        f.sup[1] = 3.0 * PI;
        let delta = ul - uf;
        f.inf[2] = uf - delta / 10.0;
        f.sup[2] = ul + delta / 10.0;
        Ok(())
    }

    /// `SetRotation(PrecAngle, LastAngle)`: the contact samples; `last_angle` is the
    /// last contact angle when every sample succeeds (the status says otherwise).
    pub(crate) fn set_rotation(
        &self,
        prec_angle: f64,
        last_angle: &mut f64,
    ) -> Result<(), Failure> {
        let curve = self.fields.read().my_curve.clone();
        let Some(c) = curve.get() else {
            return Err(construction_error(
                "GeomFill_LocationGuide::The path is not set !!",
            ));
        };
        let sec_handle = self.fields.read().my_sec.clone();
        let sec = live(&sec_handle, "GeomFill_LocationGuide::SetRotation", "mySec")?;
        let (f, l) = (c.first_parameter()?, c.last_parameter()?);
        let uperiodic = sec.is_u_periodic()?;
        let guide = self.guide_curve()?;
        let delta_g = (guide.last_parameter()? - guide.first_parameter()?) / 5.0;
        let (section, uf, ul) = rotation_section(sec)?;
        self.set_bounds((uf, ul))?;
        let u_period = if uperiodic { ul - uf } else { 0.0 };
        let mut walk = RotationWalk {
            cur_angle: prec_angle,
            old_angle: 0.0,
            delta_g,
            u_period,
            uperiodic,
        };
        for ii in 1..=NB_PTS {
            let t = sample_parameter(ii, f, l);
            if !self.rotation_sample(ii, t, c, (&section, sec, (uf, ul)), &mut walk)? {
                return Ok(());
            }
        }
        *last_angle = walk.cur_angle;
        self.fields.write().rotation = true;
        Ok(())
    }

    /// One sample of `SetRotation`; false after recording a failure status.
    fn rotation_sample(
        &self,
        ii: usize,
        t: f64,
        c: &dyn Adaptor3dCurve,
        (section, sec, bounds): (&RotationSection, &dyn GeomFillSectionLaw, (f64, f64)),
        walk: &mut RotationWalk,
    ) -> Result<bool, Failure> {
        let p = c.d0(t)?;
        let mut frame = TrihedronD0::default();
        if !self.law().d0(t, &mut frame)? {
            self.fields.write().status = self.law().error_status();
            return Ok(false);
        }
        let d = self.axis_direction(&frame)?;
        let ax = Axis1::new(p, d);
        let n3 = Axis3::new(p, d, Direction3::from_vector(frame.normal)?)?;
        let mut transfo = Transform3::default();
        transfo.set_transformation_between(n3, Axis3::new(gp::origin(), gp::dz(), gp::dx())?);
        let fields = self.fields.read().clone();
        let u = match section {
            RotationSection::Constant(_) => 0.0,
            RotationSection::Varying { .. } => {
                fields.my_first_s + (t - c.first_parameter()?) * fields.ratio
            }
        };
        let s = self.placed_section(section, sec, u, bounds)?;
        s.transform(transfo)?;
        let revol: Handle<dyn GeomSurface> =
            Handle::new(GeomSurfaceOfRevolution::new(&s, ax)?).cast();
        let Some(contact) = self.find_contact(&revol, ii, (p, &frame, u))? else {
            self.fields.write().status = GeomFillPipeError::ImpossibleContact;
            return Ok(false);
        };
        self.store_contact(ii, t, contact, walk)?;
        Ok(true)
    }

    /// Brings the contact into the periods of the previous one and stores it.
    fn store_contact(
        &self,
        ii: usize,
        t: f64,
        contact: Contact,
        walk: &mut RotationWalk,
    ) -> Result<(), Failure> {
        let prev = self.fields.read().poles2d.clone();
        let mut w = contact.w;
        if ii > 1 {
            let diff = w - prev[0][ii - 2].y();
            let guide = self.guide_curve()?;
            if diff.abs() > walk.delta_g && guide.is_periodic()? {
                in_good_period(prev[0][ii - 2].y(), guide.period()?, &mut w);
            }
        }
        let mut angle = contact.the_u;
        if ii > 1 && (angle - walk.old_angle).abs() > PI {
            in_good_period(walk.old_angle, 2.0 * PI, &mut angle);
        }
        let mut v = contact.the_v;
        if ii > 1 && walk.uperiodic {
            in_good_period(prev[1][ii - 2].y(), walk.u_period, &mut v);
        }
        walk.cur_angle = angle;
        let mut f = self.fields.write();
        f.poles2d[0][ii - 1] = Point2::new(t, w);
        f.poles2d[1][ii - 1] = Point2::new(angle, v);
        walk.old_angle = angle;
        Ok(())
    }
}

/// The running values of the sample loop (`CurAngle`, `OldAngle`, `DeltaG`,
/// `UPeriod`, `uperiodic`).
struct RotationWalk {
    cur_angle: f64,
    old_angle: f64,
    delta_g: f64,
    u_period: f64,
    uperiodic: bool,
}
