//! IntPatch_WLineTool::ExtendTwoWLines probe operation, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_extend.cxx: walking lines on two quadrics,
//! extended to their common points.
use super::common::{Reader, attempt, end, int};
use super::ics_geometry::surface;
use super::int_patch::Fixture;
use super::int_patch_wltool::report;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_patch::{
    IntPatchLine, IntPatchPoint, IntPatchPointLine, IntPatchWLType, IntPatchWLine,
    IntPatchWLineTool,
};
use rustycad_tkgeomalgo::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTypeTrans};
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Point2, Point3};
use std::f64::consts::FRAC_PI_2;

/// `ga_ex_params`.
fn params(s: &Adaptor3dSurfaceHandle, p: Point3) -> Result<(f64, f64), Failure> {
    Ok(match s.get_type()? {
        SurfaceType::Plane => elslib::parameters(&s.plane()?, p),
        SurfaceType::Cylinder => elslib::parameters(&s.cylinder()?, p),
        SurfaceType::Sphere => elslib::parameters(&s.sphere()?, p),
        SurfaceType::Cone => elslib::parameters(&s.cone()?, p),
        SurfaceType::Torus => elslib::parameters(&s.torus()?, p),
        _ => (0.0, 0.0),
    })
}

/// The vertex at the point `i` of the line.
fn end_vertex(c: &IntSurfLineOn2S, i: i32) -> Result<IntPatchPoint, Failure> {
    let q = c.value(i)?;
    let mut v = IntPatchPoint::new();
    v.set_value_tangent(q.value(), 1e-7, false);
    let (a, b, e, d) = q.parameters();
    v.set_parameters(a, b, e, d);
    v.set_parameter(f64::from(i));
    Ok(v)
}

/// `ga_ex_line`.
fn line(
    r: &mut Reader,
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
) -> Result<Handle<dyn IntPatchLine>, Failure> {
    let n = r.integer();
    let pts: Vec<Point3> = (0..n).map(|_| r.pnt()).collect();
    let (sh1, sh2) = (r.real(), r.real());
    let c: Handle<IntSurfLineOn2S> = Handle::new(IntSurfLineOn2S::new(None));
    for p in pts {
        let ((u1, v1), (u2, v2)) = (params(s1, p)?, params(s2, p)?);
        let mut q = IntSurfPntOn2S::new();
        q.set_value_all(p, u1 + sh1, v1, u2 + sh2, v2);
        c.add(&q);
    }
    let l = IntPatchWLine::new(&c, false, IntSurfTypeTrans::In, IntSurfTypeTrans::Out);
    l.set_creating_way_info(IntPatchWLType::ImpImp);
    if n > 0 {
        l.add_vertex(&end_vertex(&c, 1)?, false);
        l.add_vertex(&end_vertex(&c, n)?, false);
    }
    Ok(Handle::new(l).cast())
}

/// The critical points of a quadric: the apex of a cone, the poles of a sphere.
fn critical(s: &Adaptor3dSurfaceHandle, crit: &mut Vec<Point3>) -> Result<(), Failure> {
    match s.get_type()? {
        SurfaceType::Cone => crit.push(s.cone()?.apex()),
        SurfaceType::Sphere => {
            crit.push(s.value(0.0, FRAC_PI_2)?);
            crit.push(s.value(0.0, -FRAC_PI_2)?);
        }
        _ => {}
    }
    Ok(())
}

/// The box of the parameters of the surface bounds, enlarged by PConfusion.
fn bounds_box(s: &Adaptor3dSurfaceHandle) -> Result<BoundingBox2, Failure> {
    let (uf, ul) = (s.first_u_parameter()?, s.last_u_parameter()?);
    let (vf, vl) = (s.first_v_parameter()?, s.last_v_parameter()?);
    let mut b = BoundingBox2::default();
    for (u, v) in [(uf, vf), (ul, vf), (ul, vl), (uf, vl)] {
        b.add_point(Point2::new(u, v));
    }
    b.enlarge(precision::p_confusion());
    Ok(b)
}

fn period(s: &Adaptor3dSurfaceHandle, u: bool) -> Result<f64, Failure> {
    match u {
        true if s.is_u_periodic()? => s.u_period(),
        false if s.is_v_periodic()? => s.v_period(),
        _ => Ok(0.0),
    }
}

/// `ga_ip_extend_case`.
pub fn extend_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let fx = Fixture::new();
    attempt(|| {
        let (s1, s2) = (surface(&mut r), surface(&mut r));
        let tol = r.real();
        let mut crit: Vec<Point3> = Vec::new();
        critical(&s1, &mut crit)?;
        critical(&s2, &mut crit)?;
        for _ in 0..r.integer() {
            crit.push(r.pnt());
        }
        let mut lines: Vec<Handle<dyn IntPatchLine>> = Vec::new();
        for _ in 0..r.integer() {
            lines.push(line(&mut r, &s1, &s2)?);
        }
        let (b1, b2) = (bounds_box(&s1)?, bounds_box(&s2)?);
        let per = [
            period(&s1, true)?,
            period(&s1, false)?,
            period(&s2, true)?,
            period(&s2, false)?,
        ];
        attempt(|| {
            IntPatchWLineTool::extend_two_wlines(&mut lines, &s1, &s2, tol, &per, &b1, &b2, &crit)
        });
        int(lines.len() as i32);
        lines.iter().for_each(|l| report(&fx, l));
        Ok(())
    });
    end();
}
