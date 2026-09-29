//! The GeomBndLib and BndLib probe; the counterpart of
//! tools/oracle/tkgeombase_probe_bndlib.cxx. `gp` holds the bnd_gp and bnd_helpers cases.
use super::common::{self as p, Reader, attempt, end};
use super::lprop::build;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{
    GeomBezierSurface, GeomCurve, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::{
    GeomAdaptorCurve, GeomAdaptorSurface, GeomAdaptorTransformedSurface,
};
use rustycad_tkgeombase::bnd_lib::{Add2dCurve, Add3dCurve, AddSurface};
use rustycad_tkgeombase::geom_bnd_lib::{Curve, Curve2d, Surface};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::Transform3;

pub mod gp;

/// gb_bnd_box.
pub fn bnd_box(b: &BoundingBox3) {
    p::flag(b.is_void());
    p::flag(b.is_whole());
    p::flag(b.is_open_xmin());
    p::flag(b.is_open_xmax());
    p::flag(b.is_open_ymin());
    p::flag(b.is_open_ymax());
    p::flag(b.is_open_zmin());
    p::flag(b.is_open_zmax());
    p::real(b.gap());
    let l = b.limits();
    [l.x_min, l.x_max, l.y_min, l.y_max, l.z_min, l.z_max]
        .into_iter()
        .for_each(p::real);
}

/// gb_bnd_box2d.
pub fn bnd_box2d(b: &BoundingBox2) {
    p::flag(b.is_void());
    p::flag(b.is_whole());
    p::flag(b.is_open_xmin());
    p::flag(b.is_open_xmax());
    p::flag(b.is_open_ymin());
    p::flag(b.is_open_ymax());
    p::real(b.gap());
    let l = b.limits();
    [l.x_min, l.x_max, l.y_min, l.y_max]
        .into_iter()
        .for_each(p::real);
}

/// gb_bnd_surface: 0 an LProp surface, 1 a rational Bezier surface, 2 a rectangular
/// trimmed surface.
pub fn surface(r: &mut Reader<'_>) -> Result<Handle<dyn GeomSurface>, Failure> {
    match r.integer() {
        0 => build::surface(r),
        1 => {
            let (nu, nv) = (r.integer(), r.integer());
            let mut poles = Array2::with_bounds(1, nu, 1, nv).unwrap();
            let mut weights = Array2::with_bounds(1, nu, 1, nv).unwrap();
            for i in 1..=nu {
                for j in 1..=nv {
                    poles[(i, j)] = r.pnt();
                    weights[(i, j)] = r.real();
                }
            }
            Ok(Handle::new(GeomBezierSurface::with_weights(&poles, &weights)?).cast())
        }
        _ => {
            let (u1, u2, v1, v2) = (r.real(), r.real(), r.real(), r.real());
            let basis = surface(r)?;
            let trimmed =
                GeomRectangularTrimmedSurface::new(&basis, (u1, u2), (v1, v2), (true, true))?;
            Ok(Handle::new(trimmed).cast())
        }
    }
}

// A built geometry, or the raised exception type ending the request.
fn built<T>(value: Result<T, Failure>) -> Option<T> {
    match value {
        Ok(value) => Some(value),
        Err(error) => {
            println!("{}", error.exception_type());
            None
        }
    }
}

// The four box members of a curve dispatcher (gb_bnd_curve_op).
fn curve_op(e: &Curve<'_>, op: i32, (u1, u2, tol): (f64, f64, f64)) -> Result<(), Failure> {
    p::int(e.get_type() as i32);
    let b = match op {
        0 => e.bounding_box(tol)?,
        1 => e.bounding_box_range(u1, u2, tol)?,
        2 => e.bounding_box_optimal(tol)?,
        _ => e.bounding_box_optimal_range(u1, u2, tol)?,
    };
    bnd_box(&b);
    Ok(())
}

fn curve2d_op(e: &Curve2d<'_>, op: i32, (u1, u2, tol): (f64, f64, f64)) -> Result<(), Failure> {
    p::int(e.get_type() as i32);
    let b = match op {
        0 => e.bounding_box(tol)?,
        1 => e.bounding_box_range(u1, u2, tol)?,
        2 => e.bounding_box_optimal(tol)?,
        _ => e.bounding_box_optimal_range(u1, u2, tol)?,
    };
    bnd_box2d(&b);
    Ok(())
}

fn unit_box() -> BoundingBox3 {
    let mut b = BoundingBox3::default();
    b.update_bounds([-1.0; 3], [1.0; 3]);
    b
}

fn unit_box2d() -> BoundingBox2 {
    let mut b = BoundingBox2::default();
    b.update_bounds([-1.0; 2], [1.0; 2]);
    b
}

// BndLib_Add3dCurve on a GeomAdaptor_Curve and the box [-1, 1]^3.
fn add3d(
    c: &Handle<dyn GeomCurve>,
    op: i32,
    (u1, u2, tol): (f64, f64, f64),
) -> Result<(), Failure> {
    let a = GeomAdaptorCurve::with_curve(c)?;
    let mut b = unit_box();
    match op {
        0 => Add3dCurve::add(&a, tol, &mut b)?,
        1 => Add3dCurve::add_range(&a, u1, u2, tol, &mut b)?,
        2 => Add3dCurve::add_optimal(&a, tol, &mut b)?,
        _ => Add3dCurve::add_optimal_range(&a, u1, u2, tol, &mut b)?,
    }
    bnd_box(&b);
    Ok(())
}

pub fn curve3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, u1, u2, tol) = (r.integer(), r.real(), r.real(), r.real());
    let Some(c) = built(build::curve3d(&mut r)) else {
        return;
    };
    let (op, src, args) = (mode % 4, mode / 4, (u1, u2, tol));
    attempt(|| match src {
        0 => curve_op(&Curve::from_curve(&c)?, op, args),
        1 => {
            let a = GeomAdaptorCurve::with_curve(&c)?;
            curve_op(&Curve::from_adaptor(&a)?, op, args)
        }
        2 => {
            let a = GeomAdaptorCurve::with_range(&c, u1, u2)?;
            curve_op(&Curve::from_adaptor(&a)?, op, args)
        }
        _ => add3d(&c, op, args),
    });
    end();
}

// BndLib_Add2dCurve on the handle (source 3), an Adaptor2d_OffsetCurve (source 4) or a
// Geom2dAdaptor_Curve (source 5); every box starts as [-1, 1]^2.
fn add2d(
    c: &Handle<dyn Geom2dCurve>,
    src: i32,
    op: i32,
    (u1, u2, tol): (f64, f64, f64),
) -> Result<(), Failure> {
    let mut b = unit_box2d();
    match (src, op) {
        (3, 0) => Add2dCurve::add_curve(c, tol, &mut b)?,
        (3, 1) => Add2dCurve::add_curve_range(c, u1, u2, tol, &mut b)?,
        (3, _) => Add2dCurve::add_optimal(c, u1, u2, tol, &mut b)?,
        (4, _) => return offset_adaptor(c, op, (u1, u2, tol), &mut b),
        (_, 0) => Add2dCurve::add_adaptor(&Geom2dAdaptorCurve::with_curve(c)?, tol, &mut b)?,
        _ => {
            Add2dCurve::add_adaptor_range(&Geom2dAdaptorCurve::with_curve(c)?, u1, u2, tol, &mut b)?
        }
    }
    bnd_box2d(&b);
    Ok(())
}

// The Adaptor2d_OffsetCurve (offset 0.5) source of bnd_curve2d.
fn offset_adaptor(
    c: &Handle<dyn Geom2dCurve>,
    op: i32,
    (u1, u2, tol): (f64, f64, f64),
    b: &mut BoundingBox2,
) -> Result<(), Failure> {
    let g: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_curve(c)?).cast();
    let a = Adaptor2dOffsetCurve::with_offset(&g, 0.5)?;
    match op {
        0 => Add2dCurve::add_adaptor(&a, tol, b)?,
        1 => Add2dCurve::add_adaptor_range(&a, u1, u2, tol, b)?,
        _ => {
            let e = Curve2d::from_adaptor(&a)?;
            p::int(e.get_type() as i32);
            let result = if op == 2 {
                e.bounding_box(tol)?
            } else {
                e.bounding_box_optimal_range(u1, u2, tol)?
            };
            bnd_box2d(&result);
            return Ok(());
        }
    }
    bnd_box2d(b);
    Ok(())
}

pub fn curve2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mode, u1, u2, tol) = (r.integer(), r.real(), r.real(), r.real());
    let Some(c) = built(build::curve2d(&mut r)) else {
        return;
    };
    let (op, src, args) = (mode % 4, mode / 4, (u1, u2, tol));
    attempt(|| match src {
        0 => curve2d_op(&Curve2d::from_curve(&c)?, op, args),
        1 => {
            let a = Geom2dAdaptorCurve::with_curve(&c)?;
            curve2d_op(&Curve2d::from_adaptor(&a)?, op, args)
        }
        2 => {
            let a = Geom2dAdaptorCurve::with_range(&c, u1, u2)?;
            curve2d_op(&Curve2d::from_adaptor(&a)?, op, args)
        }
        _ => add2d(&c, src, op, args),
    });
    end();
}

// The four box members of a surface dispatcher (the `run` lambda).
fn surface_op(e: &Surface<'_>, op: i32, patch: [f64; 4], tol: f64) -> Result<(), Failure> {
    p::int(e.get_type() as i32);
    let [u1, u2, v1, v2] = patch;
    let b = match op {
        0 => e.bounding_box(tol)?,
        1 => e.bounding_box_range(u1, u2, v1, v2, tol)?,
        2 => e.bounding_box_optimal(tol)?,
        _ => e.bounding_box_optimal_range(u1, u2, v1, v2, tol)?,
    };
    bnd_box(&b);
    Ok(())
}

// BndLib_AddSurface on a GeomAdaptor_Surface and the box [-1, 1]^3.
fn add_surface(
    s: &Handle<dyn GeomSurface>,
    op: i32,
    patch: [f64; 4],
    tol: f64,
) -> Result<(), Failure> {
    let a = GeomAdaptorSurface::with_surface(s)?;
    let mut b = unit_box();
    match op {
        0 => AddSurface::add(&a, tol, &mut b)?,
        1 => AddSurface::add_range(&a, patch, tol, &mut b)?,
        2 => AddSurface::add_optimal(&a, tol, &mut b)?,
        _ => AddSurface::add_optimal_range(&a, patch, tol, &mut b)?,
    }
    bnd_box(&b);
    Ok(())
}

pub fn surface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let patch = [r.real(), r.real(), r.real(), r.real()];
    let tol = r.real();
    let (axis, angle, translation) = (r.ax1(), r.real(), r.vec());
    let Some(s) = built(surface(&mut r)) else {
        return;
    };
    let (op, src) = (mode % 4, mode / 4);
    attempt(|| match src {
        0 => surface_op(&Surface::from_surface(&s)?, op, patch, tol),
        1 => {
            let a = GeomAdaptorSurface::with_surface(&s)?;
            surface_op(&Surface::from_adaptor(&a)?, op, patch, tol)
        }
        2 => {
            let a = GeomAdaptorSurface::with_bounds(&s, (patch[0], patch[1], patch[2], patch[3]))?;
            surface_op(&Surface::from_adaptor(&a)?, op, patch, tol)
        }
        3 => {
            let mut t = Transform3::default();
            t.set_rotation(axis, angle)?;
            t.set_translation_part(translation);
            let a = GeomAdaptorTransformedSurface::with_surface(&s, t)?;
            surface_op(&Surface::from_adaptor(&a)?, op, patch, tol)
        }
        _ => add_surface(&s, op, patch, tol),
    });
    end();
}
