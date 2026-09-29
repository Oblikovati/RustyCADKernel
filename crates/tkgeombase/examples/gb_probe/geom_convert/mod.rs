//! The GeomConvert probe; the counterpart of tools/oracle/tkgeombase_probe_geomconvert.cxx
//! (conversions, printers) and tkgeombase_probe_geomconvert_ana.cxx (recognizers). The
//! printers are shared with the GeomLib probe.
mod ana;
mod convert;
pub use ana::*;
pub use convert::*;

use super::common::{Reader, ax1, ax3, circ, circ2d, cone, cylinder, dir, elips, elips2d, flag};
use super::common::{hypr, hypr2d, int, lin, lin2d, parab, parab2d, pln, pnt, real};
use super::gc::name;
use super::geom_tools::build;
use super::geom2d_convert::{bezier as bezier2d, bspline as bspline2d};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dEllipse,
    Geom2dHyperbola, Geom2dLine, Geom2dOffsetCurve, Geom2dParabola, Geom2dTrimmedCurve,
};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierCurve, GeomBezierSurface, GeomCircle,
    GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomHyperbola, GeomLine,
    GeomOffsetCurve, GeomOffsetSurface, GeomParabola, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution,
    GeomSweptSurface, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkmath::gp::{Sphere3, Torus3};

/// gb_gcv_bspline.
pub fn bspline(c: &Handle<GeomBSplineCurve>) {
    let Some(c) = c.get() else {
        return int(-1);
    };
    int(c.degree());
    flag(c.is_periodic());
    flag(c.is_rational());
    int(c.nb_poles());
    for i in 1..=c.nb_poles() {
        pnt(c.pole(i).unwrap());
        real(c.weight(i).unwrap());
    }
    int(c.nb_knots());
    for i in 1..=c.nb_knots() {
        real(c.knot(i).unwrap());
        int(c.multiplicity(i).unwrap());
    }
}

/// gb_gcv_bezier.
pub fn bezier(c: &Handle<GeomBezierCurve>) {
    let Some(c) = c.get() else {
        return int(-1);
    };
    int(c.degree());
    flag(c.is_rational());
    int(c.nb_poles());
    for i in 1..=c.nb_poles() {
        pnt(c.pole(i).unwrap());
        real(c.weight(i).unwrap());
    }
}

/// gb_gcv_bspline_surface.
pub fn bspline_surface(s: &Handle<GeomBSplineSurface>) {
    let Some(s) = s.get() else {
        return int(-1);
    };
    int(s.u_degree());
    int(s.v_degree());
    flag(s.is_u_periodic());
    flag(s.is_v_periodic());
    flag(s.is_u_rational());
    flag(s.is_v_rational());
    int(s.nb_u_poles());
    int(s.nb_v_poles());
    for i in 1..=s.nb_u_poles() {
        for j in 1..=s.nb_v_poles() {
            pnt(s.pole(i, j).unwrap());
            real(s.weight(i, j).unwrap());
        }
    }
    int(s.nb_u_knots());
    for i in 1..=s.nb_u_knots() {
        real(s.u_knot(i).unwrap());
        int(s.u_multiplicity(i).unwrap());
    }
    int(s.nb_v_knots());
    for i in 1..=s.nb_v_knots() {
        real(s.v_knot(i).unwrap());
        int(s.v_multiplicity(i).unwrap());
    }
}

/// gb_gcv_bezier_surface.
pub fn bezier_surface(s: &Handle<GeomBezierSurface>) {
    let Some(s) = s.get() else {
        return int(-1);
    };
    int(s.u_degree());
    int(s.v_degree());
    flag(s.is_u_rational());
    flag(s.is_v_rational());
    for i in 1..=s.nb_u_poles() {
        for j in 1..=s.nb_v_poles() {
            pnt(s.pole(i, j).unwrap());
            real(s.weight(i, j).unwrap());
        }
    }
}

fn sphere(s: Sphere3) {
    ax3(s.position());
    real(s.radius());
}

fn torus(t: Torus3) {
    ax3(t.position());
    real(t.major_radius());
    real(t.minor_radius());
}

/// gb_gcv_curve: the class name, then the data of its kind.
pub fn curve(c: &Handle<dyn GeomCurve>) {
    name(c);
    if let Some(t) = c.cast::<GeomTrimmedCurve>().get() {
        real(t.first_parameter());
        real(t.last_parameter());
        return curve(&t.basis_curve());
    }
    if let Some(x) = c.cast::<GeomLine>().get() {
        return lin(x.lin());
    }
    if let Some(x) = c.cast::<GeomCircle>().get() {
        return circ(x.circ().unwrap());
    }
    if let Some(x) = c.cast::<GeomEllipse>().get() {
        return elips(x.elips().unwrap());
    }
    if let Some(x) = c.cast::<GeomHyperbola>().get() {
        return hypr(x.hypr().unwrap());
    }
    if let Some(x) = c.cast::<GeomParabola>().get() {
        return parab(x.parab().unwrap());
    }
    curve_poles(c);
}

fn curve_poles(c: &Handle<dyn GeomCurve>) {
    let b: Handle<GeomBSplineCurve> = c.cast();
    if b.is_some() {
        return bspline(&b);
    }
    let z: Handle<GeomBezierCurve> = c.cast();
    if z.is_some() {
        return bezier(&z);
    }
    if let Some(x) = c.cast::<GeomOffsetCurve>().get() {
        real(x.offset());
        dir(x.direction());
        curve(&x.basis_curve());
    }
}

/// gb_gcv_curve2d.
pub fn curve2d(c: &Handle<dyn Geom2dCurve>) {
    name(c);
    if let Some(t) = c.cast::<Geom2dTrimmedCurve>().get() {
        real(t.first_parameter());
        real(t.last_parameter());
        return curve2d(&t.basis_curve());
    }
    if let Some(x) = c.cast::<Geom2dLine>().get() {
        return lin2d(x.lin2d());
    }
    if let Some(x) = c.cast::<Geom2dCircle>().get() {
        return circ2d(x.circ2d().unwrap());
    }
    if let Some(x) = c.cast::<Geom2dEllipse>().get() {
        return elips2d(x.elips2d().unwrap());
    }
    if let Some(x) = c.cast::<Geom2dHyperbola>().get() {
        return hypr2d(x.hypr2d().unwrap());
    }
    if let Some(x) = c.cast::<Geom2dParabola>().get() {
        return parab2d(x.parab2d().unwrap());
    }
    curve2d_poles(c);
}

fn curve2d_poles(c: &Handle<dyn Geom2dCurve>) {
    let b: Handle<Geom2dBSplineCurve> = c.cast();
    if b.is_some() {
        return bspline2d(&b);
    }
    let z: Handle<Geom2dBezierCurve> = c.cast();
    if z.is_some() {
        return bezier2d(&z);
    }
    if let Some(x) = c.cast::<Geom2dOffsetCurve>().get() {
        real(x.offset());
        curve2d(&x.basis_curve());
    }
}

/// gb_gcv_surface: the class name, then the data of its kind.
pub fn surface(s: &Handle<dyn GeomSurface>) {
    name(s);
    if let Some(t) = s.cast::<GeomRectangularTrimmedSurface>().get() {
        let (u1, u2, v1, v2) = t.bounds();
        [u1, u2, v1, v2].into_iter().for_each(real);
        return surface(&t.basis_surface());
    }
    if let Some(x) = s.cast::<GeomPlane>().get() {
        return pln(x.pln());
    }
    if let Some(x) = s.cast::<GeomCylindricalSurface>().get() {
        return cylinder(x.cylinder().unwrap());
    }
    if let Some(x) = s.cast::<GeomConicalSurface>().get() {
        return cone(x.cone().unwrap());
    }
    if let Some(x) = s.cast::<GeomSphericalSurface>().get() {
        return sphere(x.sphere().unwrap());
    }
    if let Some(x) = s.cast::<GeomToroidalSurface>().get() {
        return torus(x.torus().unwrap());
    }
    surface_poles(s);
}

fn surface_poles(s: &Handle<dyn GeomSurface>) {
    let b: Handle<GeomBSplineSurface> = s.cast();
    if b.is_some() {
        return bspline_surface(&b);
    }
    let z: Handle<GeomBezierSurface> = s.cast();
    if z.is_some() {
        return bezier_surface(&z);
    }
    if let Some(x) = s.cast::<GeomSurfaceOfRevolution>().get() {
        ax1(x.axis());
        return curve(&x.basis_curve());
    }
    if let Some(x) = s.cast::<GeomSurfaceOfLinearExtrusion>().get() {
        dir(x.direction());
        return curve(&x.basis_curve());
    }
    if let Some(x) = s.cast::<GeomOffsetSurface>().get() {
        real(x.offset());
        surface(&x.basis_surface());
    }
}

/// gb_gcv_read_bspline.
pub fn read_bspline(r: &mut Reader<'_>) -> Result<Handle<GeomBSplineCurve>, Failure> {
    Ok(build::curve(r)?.cast())
}

/// gb_gcv_read_bspline_surface.
pub fn read_bspline_surface(r: &mut Reader<'_>) -> Result<Handle<GeomBSplineSurface>, Failure> {
    Ok(build::surface(r)?.cast())
}

/// The unset value the in-out outputs start from (GB_GCV_UNSET).
pub const UNSET: f64 = -7.25;
