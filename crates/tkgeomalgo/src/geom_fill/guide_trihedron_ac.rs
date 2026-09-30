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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_GuideTrihedronAC.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_GuideTrihedronAC`: a frame whose normal points to the guide point of
//! the same relative arc length.
use super::frenet::copy_fusion;
use super::sweep_failures::live;
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use super::trihedron_with_guide::{GeomFillTrihedronWithGuide, GeomFillTrihedronWithGuideBase};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::approx::CurvlinFunc;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point3, Vector3};

/// The fields `SetCurve` and `Origine` write (the class's own `myCurve`, which
/// hides the base one, `myCurveAC`, `L`, `Orig1`, `Orig2`).
#[derive(Clone, Default)]
struct AcFields {
    my_curve: Handle<dyn Adaptor3dCurve>,
    curve_ac: Handle<CurvlinFunc>,
    l: f64,
    orig1: f64,
    orig2: f64,
}

/// `GeomFill_GuideTrihedronAC`: the guided law by arc length.
///
/// The class declares its own `myCurve`, hiding the base member, which stays null;
/// `myTrimmed` is the base member.
/// `let law = GeomFillGuideTrihedronAC::new(&guide)?; law.set_curve(&path)?;`.
pub struct GeomFillGuideTrihedronAC {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    guide_base: GeomFillTrihedronWithGuideBase,
    guide_ac: Handle<CurvlinFunc>,
    lguide: f64,
    u_tol: f64,
    fields: SharedState<AcFields>,
}
impl_transient!(
    GeomFillGuideTrihedronAC,
    "GeomFill_GuideTrihedronAC",
    dyn GeomFillTrihedronWithGuide,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw, dyn GeomFillTrihedronWithGuide]
);

impl GeomFillGuideTrihedronAC {
    /// `GeomFill_GuideTrihedronAC(guide)`: the arc length function of the guide
    /// (tolerance 1e-7), origins 0 and 1.
    pub fn new(guide: &Handle<dyn Adaptor3dCurve>) -> Result<Self, Failure> {
        let guide_ac = Handle::new(CurvlinFunc::from_curve(guide, 1.0e-7)?);
        let lguide = guide_ac.get().expect("new function").get_length();
        let law = Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            guide_base: GeomFillTrihedronWithGuideBase::default(),
            guide_ac,
            lguide,
            u_tol: precision::p_confusion(),
            fields: SharedState::new(AcFields {
                orig1: 0.0,
                orig2: 1.0,
                ..AcFields::default()
            }),
        };
        law.guide_base.edit(|g| {
            g.guide = guide.clone();
            g.trim_g = guide.clone();
        });
        Ok(law)
    }

    fn guide_ac(&self) -> &CurvlinFunc {
        self.guide_ac
            .get()
            .expect("the constructor creates the guide function")
    }

    fn curve_ac(&self, member: &str) -> Result<Handle<CurvlinFunc>, Failure> {
        let ac = self.fields.read().curve_ac.clone();
        live(&ac, member, "myCurveAC")?;
        Ok(ac)
    }

    /// `tG`: the guide parameter at the relative arc length `OrigG` of `param` on
    /// the path (through `s`).
    fn sample(&self, param: f64, member: &str) -> Result<f64, Failure> {
        let ac = self.curve_ac(member)?;
        let s = ac.get().expect("checked").get_s_parameter(param)?;
        let f = self.fields.read().clone();
        let orig_g = f.orig1 + s * (f.orig2 - f.orig1);
        let guide = self.guide_base.fields().guide;
        self.guide_ac()
            .get_u_parameter(live(&guide, member, "myGuide")?, orig_g, 1)
    }

    /// `myTrimmed` and `myTrimG`.
    fn trimmed_pair(
        &self,
        member: &str,
    ) -> Result<(Handle<dyn Adaptor3dCurve>, Handle<dyn Adaptor3dCurve>), Failure> {
        let (trimmed, trim_g) = (self.base.trimmed(), self.guide_base.fields().trim_g);
        live(&trimmed, member, "myTrimmed")?;
        live(&trim_g, member, "myTrimG")?;
        Ok((trimmed, trim_g))
    }

    fn set_current_point(&self, pg: Point3) {
        self.guide_base.edit(|g| g.cur_point_on_guide = pg);
    }

    /// `dtg = (Orig2 - Orig1) * (|To| / |TG|) * (Lguide / L)`.
    fn guide_speed(&self, to: Vector3, tg: Vector3) -> f64 {
        let f = self.fields.read();
        (f.orig2 - f.orig1) * (to.magnitude() / tg.magnitude()) * (self.lguide / f.l)
    }
}

/// `-(u . v) u + v`, the native `SetLinearForm(-(u.Dot(v)), u, v)`.
fn tangential_part(u: Vector3, v: Vector3) -> Vector3 {
    let mut out = Vector3::default();
    out.set_scaled_sum(-(u.dot(v)), u, v);
    out
}

impl GeomFillTrihedronLaw for GeomFillGuideTrihedronAC {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    /// Loads the path and its arc length function; always true.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        self.fields.write().my_curve = curve.clone();
        self.base.set_trimmed(curve);
        if curve.is_some() {
            let ac = Handle::new(CurvlinFunc::from_curve(curve, 1.0e-7)?);
            let l = ac.get().expect("new function").get_length();
            let mut f = self.fields.write();
            f.curve_ac = ac;
            f.l = l;
        }
        Ok(true)
    }
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillGuideTrihedronAC::new(&self.guide_base.fields().guide)?;
        let f = self.fields.read().clone();
        copy.set_curve(&f.my_curve)?;
        copy.origine(f.orig1, f.orig2)?;
        Ok(Handle::new(copy).cast())
    }
    fn d0(&self, param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        let tg_param = self.sample(param, "GeomFill_GuideTrihedronAC::D0")?;
        let (trimmed, trim_g) = self.trimmed_pair("GeomFill_GuideTrihedronAC::D0")?;
        let (p, to) = trimmed.get().expect("checked").d1(param)?;
        let pg = trim_g.get().expect("checked").d0(tg_param)?;
        self.set_current_point(pg);
        frame.normal = Vector3::between(p, pg).normalized()?;
        let b = to.crossed(frame.normal);
        frame.binormal = b / b.magnitude();
        frame.tangent = frame.normal.crossed(frame.binormal);
        frame.tangent.normalize()?;
        Ok(true)
    }
    fn d1(&self, param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        let tg_param = self.sample(param, "GeomFill_GuideTrihedronAC::D1")?;
        let (trimmed, trim_g) = self.trimmed_pair("GeomFill_GuideTrihedronAC::D1")?;
        let (p, to, d_to) = trimmed.get().expect("checked").d2(param)?;
        let (pg, tg) = trim_g.get().expect("checked").d1(tg_param)?;
        self.set_current_point(pg);
        let mut n = Vector3::between(p, pg);
        let mut norm = n.magnitude();
        if norm < 1.0e-12 {
            norm = 1.0;
        }
        n /= norm;
        let dtg = self.guide_speed(to, tg);
        let mut dn = Vector3::default();
        dn.set_linear_form(dtg, tg, -1.0, to);
        dn /= norm;
        frame.normal = n;
        let mut b = to.crossed(frame.normal);
        let norm_b = b.magnitude();
        b /= norm_b;
        frame.binormal = b;
        frame.tangent = frame.normal.crossed(frame.binormal);
        frame.tangent.normalize()?;
        frame.d_normal = tangential_part(n, dn);
        let mut b_prim = Vector3::default();
        b_prim.set_sum(d_to.crossed(frame.normal), to.crossed(frame.d_normal));
        frame.d_binormal = tangential_part(b, b_prim);
        frame.d_binormal /= norm_b;
        frame.d_tangent.set_sum(
            frame.normal.crossed(frame.d_binormal),
            frame.d_normal.crossed(frame.binormal),
        );
        Ok(true)
    }
    fn d2(&self, param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        let tg_param = self.sample(param, "GeomFill_GuideTrihedronAC::D2")?;
        let (trimmed, trim_g) = self.trimmed_pair("GeomFill_GuideTrihedronAC::D2")?;
        let path = trimmed.get().expect("checked").d3(param)?;
        let guide = trim_g.get().expect("checked").d2(tg_param)?;
        self.set_current_point(guide.0);
        let f = self.fields.read().clone();
        super::guide_trihedron_ac_d2::ac_d2(
            frame,
            path,
            guide,
            [f.orig2 - f.orig1, self.lguide / f.l],
        );
        Ok(true)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        Ok(self.fused_abscissae(shape)?.len() as i32 - 1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let seq = self.fused_abscissae(shape)?;
        let ac = self.curve_ac("GeomFill_GuideTrihedronAC::Intervals")?;
        let my_curve = self.fields.read().my_curve.clone();
        let c = live(&my_curve, "GeomFill_GuideTrihedronAC::Intervals", "myCurve")?;
        let mut params = LinearVector::with_capacity(seq.len());
        for s in seq.as_slice() {
            params.append(ac.get().expect("checked").get_u_parameter(c, *s, 1)?);
        }
        copy_fusion(&params, bounds)
    }
    /// Trims the path and the guide to the matching parameters.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        let f = self.fields.read().clone();
        let c = live(
            &f.my_curve,
            "GeomFill_GuideTrihedronAC::SetInterval",
            "myCurve",
        )?;
        self.base.set_trimmed(&c.trim(first, last, self.u_tol)?);
        let ac = self.curve_ac("GeomFill_GuideTrihedronAC::SetInterval")?;
        let (sf, sl) = (
            ac.get().expect("checked").get_s_parameter(first)?,
            ac.get().expect("checked").get_s_parameter(last)?,
        );
        let guide = self.guide_base.fields().guide;
        let gc = live(&guide, "GeomFill_GuideTrihedronAC::SetInterval", "myGuide")?;
        let u = f.orig1 + sf * (f.orig2 - f.orig1);
        let sf = self.guide_ac().get_u_parameter(gc, u, 1)?;
        let u = f.orig1 + sl * (f.orig2 - f.orig1);
        let sl = self.guide_ac().get_u_parameter(gc, u, 1)?;
        let trim_g = gc.trim(sf, sl, self.u_tol)?;
        self.guide_base.edit(|g| g.trim_g = trim_g);
        Ok(())
    }
    /// The mean of 20 frames over `[First, Last - Delta]`, not normalized.
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        let my_curve = self.fields.read().my_curve.clone();
        let c = live(
            &my_curve,
            "GeomFill_GuideTrihedronAC::GetAverageLaw",
            "myCurve",
        )?;
        let delta = (c.last_parameter()? - c.first_parameter()?) / 20.001;
        let (mut sum, mut frame) = (TrihedronD0::default(), TrihedronD0::default());
        for ii in 1..=20 {
            let t = c.first_parameter()? + f64::from(ii - 1) * delta;
            self.d0(t, &mut frame)?;
            sum.tangent += frame.tangent;
            sum.normal += frame.normal;
            sum.binormal += frame.binormal;
        }
        sum.tangent /= 20.0;
        sum.normal /= 20.0;
        sum.binormal /= 20.0;
        Ok(sum)
    }
}

impl GeomFillGuideTrihedronAC {
    /// The fusion (confusion 1e-9, midpoints) of the arc-length intervals of the
    /// path and of the guide.
    fn fused_abscissae(&self, shape: Shape) -> Result<LinearVector<f64>, Failure> {
        let ac = self.curve_ac("GeomFill_GuideTrihedronAC::NbIntervals")?;
        let ac = ac.get().expect("checked");
        let mut disc_c = one_based(ac.nb_intervals(shape)? + 1)?;
        ac.intervals(&mut disc_c, shape)?;
        let mut disc_g = one_based(self.guide_ac().nb_intervals(shape)? + 1)?;
        self.guide_ac().intervals(&mut disc_g, shape)?;
        let mut seq = LinearVector::with_capacity(0);
        geom_lib::fuse_intervals(&disc_c, &disc_g, &mut seq, 1.0e-9, false);
        Ok(seq)
    }
}

impl GeomFillTrihedronWithGuide for GeomFillGuideTrihedronAC {
    fn guide_base(&self) -> &GeomFillTrihedronWithGuideBase {
        &self.guide_base
    }
    /// `Origine(OrACR1, OrACR2)`: the relative arc lengths of the path ends on the
    /// guide.
    fn origine(&self, param1: f64, param2: f64) -> Result<(), Failure> {
        let mut f = self.fields.write();
        f.orig1 = param1;
        f.orig2 = param2;
        Ok(())
    }
}
