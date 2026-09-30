// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of GeomFill_Pipe::KPartT4 of
// ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Pipe.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The particular cases of the blend between two rails: a cylinder between three
//! parallel lines of the same length, a torus between three coaxial circles of the
//! same range.
use super::pipe::GeomFillPipe;
use super::pipe_error::GeomFillPipeError;
use super::sweep_failures::live;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{
    GeomCylindricalSurface, GeomRectangularTrimmedSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Line3, Point3, Torus3, Vector3};
use std::f64::consts::PI;

/// The path and the two rails.
struct Rails<'a> {
    path: &'a dyn Adaptor3dCurve,
    first: &'a dyn Adaptor3dCurve,
    last: &'a dyn Adaptor3dCurve,
}

impl Rails<'_> {
    fn all(&self) -> [&dyn Adaptor3dCurve; 3] {
        [self.path, self.first, self.last]
    }

    fn all_of(&self, kind: CurveType) -> Result<bool, Failure> {
        for c in self.all() {
            if c.get_type()? != kind {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The path range.
    fn path_range(&self) -> Result<(f64, f64), Failure> {
        Ok((self.path.first_parameter()?, self.path.last_parameter()?))
    }

    /// The start points of the curves.
    fn starts(&self) -> Result<[Point3; 3], Failure> {
        let mut p = [Point3::default(); 3];
        for (slot, c) in p.iter_mut().zip(self.all()) {
            *slot = c.value(c.first_parameter()?)?;
        }
        Ok(p)
    }
}

/// `D0.IsEqual(D1) && D1.IsEqual(D2)` within Precision::Angular().
fn same_directions(d: [Direction3; 3]) -> bool {
    let ang = precision::angular();
    d[0].is_equal(&d[1], ang) && d[1].is_equal(&d[2], ang)
}

/// The unit vectors from the path start to the rail starts.
fn start_directions(p: [Point3; 3]) -> Result<(Direction3, Direction3), Failure> {
    Ok((
        Direction3::from_vector(Vector3::between(p[0], p[1]))?,
        Direction3::from_vector(Vector3::between(p[0], p[2]))?,
    ))
}

impl GeomFillPipe {
    /// `KPartT4()`: true when the blend is a cylinder or a torus (then built). The
    /// rails are read only when the path has the kind tested, as natively.
    pub(crate) fn k_part_t4(&mut self) -> Result<bool, Failure> {
        let member = "GeomFill_Pipe::KPartT4";
        let (path, first, last) = (
            self.my_adp_path.clone(),
            self.my_adp_first_sect.clone(),
            self.my_adp_last_sect.clone(),
        );
        let p = live(&path, member, "myAdpPath")?;
        let kind = p.get_type()?;
        if kind != CurveType::Line && kind != CurveType::Circle {
            return Ok(false);
        }
        let rails = Rails {
            path: p,
            first: live(&first, member, "myAdpFirstSect")?,
            last: live(&last, member, "myAdpLastSect")?,
        };
        if !rails.all_of(kind)? {
            return Ok(false);
        }
        if kind == CurveType::Line {
            return self.cylinder_between(&rails);
        }
        self.torus_between(&rails)
    }

    /// Three parallel lines of the same length, the rails normal to the path: the
    /// cylinder of `myRadius` around the path between the rails.
    fn cylinder_between(&mut self, rails: &Rails<'_>) -> Result<bool, Failure> {
        let Some(f) = cylinder_frame(rails)? else {
            return Ok(false);
        };
        let z_ref = f.v1.crossed(f.v2)?;
        let mut axis = Axis3::new(f.axis.location(), f.axis.direction(), f.v1)?;
        if z_ref.dot(f.axis.direction()) < 0.0 {
            axis.y_reverse();
        }
        axis.rotate(Axis1::new(f.p0, z_ref), -PI / 2.0)?;
        let cylinder: Handle<dyn GeomSurface> =
            Handle::new(GeomCylindricalSurface::new(axis, self.my_radius)?).cast();
        // The native code stores the cylinder before trimming it.
        self.my_surface = cylinder.clone();
        let alpha = f.v1.angle_with_ref(f.v2, z_ref);
        let u = (PI / 2.0, PI / 2.0 + alpha);
        let trimmed =
            GeomRectangularTrimmedSurface::new(&cylinder, u, rails.path_range()?, (true, true))?;
        self.my_surface = Handle::new(trimmed).cast();
        self.my_status = GeomFillPipeError::PipeOk;
        Ok(true)
    }

    /// Three coaxial circles of the same range, the rails in a plane of the axis
    /// at the path start: the torus of `myRadius` around the path.
    fn torus_between(&mut self, rails: &Rails<'_>) -> Result<bool, Failure> {
        let Some(f) = torus_frame(rails)? else {
            return Ok(false);
        };
        let mut t = Torus3::new(Axis3::from_axis2(f.a0), f.radius, self.my_radius)?;
        let x_ref = Direction3::from_vector(Vector3::between(f.a0.location(), f.p0))?;
        let mut vv1 = f.v1.angle_with_ref(x_ref, f.y_ref);
        let mut vv2 = f.v2.angle_with_ref(x_ref, f.y_ref);
        let delta_v = f.v2.angle_with_ref(f.v1, f.y_ref);
        if delta_v < 0.0 {
            t.v_reverse();
            vv1 = -vv1;
            vv2 = 2.0 * PI + vv1 - delta_v;
        }
        let torus: Handle<dyn GeomSurface> = Handle::new(GeomToroidalSurface::from_torus(t)).cast();
        let trimmed = GeomRectangularTrimmedSurface::new(
            &torus,
            rails.path_range()?,
            (vv1, vv2),
            (true, true),
        )?;
        self.my_surface = Handle::new(trimmed).cast();
        self.my_exch_uv = true;
        self.my_status = GeomFillPipeError::PipeOk;
        Ok(true)
    }
}

/// The frame of a cylindrical blend: the path axis, its start point and the unit
/// vectors to the rail starts.
struct CylinderFrame {
    axis: Axis1,
    p0: Point3,
    v1: Direction3,
    v2: Direction3,
}

/// The frame of three parallel lines of the same length whose starts lie in a
/// plane normal to them, or `None`.
fn cylinder_frame(rails: &Rails<'_>) -> Result<Option<CylinderFrame>, Failure> {
    let a0 = rails.path.line()?.position();
    let d0 = a0.direction();
    if !same_directions([
        d0,
        rails.first.line()?.direction(),
        rails.last.line()?.direction(),
    ]) {
        return Ok(None);
    }
    let mut lengths = [0.0; 3];
    for (l, c) in lengths.iter_mut().zip(rails.all()) {
        *l = c.last_parameter()? - c.first_parameter()?;
    }
    let conf = precision::confusion();
    if (lengths[1] - lengths[0]).abs() > conf || (lengths[2] - lengths[0]).abs() > conf {
        return Ok(None);
    }
    let p = rails.starts()?;
    let (v1, v2) = start_directions(p)?;
    if v1.dot(d0).abs() > conf || v2.dot(d0).abs() > conf {
        return Ok(None);
    }
    Ok(Some(CylinderFrame {
        axis: a0,
        p0: p[0],
        v1,
        v2,
    }))
}

/// The frame of a toroidal blend: the path circle frame and radius, its start
/// point, the unit vectors to the rail starts and the path tangent at the start.
struct TorusFrame {
    a0: Axis2,
    radius: f64,
    p0: Point3,
    v1: Direction3,
    v2: Direction3,
    y_ref: Direction3,
}

/// The frame of three coaxial circles of the same range whose starts lie in a
/// plane of the axis, or `None`.
fn torus_frame(rails: &Rails<'_>) -> Result<Option<TorusFrame>, Failure> {
    if !same_ranges(rails)? {
        return Ok(None);
    }
    let a = [
        rails.path.circle()?.position(),
        rails.first.circle()?.position(),
        rails.last.circle()?.position(),
    ];
    let p = rails.starts()?;
    if !same_directions([a[0].direction(), a[1].direction(), a[2].direction()])
        || !coaxial_centres(a)
    {
        return Ok(None);
    }
    let (v1, v2) = start_directions(p)?;
    let radius = rails.path.circle()?.radius();
    let y_ref = start_tangent(rails.path, a[0], radius)?;
    let conf = precision::confusion();
    if v1.dot(y_ref).abs() > conf || v2.dot(y_ref).abs() > conf {
        return Ok(None);
    }
    Ok(Some(TorusFrame {
        a0: a[0],
        radius,
        p0: p[0],
        v1,
        v2,
        y_ref,
    }))
}

/// The unit tangent of the path circle at its start.
fn start_tangent(path: &dyn Adaptor3dCurve, a0: Axis2, radius: f64) -> Result<Direction3, Failure> {
    Direction3::from_vector(elclib::circle_dn(path.first_parameter()?, a0, radius, 1))
}

/// Whether the three circles span the same angle (within Precision::Angular()).
fn same_ranges(rails: &Rails<'_>) -> Result<bool, Failure> {
    let mut alp = [0.0; 3];
    for (a, c) in alp.iter_mut().zip(rails.all()) {
        *a = c.first_parameter()? - c.last_parameter()?;
    }
    let ang = precision::angular();
    let differs = (alp[0] - alp[1]).abs() > ang || (alp[0] - alp[2]).abs() > ang;
    Ok(!differs)
}

/// Whether the rail centres lie on the path axis.
fn coaxial_centres(a: [Axis2; 3]) -> bool {
    let l = Line3::from_axis(a[0].axis());
    let conf = precision::confusion();
    l.contains(a[1].location(), conf) && l.contains(a[2].location(), conf)
}
