//! Readers and printers shared by the ShapeCustom, ShapeUpgrade and ShapeProcess probe
//! operations, the Rust twin of tools/oracle/tkshhealing_probe_shcommon.cxx (which
//! documents the records).
use super::brep_lib_common::type_name;
use super::common::{Reader, attempt, flag, pnt, pnt2d, real, show, xy, xyz};
use super::shapes::{TpShapes, tp_shapes_into};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomOffsetSurface, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;

/// `shc_trsf`: a transformation record.
pub fn trsf(r: &mut Reader<'_>) -> Result<Transform3, Failure> {
    let kind = r.integer();
    let mut t = Transform3::default();
    match kind {
        0 => t.set_translation(Vector3::from(r.xyz())),
        1 => {
            let (p, d) = (r.pnt(), r.dir());
            t.set_rotation(Axis1::new(p, d), r.real())?;
        }
        2 => {
            let p = r.pnt();
            t.set_scale(p, r.real())?;
        }
        3 => t.set_mirror_point(r.pnt()),
        4 => {
            let p = r.pnt();
            t.set_mirror_axis(Axis1::new(p, r.dir()));
        }
        5 => {
            let p = r.pnt();
            t.set_mirror_plane(Axis2::new(p, r.dir())?);
        }
        _ => {
            let a = trsf(r)?;
            t = a.multiplied(trsf(r)?);
        }
    }
    Ok(t)
}

/// `shc_elementary`: an elementary surface on a possibly indirect frame.
fn elementary(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    let kind = r.integer();
    let (p, n, x) = (r.pnt(), r.dir(), r.dir());
    let mut frame = Axis3::new(p, n, x)?;
    if r.flag() {
        frame.y_reverse();
    }
    Ok(match kind {
        0 => Handle::new(GeomPlane::new(frame)).cast(),
        1 => Handle::new(GeomCylindricalSurface::new(frame, r.real())?).cast(),
        2 => Handle::new(GeomSphericalSurface::new(frame, r.real())?).cast(),
        3 => {
            let angle = r.real();
            Handle::new(GeomConicalSurface::new(frame, angle, r.real())?).cast()
        }
        _ => {
            let major = r.real();
            Handle::new(GeomToroidalSurface::new(frame, major, r.real())?).cast()
        }
    })
}

/// `shc_prelude`: the extra surfaces and locations.
fn prelude(r: &mut Reader<'_>, t: &mut TpShapes) -> Result<(), Failure> {
    for _ in 0..r.integer() {
        match r.integer() {
            0 => t.surfaces.push(elementary(r)?),
            1 => {
                let s = t.surfaces[r.integer() as usize].clone();
                let (u1, u2, v1) = (r.real(), r.real(), r.real());
                let trimmed =
                    GeomRectangularTrimmedSurface::new(&s, (u1, u2), (v1, r.real()), (true, true))?;
                t.surfaces.push(Handle::new(trimmed).cast());
            }
            2 => {
                let s = t.surfaces[r.integer() as usize].clone();
                t.surfaces
                    .push(Handle::new(GeomOffsetSurface::new(&s, r.real(), false)?).cast());
            }
            _ => t.locations.push(Location::new(trsf(r)?)),
        }
    }
    Ok(())
}

/// `shc_shapes`: the prelude, then the shape program on the same tables.
pub fn shapes(r: &mut Reader<'_>) -> Result<TpShapes, Failure> {
    let mut t = TpShapes {
        surfaces: Vec::new(),
        curves: Vec::new(),
        pcurves: Vec::new(),
        locations: vec![Location::default()],
        triangulations: Vec::new(),
        shapes: Vec::new(),
    };
    prelude(r, &mut t)?;
    tp_shapes_into(r, &mut t)?;
    Ok(t)
}

/// `shc_surface`: the digest of a surface.
pub fn surface(s: &Handle<dyn GeomSurface>) {
    type_name(s);
    let Some(s) = s.get() else { return };
    let (u1, u2, v1, v2) = s.bounds();
    [u1, u2, v1, v2].into_iter().for_each(real);
    flag(s.is_u_periodic());
    flag(s.is_v_periodic());
    for (u, v) in [(0.25, 0.5), (1.0, -0.5), (-0.7, 0.3)] {
        show(s.value(u, v), pnt);
    }
    attempt(|| {
        let d1 = s.d1(0.3, 0.2)?;
        pnt(d1.point);
        xyz(*d1.d1u.xyz());
        xyz(*d1.d1v.xyz());
        Ok(())
    });
}

/// `shc_curve_points`: the range, the points at the finite ends, the middle and 0.3, and
/// the first derivative at 0.3.
fn curve_points((f, l): (f64, f64), point: impl Fn(f64), derivative: impl Fn(f64)) {
    real(f);
    real(l);
    let (f_inf, l_inf) = (precision::is_infinite(f), precision::is_infinite(l));
    if !f_inf {
        point(f);
    }
    if !l_inf {
        point(l);
    }
    if !f_inf && !l_inf {
        point(0.5 * (f + l));
    }
    point(0.3);
    derivative(0.3);
}

/// `shc_curve`: the digest of a 3d curve.
pub fn curve(c: &Handle<dyn GeomCurve>) {
    type_name(c);
    let Some(c) = c.get() else { return };
    curve_points(
        (c.first_parameter(), c.last_parameter()),
        |u| show(c.value(u), pnt),
        |u| show(c.dn(u, 1), |d| xyz(*d.xyz())),
    );
}

/// `shc_curve2d`: the digest of a 2d curve.
pub fn curve2d(c: &Handle<dyn Geom2dCurve>) {
    type_name(c);
    let Some(c) = c.get() else { return };
    curve_points(
        (c.first_parameter(), c.last_parameter()),
        |u| show(c.value(u), pnt2d),
        |u| show(c.dn(u, 1), |d| xy(d.xy())),
    );
}

/// `shc_location`: the identity flag and the 12 values of the transformation.
pub fn location(l: &Location) {
    flag(l.is_identity());
    let t = l.transformation();
    for i in 1..=3 {
        for j in 1..=4 {
            real(t.value(i, j).unwrap_or(f64::NAN));
        }
    }
}
