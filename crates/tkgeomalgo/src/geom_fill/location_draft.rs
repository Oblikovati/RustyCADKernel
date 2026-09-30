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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_LocationDraft.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_LocationDraft`: the location law of a draft along a path, with the
//! optional trace of the draft generatrices on a stop surface (the evaluators are in
//! `location_draft_eval.rs`).
use super::draft_trihedron::GeomFillDraftTrihedron;
use super::location_helpers::{average_placement, differs_from_identity};
use super::location_law::{GeomFillLocationLaw, LocationD0};
use super::sweep_failures::live;
use super::trihedron_law::{GeomFillTrihedronLaw, TrihedronD0};
use crate::int_curve_surface::HInter;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Direction3, Matrix3, Point2, Point3, Vector3};

/// The number of sampled path points (`myNbPts`).
pub(crate) const NB_PTS: i32 = 41;

/// The fields the setters and `Prepare` write.
#[derive(Clone)]
pub(crate) struct DraftFields {
    pub trans: Matrix3,
    pub my_surf: Handle<dyn Adaptor3dSurface>,
    pub my_curve: Handle<dyn Adaptor3dCurve>,
    pub my_trimmed: Handle<dyn Adaptor3dCurve>,
    pub my_angle: f64,
    pub intersec: bool,
    pub with_trans: bool,
    /// `myPoles2d` (`1..2 * myNbPts`): `(w, t)` of the generatrix at odd indices,
    /// `(u, v)` on the stop surface at even ones.
    pub poles2d: Vec<Point2>,
}

/// `GeomFill_LocationDraft`: the draft location law.
/// `let law = GeomFillLocationDraft::new(dir, 0.1)?; law.set_curve(&path)?;`.
pub struct GeomFillLocationDraft {
    parent: StandardTransient,
    pub(crate) my_law: Handle<GeomFillDraftTrihedron>,
    my_dir: Direction3,
    pub(crate) fields: SharedState<DraftFields>,
}
impl_transient!(
    GeomFillLocationDraft,
    "GeomFill_LocationDraft",
    dyn GeomFillLocationLaw,
    parent,
    interfaces: [dyn GeomFillLocationLaw]
);

/// `D = cos(angle) B + sin(angle) N`: the direction of the draft generatrix.
pub(crate) fn generatrix_direction(frame: &TrihedronD0, angle: f64) -> Vector3 {
    angle.cos() * frame.binormal + angle.sin() * frame.normal
}

/// The adaptor of the line through `p` along `d` (`gp_Dir(D)` normalizes it).
pub(crate) fn generatrix(p: Point3, d: Vector3) -> Result<Handle<dyn Adaptor3dCurve>, Failure> {
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(p, Direction3::from_vector(d)?)).cast();
    Ok(Handle::new(GeomAdaptorCurve::with_curve(&line)?).cast())
}

impl GeomFillLocationDraft {
    /// `GeomFill_LocationDraft(Direction, Angle)`: no stop surface; `Trans` starts
    /// as the null matrix (the native `gp_Mat` default), unused until `SetTrsf`.
    pub fn new(direction: Direction3, angle: f64) -> Result<Self, Failure> {
        Ok(Self {
            parent: StandardTransient::default(),
            my_law: Handle::new(GeomFillDraftTrihedron::new(
                Vector3::from(direction),
                angle,
            )?),
            my_dir: direction,
            fields: SharedState::new(DraftFields {
                trans: Matrix3::default(),
                my_surf: Handle::default(),
                my_curve: Handle::default(),
                my_trimmed: Handle::default(),
                my_angle: angle,
                intersec: false,
                with_trans: false,
                poles2d: vec![Point2::default(); 2 * NB_PTS as usize],
            }),
        })
    }

    pub(crate) fn law(&self) -> &GeomFillDraftTrihedron {
        self.my_law
            .get()
            .expect("the constructor creates the draft law")
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

    /// `SetStopSurf(Surf)`: the surface the generatrices stop on, then `Prepare`.
    pub fn set_stop_surf(&self, surf: &Handle<dyn Adaptor3dSurface>) -> Result<(), Failure> {
        self.fields.write().my_surf = surf.clone();
        self.prepare()
    }

    /// `SetAngle(Angle)`: the draft angle of the law and its trihedron, then
    /// `Prepare`.
    pub fn set_angle(&self, angle: f64) -> Result<(), Failure> {
        self.fields.write().my_angle = angle;
        self.law().set_angle(angle);
        self.prepare()
    }

    /// `IsIntersec()`: whether every sampled generatrix meets the stop surface.
    pub fn is_intersec(&self) -> bool {
        self.fields.read().intersec
    }

    /// `Direction()`: the draft direction.
    pub fn direction(&self) -> Direction3 {
        self.my_dir
    }

    /// `Prepare()`: the intersections of the generatrices at `myNbPts` path points
    /// with the stop surface (the nearest along each generatrix).
    fn prepare(&self) -> Result<(), Failure> {
        let surf = self.fields.read().my_surf.clone();
        if surf.is_null() {
            self.fields.write().intersec = false;
            return Ok(());
        }
        self.fields.write().intersec = true;
        let curve = self.curve_of(false, "GeomFill_LocationDraft::Prepare")?;
        let c = curve.get().expect("checked");
        let (f, l) = (c.first_parameter()?, c.last_parameter()?);
        let angle = self.fields.read().my_angle;
        for ii in 1..=NB_PTS {
            let mut t = f64::from(NB_PTS - ii) * f + f64::from(ii - 1) * l;
            t /= f64::from(NB_PTS - 1);
            let p = c.d0(t)?;
            let mut frame = TrihedronD0::default();
            self.law().d0(t, &mut frame)?;
            let line = generatrix(p, generatrix_direction(&frame, angle))?;
            self.store_trace(ii, t, &line, &surf)?;
        }
        Ok(())
    }

    /// The nearest intersection of one generatrix, stored at `2 ii - 1` and `2 ii`,
    /// or `Intersec = false`.
    fn store_trace(
        &self,
        ii: i32,
        t: f64,
        line: &Handle<dyn Adaptor3dCurve>,
        surf: &Handle<dyn Adaptor3dSurface>,
    ) -> Result<(), Failure> {
        let mut int = HInter::new();
        int.perform(line, surf)?;
        if int.nb_points()? == 0 {
            self.fields.write().intersec = false;
            return Ok(());
        }
        let mut p1 = int.point(1)?;
        for jj in 2..=int.nb_points()? {
            let p2 = int.point(jj)?;
            if p1.w() > p2.w() {
                p1 = p2;
            }
        }
        let mut fields = self.fields.write();
        fields.poles2d[(2 * ii - 2) as usize] = Point2::new(p1.w(), t);
        fields.poles2d[(2 * ii - 1) as usize] = Point2::new(p1.u(), p1.v());
        Ok(())
    }

    fn trimmed_bounds(&self, trimmed: bool, member: &str) -> Result<(f64, f64), Failure> {
        let curve = self.curve_of(trimmed, member)?;
        let c = curve.get().expect("checked");
        Ok((c.first_parameter()?, c.last_parameter()?))
    }
}

impl GeomFillLocationLaw for GeomFillLocationDraft {
    /// Loads the path into the law and its trihedron, then `Prepare`.
    fn set_curve(&self, curve: &Handle<dyn Adaptor3dCurve>) -> Result<bool, Failure> {
        {
            let mut fields = self.fields.write();
            fields.my_curve = curve.clone();
            fields.my_trimmed = curve.clone();
        }
        let is_ok = self.law().set_curve(curve)?;
        self.prepare()?;
        Ok(is_ok)
    }
    fn get_curve(&self) -> Handle<dyn Adaptor3dCurve> {
        self.fields.read().my_curve.clone()
    }
    fn set_trsf(&self, transfo: &Matrix3) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        fields.trans = *transfo;
        fields.with_trans = differs_from_identity(transfo);
        Ok(())
    }
    /// A new draft law of the same direction and angle on the same path and stop
    /// surface (the trihedron law is copied once, unused, as natively).
    fn copy(&self) -> Result<Handle<dyn GeomFillLocationLaw>, Failure> {
        let _unused_copy = self.law().copy()?;
        let fields = self.fields.read().clone();
        let copy = GeomFillLocationDraft::new(self.my_dir, fields.my_angle)?;
        copy.set_curve(&fields.my_curve)?;
        copy.set_stop_surf(&fields.my_surf)?;
        if fields.with_trans {
            copy.set_trsf(&fields.trans)?;
        }
        Ok(Handle::new(copy).cast())
    }
    fn d0(&self, param: f64, placement: &mut LocationD0) -> Result<bool, Failure> {
        self.draft_d0(param, placement)
    }
    fn d0_poles2d(
        &self,
        param: f64,
        placement: &mut LocationD0,
        poles2d: &mut Array1<'_, Point2>,
    ) -> Result<bool, Failure> {
        self.draft_d0_poles2d(param, placement, poles2d)
    }
    fn d1(
        &self,
        param: f64,
        placement: &mut super::location_law::LocationD1,
        poles2d: &mut Array1<'_, Point2>,
        d_poles2d: &mut Array1<'_, rustycad_tkmath::gp::Vector2>,
    ) -> Result<bool, Failure> {
        self.draft_d1(param, placement, poles2d, d_poles2d)
    }
    fn d2(
        &self,
        param: f64,
        placement: &mut super::location_law::LocationD2,
        poles: super::location_law::LocationPoles2d<'_, '_>,
    ) -> Result<bool, Failure> {
        self.draft_d2(param, placement, poles)
    }
    /// False.
    fn has_first_restriction(&self) -> bool {
        false
    }
    /// `Intersec`.
    fn has_last_restriction(&self) -> bool {
        self.fields.read().intersec
    }
    /// 1 when the generatrices meet the stop surface, else 0.
    fn trace_number(&self) -> i32 {
        i32::from(self.fields.read().intersec)
    }
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        self.law().nb_intervals(shape)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure> {
        self.law().intervals(bounds, shape)
    }
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure> {
        self.law().set_interval(first, last)?;
        let curve = self.curve_of(false, "GeomFill_LocationDraft::SetInterval")?;
        let trimmed = curve.get().expect("checked").trim(first, last, 0.0)?;
        self.fields.write().my_trimmed = trimmed;
        Ok(())
    }
    fn get_interval(&self) -> Result<(f64, f64), Failure> {
        self.trimmed_bounds(true, "GeomFill_LocationDraft::GetInterval")
    }
    fn get_domain(&self) -> Result<(f64, f64), Failure> {
        self.trimmed_bounds(false, "GeomFill_LocationDraft::GetDomain")
    }
    /// The surface resolutions for the trace (index 1), `tol` otherwise.
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        if index != 1 {
            return Ok((tol, tol));
        }
        let surf = self.fields.read().my_surf.clone();
        let s = live(&surf, "GeomFill_LocationDraft::Resolution", "mySurf")?;
        Ok((s.u_resolution(tol)?, s.v_resolution(tol)?))
    }
    /// Always 1.
    fn get_maximal_norm(&self) -> Result<f64, Failure> {
        Ok(1.0)
    }
    fn get_average_law(&self) -> Result<LocationD0, Failure> {
        let frame = self.law().get_average_law()?;
        let trimmed = self.curve_of(true, "GeomFill_LocationDraft::GetAverageLaw")?;
        average_placement(frame, trimmed.get().expect("checked"))
    }
    /// Whether the trihedron law is constant (`Error` untouched).
    fn is_translation(&self, _error: &mut f64) -> Result<bool, Failure> {
        self.law().is_constant()
    }
    /// A circle path with a curve-only trihedron law; `Error = 0`.
    fn is_rotation(&self, error: &mut f64) -> Result<bool, Failure> {
        *error = 0.0;
        let curve = self.curve_of(false, "GeomFill_LocationDraft::IsRotation")?;
        if curve.get().expect("checked").get_type()? == CurveType::Circle {
            return self.law().is_only_by_3d_curve();
        }
        Ok(false)
    }
    /// The centre of the circle path.
    fn rotation(&self, center: &mut Point3) -> Result<(), Failure> {
        let curve = self.curve_of(false, "GeomFill_LocationDraft::Rotation")?;
        *center = curve.get().expect("checked").circle()?.location();
        Ok(())
    }
}
