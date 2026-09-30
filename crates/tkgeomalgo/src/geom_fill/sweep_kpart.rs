// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of GeomFill_Sweep::BuildKPart (its translation part and the
// final trim) and the static IsSweepParallelSpine of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Sweep.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The elementary sweeps of a translation: a plane or a cylinder for a straight or
//! circular constant section, a cone for a conical section law, and an extrusion
//! otherwise; then the trim of the elementary surface to the sweep domain.
use super::frame_math::std_min;
use super::location_law::{GeomFillLocationLaw, LocationD0};
use super::section_law::GeomFillSectionLaw;
use super::sweep::GeomFillSweep;
use super::sweep_failures::live;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Direction3, GeneralTransform3, Point3, Transform3, Vector3,
};
use std::f64::consts::PI;

/// The elementary surface `BuildKPart` builds and its trim.
pub(crate) struct KPart {
    pub ok: bool,
    pub s: Handle<dyn GeomSurface>,
    pub u: (f64, f64),
    pub v: (f64, f64),
    pub u_periodic: bool,
    pub v_periodic: bool,
    pub tol: f64,
    pub error: f64,
}

/// `Tf2.SetValues(Tf(1, 1), ..., Tf(3, 4))` of the `gp_GTrsf` with the vectorial
/// part `M` and the translation `V` of a placement.
pub(crate) fn placement_trsf(p: &LocationD0) -> Result<Transform3, Failure> {
    let mut tf = GeneralTransform3::default();
    tf.set_vectorial_part(p.m);
    tf.set_translation_part(*p.v.xyz());
    let mut rows = [[0.0; 4]; 3];
    for (i, row) in rows.iter_mut().enumerate() {
        for (j, value) in row.iter_mut().enumerate() {
            *value = tf.value(i as i32 + 1, j as i32 + 1)?;
        }
    }
    let mut tf2 = Transform3::default();
    tf2.set_values(rows)?;
    Ok(tf2)
}

/// `D0` at `a` then at `b` into the same matrix, as natively: the placements.
pub(crate) fn two_placements(
    loc: &dyn GeomFillLocationLaw,
    a: f64,
    b: f64,
) -> Result<(LocationD0, LocationD0), Failure> {
    let mut at_a = LocationD0::default();
    loc.d0(a, &mut at_a)?;
    let mut at_b = LocationD0 {
        m: at_a.m,
        v: Vector3::default(),
    };
    loc.d0(b, &mut at_b)?;
    Ok((at_a, at_b))
}

/// `(2 PI - ULast, 2 PI - UFirst)`: the section range of a reversed circle.
pub(crate) fn reversed_range(u: (f64, f64)) -> (f64, f64) {
    (2.0 * PI - u.1, 2.0 * PI - u.0)
}

/// The static `IsSweepParallelSpine`: whether the move of the first section point
/// to the last section is parallel to the move of the path.
fn is_sweep_parallel_spine(
    loc: &dyn GeomFillLocationLaw,
    sec: &dyn GeomFillSectionLaw,
    tol: f64,
) -> Result<bool, Failure> {
    let (a_first, a_last) = loc.get_domain()?;
    let (begin, end) = two_placements(loc, a_first, a_last)?;
    let (tf_begin, tf_end) = (placement_trsf(&begin)?, placement_trsf(&end)?);
    let a_surf = sec.bspline_surface()?;
    let (_, _, _, vmax) = live(&a_surf, "IsSweepParallelSpine", "aSurf")?.bounds();
    let first_section = sec.constant_section()?;
    let ac_first = GeomAdaptorCurve::with_curve(&first_section)?;
    let u_first = ac_first.first_parameter()?;
    let mut l = ac_first.line()?;
    l.transform(tf_begin);
    let a_last_section = live(&a_surf, "IsSweepParallelSpine", "aSurf")?.v_iso(vmax)?;
    let last_curve = live(&a_last_section, "IsSweepParallelSpine", "aLastSection")?;
    let mut a_pnt_last_sec = last_curve.value(last_curve.first_parameter())?;
    a_pnt_last_sec.transform(tf_end);
    let a_pnt_first_sec: Point3 = elclib::value(u_first, &l);
    let a_vec_sec = Vector3::between(a_pnt_first_sec, a_pnt_last_sec);
    a_vec_sec.is_parallel(end.v - begin.v, tol)
}

impl GeomFillSweep {
    /// `BuildKPart()`: an elementary surface for a translated or rotated law with
    /// a suitable section, trimmed to the sweep domain.
    pub(crate) fn build_k_part(&mut self) -> Result<bool, Failure> {
        let mut k = KPart {
            ok: false,
            s: Handle::default(),
            u: (0.0, 0.0),
            v: (self.first, self.last),
            u_periodic: self.sec()?.is_u_periodic()?,
            v_periodic: false,
            tol: std_min(self.tol3d, self.bound_tol),
            error: 0.0,
        };
        if self.loc()?.is_translation(&mut k.error)?
            && let Some(result) = self.k_part_translation(&mut k)?
        {
            return Ok(result);
        }
        if self.loc()?.is_rotation(&mut k.error)?
            && self.sec()?.is_constant(&mut k.error)?
            && let Some(result) = self.k_part_rotation(&mut k)?
        {
            return Ok(result);
        }
        if k.ok {
            self.trim_k_part(k)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// The translation: `Some(false)` when the placement is not a rigid motion or
    /// a straight section does not follow the path.
    fn k_part_translation(&mut self, k: &mut KPart) -> Result<Option<bool>, Failure> {
        let (at1, at0) = two_placements(self.loc()?, 1.0, 0.0)?;
        let mut dp = at1.v - at0.v;
        dp.normalize()?;
        let tf2 = match placement_trsf(&at0) {
            Ok(tf2) => tf2,
            Err(e) if e.exception_type() == "Standard_ConstructionError" => return Ok(Some(false)),
            Err(e) => return Err(e),
        };
        if self.sec()?.is_constant(&mut k.error)? {
            return self.translated_constant(k, tf2, dp);
        }
        if self.sec()?.is_conical_law(&mut k.error)? {
            self.translated_conical(k, tf2, dp)?;
        }
        Ok(None)
    }

    fn translated_constant(
        &mut self,
        k: &mut KPart,
        tf2: Transform3,
        dp: Vector3,
    ) -> Result<Option<bool>, Failure> {
        let section = self.sec()?.constant_section()?;
        let ac = GeomAdaptorCurve::with_curve(&section)?;
        let section_type = ac.get_type()?;
        k.u = (ac.first_parameter()?, ac.last_parameter()?);
        if section_type == CurveType::Line {
            if !is_sweep_parallel_spine(self.loc()?, self.sec()?, k.tol)? {
                return Ok(Some(false));
            }
            self.translated_line(k, &ac, tf2, dp)?;
        }
        if section_type == CurveType::Circle {
            self.translated_circle(k, &ac, tf2, dp)?;
        }
        if !k.ok {
            live(&section, "GeomFill_Sweep::BuildKPart", "Section")?.transform(tf2)?;
            let extrusion =
                GeomSurfaceOfLinearExtrusion::new(&section, Direction3::from_vector(dp)?)?;
            k.s = Handle::new(extrusion).cast();
            self.s_error = 0.0;
            k.ok = true;
        }
        Ok(None)
    }

    /// A straight section: the plane of the section and the path direction.
    fn translated_line(
        &mut self,
        k: &mut KPart,
        ac: &GeomAdaptorCurve,
        tf2: Transform3,
        dp: Vector3,
    ) -> Result<(), Failure> {
        let mut l = ac.line()?;
        l.transform(tf2);
        let mut ds = Vector3::from(l.position().direction());
        ds.normalize()?;
        let levier = ds.dot(dp).abs();
        self.s_error = k.error + levier * (self.last - self.first).abs();
        if self.s_error <= k.tol {
            k.ok = true;
            let normal = Direction3::from_vector(ds.crossed(dp))?;
            let x = Direction3::from_vector(ds)?;
            let axis_of_plane = Axis2::with_x_direction(l.location(), normal, x)?;
            k.s = Handle::new(GeomPlane::new(Axis3::from_axis2(axis_of_plane))).cast();
            return Ok(());
        }
        self.s_error = 0.0;
        Ok(())
    }

    /// A circular section normal to the path: a cylinder.
    fn translated_circle(
        &mut self,
        k: &mut KPart,
        ac: &GeomAdaptorCurve,
        tf2: Transform3,
        dp: Vector3,
    ) -> Result<(), Failure> {
        let tol_prod = 1.0e-6;
        let mut c = ac.circle()?;
        c.transform(tf2)?;
        let mut ds = Vector3::from(c.position().direction());
        ds.normalize()?;
        let levier = ds.cross_magnitude(dp).abs() * c.radius();
        self.s_error = levier * (self.last - self.first).abs();
        if self.s_error <= tol_prod {
            k.ok = true;
            let dp_dir = Direction3::from_vector(dp)?;
            let axe = Axis3::new(c.location(), dp_dir, c.position().x_direction())?;
            k.s = Handle::new(GeomCylindricalSurface::new(axe, c.radius())?).cast();
            if c.position().direction().is_opposite(axe.direction(), 0.1) {
                k.u = reversed_range(k.u);
                self.is_u_reversed = true;
            }
            return Ok(());
        }
        self.s_error = 0.0;
        Ok(())
    }

    /// `CirclSection(param)` moved by `Tf2` and translated by `shift`, loaded in `ac`.
    fn placed_circle(
        &self,
        ac: &GeomAdaptorCurve,
        param: f64,
        tf2: Transform3,
        shift: Option<Vector3>,
    ) -> Result<(), Failure> {
        let section: Handle<dyn GeomCurve> = self.sec()?.circl_section(param)?;
        let s = live(&section, "GeomFill_Sweep::BuildKPart", "Section")?;
        s.transform(tf2)?;
        if let Some(shift) = shift {
            s.translate(shift)?;
        }
        ac.load(&section)
    }

    /// A conical section law: a cone through the circles at both ends when their
    /// chord makes a proper angle with the axis.
    fn translated_conical(
        &mut self,
        k: &mut KPart,
        tf2: Transform3,
        dp: Vector3,
    ) -> Result<(), Failure> {
        let ac = GeomAdaptorCurve::default();
        self.placed_circle(&ac, self.s_last, tf2, Some(dp * self.last))?;
        let c = ac.circle()?;
        let (centre2, r2) = (c.location(), c.radius());
        let (p2, dsection) = ac.d1(0.0)?;
        self.placed_circle(&ac, self.s_first, tf2, Some(dp * self.first))?;
        let c = ac.circle()?;
        let (centre1, p1, r1) = (c.location(), ac.value(0.0)?, c.radius());
        let ratio = (self.s_last - self.s_first) / (self.last - self.first);
        self.placed_circle(&ac, self.s_first - self.first * ratio, tf2, None)?;
        let c = ac.circle()?;
        let centre0 = c.location();
        let mut n = Vector3::between(centre1, p1);
        if n.magnitude() < 1.0e-9 {
            n = Vector3::between(centre2, p2);
        }
        let (l, dir) = (Vector3::between(p1, p2), Vector3::between(centre1, centre2));
        let mut angle = l.angle(dir)?;
        let proper = angle > 0.01 && angle < PI / 2.0 - 0.01;
        if !proper {
            return Ok(());
        }
        if r2 < r1 {
            angle = -angle;
        }
        self.s_error = k.error;
        let axis = Axis3::new(
            centre0,
            Direction3::from_vector(dir)?,
            Direction3::from_vector(n)?,
        )?;
        let s: Handle<dyn GeomSurface> =
            Handle::new(GeomConicalSurface::new(axis, angle, c.radius())?).cast();
        k.v = (self.first / angle.cos(), self.last / angle.cos());
        k.u = (ac.first_parameter()?, ac.last_parameter()?);
        let iso = live(&s, "GeomFill_Sweep::BuildKPart", "S")?.v_iso(k.v.1)?;
        let (_, diso) = live(&iso, "GeomFill_Sweep::BuildKPart", "VIso")?.d1(0.0)?;
        if diso.magnitude() > 1.0e-9 && dsection.magnitude() > 1.0e-9 {
            self.is_u_reversed = diso.is_opposite(dsection, 0.1)?;
        }
        if self.is_u_reversed {
            k.u = reversed_range(k.u);
        }
        k.ok = true;
        k.s = s;
        Ok(())
    }

    /// The trim of the elementary surface (U and V exchanged first when the path is
    /// the U direction); a periodic direction is not trimmed.
    fn trim_k_part(&mut self, mut k: KPart) -> Result<(), Failure> {
        if self.my_exch_uv {
            (k.u_periodic, k.v_periodic) = (k.v_periodic, k.u_periodic);
            (k.u, k.v) = (k.v, k.u);
        }
        self.my_surface = match (k.u_periodic, k.v_periodic) {
            (false, false) => Handle::new(GeomRectangularTrimmedSurface::new(
                &k.s,
                k.u,
                k.v,
                (true, true),
            )?)
            .cast(),
            (true, true) => k.s,
            (true, false) => Handle::new(GeomRectangularTrimmedSurface::with_one_direction(
                &k.s, k.v.0, k.v.1, false, true,
            )?)
            .cast(),
            (false, true) => Handle::new(GeomRectangularTrimmedSurface::with_one_direction(
                &k.s, k.u.0, k.u.1, true, true,
            )?)
            .cast(),
        };
        Ok(())
    }
}
