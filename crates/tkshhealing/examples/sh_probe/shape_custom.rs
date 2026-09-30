//! ShapeCustom_Curve2d and ShapeCustom_RestrictionParameters probe operations, the Rust
//! twin of tools/oracle/tkshhealing_probe_shapecustom.cxx (which documents the requests).
use super::brep_lib_common::{geom2d_curve, type_name};
use super::common::{Reader, attempt, end, flag, int, pnt2d, real, show, text};
use super::sh_common::curve2d;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::Point2;
use rustycad_tkshhealing::shape_custom::{ShapeCustomCurve2d, ShapeCustomRestrictionParameters};

fn is_linear(r: &mut Reader<'_>) {
    let n = r.integer();
    let points: Vec<Point2> = (0..n).map(|_| r.pnt2d()).collect();
    let poles = Array1::from_shared(&points, 1);
    let tol = r.real();
    attempt(|| {
        let deviation = ShapeCustomCurve2d::is_linear(&poles, tol)?;
        flag(deviation.is_some());
        real(deviation.unwrap_or(-1.0));
        Ok(())
    });
}

fn convert_line(r: &mut Reader<'_>) -> Result<(), Failure> {
    let c = geom2d_curve(r)?;
    let (c1, c2, tol) = (r.real(), r.real(), r.real());
    attempt(|| {
        let conversion = ShapeCustomCurve2d::convert_to_line2d(&c, c1, c2, tol)?;
        let line = conversion
            .as_ref()
            .map_or_else(Handle::<Geom2dLine>::default, |found| found.line.clone());
        type_name(&line);
        let (cf, cl, dev) = conversion.as_ref().map_or((-1.0, -1.0, -1.0), |found| {
            (found.first, found.last, found.deviation)
        });
        [cf, cl, dev].into_iter().for_each(real);
        if !line.is_null() {
            curve2d(&line.cast::<dyn Geom2dCurve>());
        }
        Ok(())
    });
    Ok(())
}

fn simplify(r: &mut Reader<'_>) -> Result<(), Failure> {
    let curve = geom2d_curve(r)?;
    let tol = r.real();
    let handle = curve.cast::<Geom2dBSplineCurve>();
    let bs = handle.get().expect("probe B-spline record");
    show(ShapeCustomCurve2d::simplify_bspline2d(bs, tol), flag);
    int(bs.degree());
    int(bs.nb_knots());
    for i in 1..=bs.nb_knots() {
        real(bs.knot(i)?);
        int(bs.multiplicity(i)?);
    }
    int(bs.nb_poles());
    for i in 1..=bs.nb_poles() {
        pnt2d(bs.pole(i)?);
    }
    curve2d(&curve);
    Ok(())
}

/// `sc_curve2d op ...`.
pub fn curve2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    match r.integer() {
        0 => is_linear(&mut r),
        1 => convert_line(&mut r).expect("probe curve record"),
        _ => simplify(&mut r).expect("probe curve record"),
    }
    end();
}

fn restriction_set(p: &ShapeCustomRestrictionParameters, field: i32, value: i32) {
    type Setter = fn(&ShapeCustomRestrictionParameters, bool);
    const FLAGS: [Setter; 14] = [
        ShapeCustomRestrictionParameters::set_convert_plane,
        ShapeCustomRestrictionParameters::set_convert_bezier_surf,
        ShapeCustomRestrictionParameters::set_convert_revolution_surf,
        ShapeCustomRestrictionParameters::set_convert_extrusion_surf,
        ShapeCustomRestrictionParameters::set_convert_offset_surf,
        ShapeCustomRestrictionParameters::set_convert_cylindrical_surf,
        ShapeCustomRestrictionParameters::set_convert_conical_surf,
        ShapeCustomRestrictionParameters::set_convert_toroidal_surf,
        ShapeCustomRestrictionParameters::set_convert_spherical_surf,
        ShapeCustomRestrictionParameters::set_segment_surface_mode,
        ShapeCustomRestrictionParameters::set_convert_curve3d,
        ShapeCustomRestrictionParameters::set_convert_offset_curv3d,
        ShapeCustomRestrictionParameters::set_convert_curve2d,
        ShapeCustomRestrictionParameters::set_convert_offset_curv2d,
    ];
    match field {
        0 => p.set_g_max_degree(value),
        1 => p.set_g_max_seg(value),
        _ => FLAGS[field as usize - 2](p, value != 0),
    }
}

/// `sc_restriction nops (field value)...`.
pub fn restriction_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = Handle::new(ShapeCustomRestrictionParameters::new());
    for _ in 0..r.integer() {
        let field = r.integer();
        restriction_set(&p, field, r.integer());
    }
    int(p.g_max_degree());
    int(p.g_max_seg());
    [
        p.convert_plane(),
        p.convert_bezier_surf(),
        p.convert_revolution_surf(),
        p.convert_extrusion_surf(),
        p.convert_offset_surf(),
        p.convert_cylindrical_surf(),
        p.convert_conical_surf(),
        p.convert_toroidal_surf(),
        p.convert_spherical_surf(),
        p.segment_surface_mode(),
        p.convert_curve3d(),
        p.convert_offset_curv3d(),
        p.convert_curve2d(),
        p.convert_offset_curv2d(),
    ]
    .into_iter()
    .for_each(flag);
    text(p.dynamic_type().map_or("null", |t| t.name()).as_bytes());
    end();
}
