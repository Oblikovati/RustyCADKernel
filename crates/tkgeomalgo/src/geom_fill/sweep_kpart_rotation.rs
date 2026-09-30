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
// Rust translation of the rotation part of GeomFill_Sweep::BuildKPart of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Sweep.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The elementary sweeps of a rotation with a constant section: a sphere or a torus
//! for a circle in the rotation plane, a cylinder or a cone for a line in a plane
//! of the axis, and a surface of revolution otherwise.
use super::frame_math::std_max;
use super::sweep::GeomFillSweep;
use super::sweep_failures::live;
use super::sweep_kpart::{KPart, placement_trsf, reversed_range, two_placements};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomGeometry, GeomLine,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution, GeomToroidalSurface,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{
    self, Axis1, Axis3, Circle3, Direction3, Line3, Point3, Sphere3, Transform3, Vector3,
};
use std::f64::consts::{FRAC_PI_2, PI};

/// The rotation frame: the centre (moved onto the circle plane by a circular
/// section), the radial `DS`, the axis `DN`, the tangential `DP` and `Tf2`.
struct RotationFrame {
    centre: Point3,
    ds: Vector3,
    dn: Vector3,
    dp: Vector3,
    tf2: Transform3,
}

/// The constant section and its adaptor.
struct ConstantSection {
    section: Handle<dyn GeomCurve>,
    ac: GeomAdaptorCurve,
}

/// The distance of the circle centre from the axis (after moving the frame
/// centre into the circle plane), the radial vector (the frame one for a centre on
/// the axis) and whether the centre is on the side of the path.
fn circle_offset(c: &Circle3, f: &mut RotationFrame, tol: f64) -> (f64, Vector3, bool) {
    let dc = Vector3::from(c.location().xyz() - f.centre.xyz());
    f.centre = Point3::from(f.centre.xyz() + dc.dot(f.dn) * *f.dn.xyz());
    let mut dc = Vector3::from(c.location().xyz() - f.centre.xyz());
    let radius = dc.magnitude();
    let is_good_side = !(radius > tol && dc.dot(f.ds) < 0.0);
    if radius < tol / 100.0 {
        dc = f.ds;
    }
    (radius, dc, is_good_side)
}

/// How far the circle plane is from holding the axis, scaled by the radius.
fn circle_plane_error(c: &Circle3, f: &RotationFrame) -> Result<f64, Failure> {
    let mut nc = Vector3::from(c.position().direction());
    nc.normalize()?;
    let mut error = nc.dot(f.dn).abs();
    error += nc.dot(f.ds).abs();
    error *= c.radius();
    Ok(error)
}

/// `((UFirst, ULast), (VFirst, VLast))` on a sphere.
type SphereRange = ((f64, f64), (f64, f64));

/// The sphere parameters `((UFirst, ULast), (VFirst, VLast))` of the ends of the
/// section moved by `tf2`.
fn section_on_sphere(
    cs: &ConstantSection,
    tf2: Transform3,
    sphere: &Sphere3,
) -> Result<SphereRange, Failure> {
    let (fpar, lpar) = (cs.ac.first_parameter()?, cs.ac.last_parameter()?);
    let the_section = GeomTrimmedCurve::new(&cs.section, fpar, lpar, true, true)?;
    the_section.transform(tf2)?;
    let first_point = the_section.value(the_section.first_parameter())?;
    let last_point = the_section.value(the_section.last_parameter())?;
    let (u1, v1) = elslib::parameters(sphere, first_point);
    let (u2, v2) = elslib::parameters(sphere, last_point);
    Ok(((u1, u2), (v1, v2)))
}

/// The frame of the U iso circle at 0 of a torus.
fn first_u_iso_frame(s: &Handle<dyn GeomSurface>) -> Result<rustycad_tkmath::gp::Axis2, Failure> {
    let iso: Handle<GeomCircle> = live(s, "GeomFill_Sweep::BuildKPart", "S")?
        .u_iso(0.0)?
        .cast();
    Ok(live(&iso, "GeomFill_Sweep::BuildKPart", "Iso")?
        .circ()?
        .position())
}

impl GeomFillSweep {
    /// The rotation part of `BuildKPart`; `Some(false)` for a degenerate rotation
    /// or a spherical zone the sphere cannot hold.
    pub(crate) fn k_part_rotation(&mut self, k: &mut KPart) -> Result<Option<bool>, Failure> {
        k.v_periodic = (self.last - self.first - 2.0 * PI).abs() < 1.0e-15;
        let Some(mut frame) = self.rotation_frame()? else {
            return Ok(Some(false));
        };
        let section = self.sec()?.constant_section()?;
        let ac = GeomAdaptorCurve::with_curve(&section)?;
        let section_type = ac.get_type()?;
        k.u = (ac.first_parameter()?, ac.last_parameter()?);
        let cs = ConstantSection { section, ac };
        if section_type == CurveType::Circle
            && let Some(result) = self.rotated_circle(k, &cs, &mut frame)?
        {
            return Ok(Some(result));
        }
        if section_type == CurveType::Line {
            self.rotated_line(k, &cs, &mut frame)?;
        }
        if !k.ok {
            live(&cs.section, "GeomFill_Sweep::BuildKPart", "Section")?.transform(frame.tf2)?;
            let axis = Axis1::new(frame.centre, Direction3::from_vector(frame.dn)?);
            k.s = Handle::new(GeomSurfaceOfRevolution::new(&cs.section, axis)?).cast();
            self.my_exch_uv = true;
            self.s_error = 0.0;
            k.ok = true;
        }
        Ok(None)
    }

    /// The frame of the rotation from the placements at 0.1 and 0, or `None` when
    /// the path point is on the axis.
    fn rotation_frame(&self) -> Result<Option<RotationFrame>, Failure> {
        let loc = self.loc()?;
        let (at_tenth, at0) = two_placements(loc, 0.1, 0.0)?;
        let mut centre = Point3::default();
        loc.rotation(&mut centre)?;
        let dp = at_tenth.v - at0.v;
        let mut ds = Vector3::from(*at0.v.xyz() - centre.xyz());
        if ds.magnitude() > 1.0e-15 {
            ds.normalize()?;
        } else {
            return Ok(None);
        }
        let mut dn = ds.crossed(dp);
        dn.normalize()?;
        let mut dp = dn.crossed(ds);
        dp.normalize()?;
        let tf2 = placement_trsf(&at0)?;
        Ok(Some(RotationFrame {
            centre,
            ds,
            dn,
            dp,
            tf2,
        }))
    }

    /// A circular section: a sphere when its centre is on the axis, a torus when
    /// it is on the good side, if the circle plane holds the axis.
    fn rotated_circle(
        &mut self,
        k: &mut KPart,
        cs: &ConstantSection,
        f: &mut RotationFrame,
    ) -> Result<Option<bool>, Failure> {
        let mut c = cs.ac.circle()?;
        c.transform(f.tf2)?;
        let (radius, dc, is_good_side) = circle_offset(&c, f, k.tol);
        k.error = circle_plane_error(&c, f)?;
        let within = k.error <= k.tol;
        if !within {
            self.s_error = 0.0;
            return Ok(None);
        }
        self.s_error = k.error;
        k.error += radius;
        if radius <= k.tol {
            return self.spherical(k, cs, f, &c);
        }
        if is_good_side {
            self.toroidal(k, f, &c, radius, dc)?;
        }
        Ok(None)
    }

    /// The sphere through the circle, with the section as its V range.
    fn spherical(
        &mut self,
        k: &mut KPart,
        cs: &ConstantSection,
        f: &mut RotationFrame,
        c: &Circle3,
    ) -> Result<Option<bool>, Failure> {
        self.s_error = k.error;
        f.centre.barycenter(1.0, c.location(), 1.0);
        let dn = Direction3::from_vector(f.dn)?;
        let axis_of_sphere = Axis3::new(f.centre, dn, Direction3::from_vector(f.ds)?)?;
        let the_sphere = Sphere3::new(axis_of_sphere, c.radius())?;
        k.s = Handle::new(GeomSphericalSurface::from_sphere(the_sphere)).cast();
        let (u_on_sec, (v_first, v_last)) = section_on_sphere(cs, f.tf2, &the_sphere)?;
        let (fv, lv) = if v_first < v_last {
            (v_first, v_last)
        } else {
            self.is_u_reversed = true;
            (v_last, v_first)
        };
        if (lv - fv).abs() <= precision::p_confusion()
            || (u_on_sec.1 - u_on_sec.0).abs() > FRAC_PI_2
        {
            self.is_u_reversed = false;
            return Ok(Some(k.ok));
        }
        if fv >= -PI / 2.0 && lv <= PI / 2.0 {
            k.ok = true;
            self.my_exch_uv = true;
            k.u = (fv, lv);
        } else {
            self.is_u_reversed = false;
        }
        Ok(None)
    }

    /// The torus of the circle around the axis.
    fn toroidal(
        &mut self,
        k: &mut KPart,
        f: &RotationFrame,
        c: &Circle3,
        radius: f64,
        dc: Vector3,
    ) -> Result<(), Failure> {
        let dn = Direction3::from_vector(f.dn)?;
        let axis_of_tore = Axis3::new(f.centre, dn, Direction3::from_vector(dc)?)?;
        let s: Handle<dyn GeomSurface> =
            Handle::new(GeomToroidalSurface::new(axis_of_tore, radius, c.radius())?).cast();
        let axeiso = first_u_iso_frame(&s)?;
        if c.position()
            .direction()
            .is_opposite(axeiso.direction(), 0.1)
        {
            k.u = reversed_range(k.u);
            self.is_u_reversed = true;
        }
        let rot = c
            .position()
            .x_direction()
            .angle_with_ref(axeiso.x_direction(), axeiso.direction());
        k.u = (k.u.0 - rot, k.u.1 - rot);
        self.my_exch_uv = true;
        if k.u_periodic && k.u.0.abs() > precision::p_confusion() {
            k.u_periodic = false;
        }
        k.ok = true;
        k.s = s;
        Ok(())
    }

    /// A straight section in a plane of the axis: a cylinder when parallel to the
    /// axis, else a cone.
    fn rotated_line(
        &mut self,
        k: &mut KPart,
        cs: &ConstantSection,
        f: &mut RotationFrame,
    ) -> Result<(), Failure> {
        let mut l = cs.ac.line()?;
        l.transform(f.tf2);
        let dl = Vector3::from(l.direction());
        let levier = std_max(cs.ac.first_parameter()?.abs(), cs.ac.last_parameter()?);
        self.s_error = k.error + levier * dl.dot(f.dp).abs();
        if self.s_error <= k.tol {
            let reverse = dl.dot(f.dn) < 0.0;
            self.revolved_line(k, f, &l, dl, levier)?;
            if k.ok && reverse {
                let cl = GeomLine::from_lin(l);
                let uf = cl.reversed_parameter(k.u.1)?;
                let ul = cl.reversed_parameter(k.u.0)?;
                k.u = (ul, uf);
            }
            return Ok(());
        }
        self.s_error = 0.0;
        Ok(())
    }

    fn revolved_line(
        &mut self,
        k: &mut KPart,
        f: &mut RotationFrame,
        l: &Line3,
        dl: Vector3,
        levier: f64,
    ) -> Result<(), Failure> {
        let dir = Line3::new(f.centre, Direction3::from_vector(f.dn)?);
        let o1o2 = Vector3::between(f.centre, l.location());
        let mut trans = f.dn;
        trans *= f.dn.dot(o1o2);
        let centre_of_surf = f.centre.translated(trans);
        f.ds = Vector3::from(l.location().xyz() - centre_of_surf.xyz());
        k.error = self.s_error;
        k.error += dl.xyz().cross_magnitude(*f.dn.xyz()) * levier;
        if k.error <= k.tol {
            self.s_error = k.error;
            let mut axis = Axis3::from_normal(centre_of_surf, dir.direction())?;
            if f.ds.square_magnitude() > gp::resolution() {
                axis.set_x_direction(Direction3::from_vector(f.ds)?)?;
            }
            let cylinder = GeomCylindricalSurface::new(axis, l.distance(centre_of_surf))?;
            k.s = Handle::new(cylinder).cast();
            (k.ok, self.my_exch_uv) = (true, true);
            return Ok(());
        }
        self.conical_line(k, f, l, dl, &dir, centre_of_surf)
    }

    fn conical_line(
        &mut self,
        k: &mut KPart,
        f: &RotationFrame,
        l: &Line3,
        dl: Vector3,
        dir: &Line3,
        centre_of_surf: Point3,
    ) -> Result<(), Failure> {
        let mut angle = dir.angle(*l).abs();
        if angle > PI / 2.0 {
            angle = PI - angle;
        }
        if dl.dot(f.dn) < 0.0 {
            angle = -angle;
        }
        if f.ds.dot(dl) < 0.0 {
            angle = -angle;
        }
        if (angle.abs() - PI / 2.0).abs() > 0.01 {
            let radius = centre_of_surf.distance(&l.location());
            let axis = Axis3::new(
                centre_of_surf,
                dir.direction(),
                Direction3::from_vector(f.ds)?,
            )?;
            k.s = Handle::new(GeomConicalSurface::new(axis, angle, radius)?).cast();
            (k.ok, self.my_exch_uv) = (true, true);
            return Ok(());
        }
        self.s_error = 0.0;
        Ok(())
    }
}
