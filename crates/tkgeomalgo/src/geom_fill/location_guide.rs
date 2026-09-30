// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocationGuide.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_LocationGuide`: the location law of a guided sweep, whose section may
//! be turned around the tangent so that it touches the guide. The rotation search is
//! in `location_guide_rotation.rs`, the evaluators in `location_guide_eval.rs`.
use super::location_helpers::{columns, differs_from_identity};
use super::location_law::{
    GeomFillLocationLaw, LocationD0, LocationD1, LocationD2, LocationPoles2d,
};
use super::pipe_error::GeomFillPipeError;
use super::section_law::GeomFillSectionLaw;
use super::sweep_failures::{live, not_implemented};
use super::trihedron_law::{TrihedronD0, TrihedronD1, TrihedronD2};
use super::trihedron_with_guide::GeomFillTrihedronWithGuide;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, LinearVector};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::{one_based, put};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Matrix3, Point2, Point3, Vector2, Vector3};

/// The number of sampled path points (`myNbPts`).
pub(crate) const NB_PTS: usize = 21;

/// The fields of the law: the section, the path, the rotation state, the
/// transformation, the solver data (`TolRes`, `Inf`, `Sup`, `X`, `R`) and the
/// sampled contact parameters (`myPoles2d`: row 1 `(t, w)`, row 2 `(angle, v)`).
#[derive(Clone)]
pub(crate) struct GuideLocFields {
    pub my_sec: Handle<dyn GeomFillSectionLaw>,
    pub my_curve: Handle<dyn Adaptor3dCurve>,
    pub my_trimmed: Handle<dyn Adaptor3dCurve>,
    pub rotation: bool,
    pub orig_param1: f64,
    pub orig_param2: f64,
    pub uf: f64,
    pub ul: f64,
    pub my_first_s: f64,
    pub my_last_s: f64,
    pub ratio: f64,
    pub with_trans: bool,
    pub trans: Matrix3,
    pub tol_res: [f64; 3],
    pub inf: [f64; 3],
    pub sup: [f64; 3],
    pub x: [f64; 3],
    pub r: [f64; 3],
    pub status: GeomFillPipeError,
    pub poles2d: [Vec<Point2>; 2],
}

/// `GeomFill_LocationGuide`: the guided location law.
/// `let law = GeomFillLocationGuide::new(&trihedron)?; law.set_curve(&path)?;`.
pub struct GeomFillLocationGuide {
    parent: StandardTransient,
    pub(crate) my_law: Handle<dyn GeomFillTrihedronWithGuide>,
    pub(crate) my_guide: Handle<dyn Adaptor3dCurve>,
    pub(crate) fields: SharedState<GuideLocFields>,
}
impl_transient!(
    GeomFillLocationGuide,
    "GeomFill_LocationGuide",
    dyn GeomFillLocationLaw,
    parent,
    interfaces: [dyn GeomFillLocationLaw]
);

/// The guide of the law, extended by 1% at both ends when it is not periodic.
fn extended_guide(
    law: &dyn GeomFillTrihedronWithGuide,
) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let guide = law.guide();
    let g = live(&guide, "GeomFill_LocationGuide", "myGuide")?;
    if g.is_periodic()? {
        return Ok(guide.clone());
    }
    let (mut f, mut l) = (g.first_parameter()?, g.last_parameter()?);
    let delta = (l - f) / 100.0;
    f -= delta;
    l += delta;
    g.trim(f, l, delta * 1.0e-7)
}

impl GeomFillLocationGuide {
    /// `GeomFill_LocationGuide(Triedre)`: no section, no path, no rotation.
    pub fn new(triedre: &Handle<dyn GeomFillTrihedronWithGuide>) -> Result<Self, Failure> {
        let my_guide = extended_guide(live(triedre, "GeomFill_LocationGuide", "Triedre")?)?;
        let mut trans = Matrix3::default();
        trans.set_identity();
        Ok(Self {
            parent: StandardTransient::default(),
            my_law: triedre.clone(),
            my_guide,
            fields: SharedState::new(GuideLocFields {
                my_sec: Handle::default(),
                my_curve: Handle::default(),
                my_trimmed: Handle::default(),
                rotation: false,
                orig_param1: 0.0,
                orig_param2: 1.0,
                uf: 0.0,
                ul: 0.0,
                my_first_s: -505e77,
                my_last_s: -505e77,
                ratio: 0.0,
                with_trans: false,
                trans,
                tol_res: [1.0e-6; 3],
                inf: [0.0; 3],
                sup: [0.0; 3],
                x: [0.0; 3],
                r: [0.0; 3],
                status: GeomFillPipeError::PipeOk,
                poles2d: [
                    vec![Point2::default(); NB_PTS],
                    vec![Point2::default(); NB_PTS],
                ],
            }),
        })
    }

    pub(crate) fn law(&self) -> &dyn GeomFillTrihedronWithGuide {
        self.my_law.get().expect("the constructor checks the law")
    }

    pub(crate) fn guide_curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        live(&self.my_guide, "GeomFill_LocationGuide", "myGuide")
    }

    pub(crate) fn curve_of(
        &self,
        trimmed: bool,
        member: &str,
    ) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
        let fields = self.fields.read();
        let (curve, name) = if trimmed {
            (fields.my_trimmed.clone(), "myTrimmed")
        } else {
            (fields.my_curve.clone(), "myCurve")
        };
        live(&curve, member, name)?;
        Ok(curve)
    }

    /// `Set(Section, rotat, SFirst, SLast, PrecAngle, LastAngle)`: the section law,
    /// its parameter range along the path and, with `rotat`, the contact rotation;
    /// returns `LastAngle` (the last contact angle, `prec_angle` when the rotation
    /// search stops early).
    pub fn set(
        &self,
        section: &Handle<dyn GeomFillSectionLaw>,
        rotat: bool,
        s_first: f64,
        s_last: f64,
        prec_angle: f64,
    ) -> Result<f64, Failure> {
        let curve = {
            let mut f = self.fields.write();
            f.status = GeomFillPipeError::PipeOk;
            f.my_first_s = s_first;
            f.my_last_s = s_last;
            f.my_curve.clone()
        };
        let ratio = match curve.get() {
            None => 0.0,
            Some(c) => (s_last - s_first) / (c.last_parameter()? - c.first_parameter()?),
        };
        {
            let mut f = self.fields.write();
            f.ratio = ratio;
            f.my_sec = section.clone();
        }
        let mut last_angle = prec_angle;
        if rotat {
            self.set_rotation(prec_angle, &mut last_angle)?;
        } else {
            self.fields.write().rotation = false;
        }
        Ok(last_angle)
    }

    /// `EraseRotation()`: no rotation; an ImpossibleContact status is cleared.
    pub fn erase_rotation(&self) {
        let mut f = self.fields.write();
        f.rotation = false;
        if f.status == GeomFillPipeError::ImpossibleContact {
            f.status = GeomFillPipeError::PipeOk;
        }
    }

    /// `Section()`: the constant section of the section law.
    pub fn section(&self) -> Result<Handle<dyn GeomCurve>, Failure> {
        let sec = self.fields.read().my_sec.clone();
        live(&sec, "GeomFill_LocationGuide::Section", "mySec")?.constant_section()
    }

    /// `Guide()`: the (extended) guide.
    pub fn guide(&self) -> Handle<dyn Adaptor3dCurve> {
        self.my_guide.clone()
    }

    /// `SetOrigine(Param1, Param2)`: the origins given to the trihedron law.
    pub fn set_origine(&self, param1: f64, param2: f64) {
        let mut f = self.fields.write();
        f.orig_param1 = param1;
        f.orig_param2 = param2;
    }

    /// `ComputeAutomaticLaw(ParAndRad)`: the distance from the path to the guide
    /// point at `myNbPts` path parameters, `(t, distance)`; the status of the
    /// trihedron law when it fails (the rest of the array then stays null).
    pub fn compute_automatic_law(
        &self,
    ) -> Result<(GeomFillPipeError, Array1<'static, Point2>), Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationGuide::ComputeAutomaticLaw")?;
        let c = curve.get().expect("checked");
        let (f, l) = (c.first_parameter()?, c.last_parameter()?);
        let mut par_and_rad: Array1<'static, Point2> = one_based(NB_PTS as i32)?;
        for ii in 1..=NB_PTS {
            let t = sample_parameter(ii, f, l);
            let p = c.d0(t)?;
            let mut frame = TrihedronD0::default();
            if !self.law().d0(t, &mut frame)? {
                return Ok((self.law().error_status(), par_and_rad));
            }
            let cur_width = p.distance(&self.law().current_point_on_guide());
            put(&mut par_and_rad, ii as i32, Point2::new(t, cur_width))?;
        }
        Ok((GeomFillPipeError::PipeOk, par_and_rad))
    }

    fn fused_intervals(
        &self,
        shape: Shape,
        nb_sec: i32,
        nb_law: i32,
    ) -> Result<LinearVector<f64>, Failure> {
        let trimmed = self.curve_of(true, "GeomFill_LocationGuide::Intervals")?;
        let mut int_c = one_based(nb_sec + 1)?;
        let mut int_l = one_based(nb_law + 1)?;
        trimmed
            .get()
            .expect("checked")
            .intervals(&mut int_c, shape)?;
        self.law().intervals(&mut int_l, shape)?;
        let mut inter = LinearVector::with_capacity(0);
        geom_lib::fuse_intervals(
            &int_c,
            &int_l,
            &mut inter,
            precision::p_confusion() * 0.99,
            false,
        );
        Ok(inter)
    }
}

/// `t = ((myNbPts - ii) f + (ii - 1) l) / (myNbPts - 1)`.
pub(crate) fn sample_parameter(ii: usize, f: f64, l: f64) -> f64 {
    let mut t = (NB_PTS - ii) as f64 * f + (ii - 1) as f64 * l;
    t /= (NB_PTS - 1) as f64;
    t
}

impl GeomFillLocationLaw for GeomFillLocationGuide {
    /// Loads the path into the law and the trihedron law (with the origins); a
    /// rotating law recomputes its contact from the first sampled parameter.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        {
            let mut f = self.fields.write();
            f.my_curve = curve.clone();
            f.my_trimmed = curve.clone();
        }
        if curve.is_some() {
            self.law().set_curve(curve)?;
            let (o1, o2) = {
                let f = self.fields.read();
                (f.orig_param1, f.orig_param2)
            };
            self.law().origine(o1, o2)?;
            let status = self.law().error_status();
            let (rotation, first) = {
                let mut f = self.fields.write();
                f.status = status;
                (f.rotation, f.poles2d[0][0].x())
            };
            if rotation {
                let mut last_angle = 0.0;
                self.set_rotation(first, &mut last_angle)?;
            }
        }
        Ok(self.fields.read().status == GeomFillPipeError::PipeOk)
    }
    fn get_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.fields.read().my_curve.clone()
    }
    fn set_trsf(&self, transfo: &Matrix3) -> Result<(), Failure> {
        let mut f = self.fields.write();
        f.trans = *transfo;
        f.with_trans = differs_from_identity(transfo);
        Ok(())
    }
    /// A law on a copy of the trihedron law with the same origins, section and
    /// transformation; the copy has no path, so a rotating law raises from `Set`
    /// (Standard_ConstructionError), as natively.
    fn copy(&self) -> Result<Handle<dyn GeomFillLocationLaw>, Failure> {
        let law: Handle<dyn GeomFillTrihedronWithGuide> = self.law().copy()?.cast();
        let copy = GeomFillLocationGuide::new(&law)?;
        let f = self.fields.read().clone();
        copy.set_origine(f.orig_param1, f.orig_param2);
        copy.set(
            &f.my_sec,
            f.rotation,
            f.my_first_s,
            f.my_last_s,
            f.poles2d[0][0].x(),
        )?;
        copy.set_trsf(&f.trans)?;
        Ok(Handle::new(copy).cast())
    }
    fn d0(&self, param: f64, placement: &mut LocationD0) -> Result<bool, Failure> {
        self.guide_d0(param, placement)
    }
    fn d0_poles2d(
        &self,
        param: f64,
        placement: &mut LocationD0,
        _poles2d: &mut Array1<'_, Point2>,
    ) -> Result<bool, Failure> {
        self.guide_d0(param, placement)
    }
    /// The placement and its derivative; false for a rotating law.
    fn d1(
        &self,
        param: f64,
        placement: &mut LocationD1,
        _poles2d: &mut Array1<'_, Point2>,
        _d_poles2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationGuide::D1")?;
        let (p, dv) = curve.get().expect("checked").d1(param)?;
        placement.dv = dv;
        placement.v = Vector3::from(p.xyz());
        let mut f = TrihedronD1::default();
        if !self.law().d1(param, &mut f)? {
            self.fields.write().status = self.law().error_status();
            return Ok(false);
        }
        placement.m = columns(f.normal, f.binormal, f.tangent);
        placement.dm = columns(f.d_normal, f.d_binormal, f.d_tangent);
        let fields = self.fields.read().clone();
        if fields.with_trans {
            placement.m *= fields.trans;
            placement.dm *= fields.trans;
        }
        Ok(!fields.rotation)
    }
    /// Natively the transformation multiplies the output matrices before they are
    /// written, so it only shows when a rotating law answers false.
    fn d2(
        &self,
        param: f64,
        placement: &mut LocationD2,
        _poles: LocationPoles2d<'_, '_>,
    ) -> Result<bool, Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationGuide::D2")?;
        let (p, dv, d2v) = curve.get().expect("checked").d2(param)?;
        (placement.dv, placement.d2v) = (dv, d2v);
        placement.v = Vector3::from(p.xyz());
        let mut f = TrihedronD2::default();
        if !self.law().d2(param, &mut f)? {
            self.fields.write().status = self.law().error_status();
            return Ok(false);
        }
        let fields = self.fields.read().clone();
        if fields.with_trans {
            placement.m *= fields.trans;
            placement.dm *= fields.trans;
            placement.d2m *= fields.trans;
        }
        if fields.rotation {
            return Ok(false);
        }
        placement.m = columns(f.normal, f.binormal, f.tangent);
        placement.dm = columns(f.d_normal, f.d_binormal, f.d_tangent);
        placement.d2m = columns(f.d2_normal, f.d2_binormal, f.d2_tangent);
        Ok(true)
    }
    fn error_status(&self) -> GeomFillPipeError {
        self.fields.read().status
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        let trimmed = self.curve_of(true, "GeomFill_LocationGuide::NbIntervals")?;
        let nb_sec = trimmed.get().expect("checked").nb_intervals(shape)?;
        let nb_law = self.law().nb_intervals(shape)?;
        if nb_sec == 1 {
            return Ok(nb_law);
        }
        if nb_law == 1 {
            return Ok(nb_sec);
        }
        Ok(self.fused_intervals(shape, nb_sec, nb_law)?.len() as i32 - 1)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        let trimmed = self.curve_of(true, "GeomFill_LocationGuide::Intervals")?;
        let nb_sec = trimmed.get().expect("checked").nb_intervals(shape)?;
        let nb_law = self.law().nb_intervals(shape)?;
        if nb_sec == 1 {
            return self.law().intervals(bounds, shape);
        }
        if nb_law == 1 {
            return trimmed.get().expect("checked").intervals(bounds, shape);
        }
        super::frenet::copy_fusion(&self.fused_intervals(shape, nb_sec, nb_law)?, bounds)
    }
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.law().set_interval(first, last)?;
        let curve = self.curve_of(false, "GeomFill_LocationGuide::SetInterval")?;
        let trimmed = curve.get().expect("checked").trim(first, last, 0.0)?;
        self.fields.write().my_trimmed = trimmed;
        Ok(())
    }
    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        let curve = self.curve_of(true, "GeomFill_LocationGuide::GetInterval")?;
        let c = curve.get().expect("checked");
        Ok((c.first_parameter()?, c.last_parameter()?))
    }
    fn get_domain(&self) -> Result<(f64, f64), Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationGuide::GetDomain")?;
        let c = curve.get().expect("checked");
        Ok((c.first_parameter()?, c.last_parameter()?))
    }
    /// `TolRes = (guide resolution of Tol3d, Tol3d / 100, Tol3d / 100)`.
    fn set_tolerance(&self, tol3d: f64, _tol2d: f64) -> Result<(), Failure> {
        let first = self.guide_curve()?.resolution(tol3d)?;
        let (u, v) = self.resolution(1, tol3d)?;
        self.fields.write().tol_res = [first, u, v];
        Ok(())
    }
    /// `Tol / 100` in both directions.
    fn resolution(&self, _index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        Ok((tol / 100.0, tol / 100.0))
    }
    /// Always 1.
    fn get_maximal_norm(&self) -> Result<f64, Failure> {
        Ok(1.0)
    }
    /// The mean frame and the mean of 22 path points spaced by a tenth of the
    /// interval (beyond its end), as natively.
    fn get_average_law(&self) -> Result<LocationD0, Failure> {
        let frame = self.law().get_average_law()?;
        let am = columns(frame.tangent, frame.normal, frame.binormal);
        let trimmed = self.curve_of(true, "GeomFill_LocationGuide::GetAverageLaw")?;
        let c = trimmed.get().expect("checked");
        let mut av = Vector3::new(0.0, 0.0, 0.0);
        let delta = (c.last_parameter()? - c.first_parameter()?) / 10.0;
        let mut u = c.first_parameter()?;
        for _ in 0..=NB_PTS {
            av += Vector3::from(c.value(u)?.xyz());
            u += delta;
        }
        av /= (NB_PTS + 1) as f64;
        Ok(LocationD0 { m: am, v: av })
    }
    /// Always false.
    fn is_translation(&self, _error: &mut f64) -> Result<bool, Failure> {
        Ok(false)
    }
    /// Always false.
    fn is_rotation(&self, _error: &mut f64) -> Result<bool, Failure> {
        Ok(false)
    }
    /// Standard_NotImplemented.
    fn rotation(&self, _center: &mut Point3) -> Result<(), Failure> {
        Err(not_implemented("GeomFill_LocationGuide::Rotation"))
    }
}
