// Rust translation of BRepFeat::IsInside, FaceUntil and Tool of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The face functions of the BRepFeat package: whether a face lies inside another, a
//! face extended to cover a shape, and the half space a face bounds in a shape.
use crate::errors::null_object;
use crate::loc_ope::BuildShape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape, Solid};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomPlane, GeomRectangularTrimmedSurface,
    GeomSurface,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeombase::gcpnts::QuasiUniformDeflection;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Vector2};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
use rustycad_tktopalgo::brep_lib::BRepLibMakeFace;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;

/// The static `IsIn(FC, AC)`: no point of a quasi-uniform deflection sampling of the
/// curve (100 Confusion) is OUT of the face.
fn is_in(fc: &BRepTopAdaptorFClass2d, ac: &Geom2dAdaptorCurve) -> Result<bool, Failure> {
    let def = 100. * precision::confusion();
    let qu = QuasiUniformDeflection::with_curve(ac as &dyn Adaptor2dCurve2d, def, Continuity::C1)?;
    for i in 1..=qu.nb_points()? {
        let p = ac.value(qu.parameter(i)?)?;
        if fc.perform(p, false)? == State::Out {
            return Ok(false);
        }
    }
    Ok(true)
}

/// `PutInBoundsU` / `PutInBoundsV`: shifts the p-curve by periods along the direction
/// `along_u` so that it starts in `[min, max]`, then by one period back when it then
/// overflows more than the underflow it would make.
fn put_in_bounds(
    bounds: (f64, f64),
    eps: f64,
    period: f64,
    (f, l): (f64, f64),
    c2d: &Handle<dyn Geom2dCurve>,
    along_u: bool,
) -> Result<(), Failure> {
    let curve = c2d
        .get()
        .ok_or_else(|| null_object("BRepFeat::IsInside", "null p-curve"))?;
    let coord = |u: f64| -> Result<f64, Failure> {
        let p = curve.value(u)?;
        Ok(if along_u { p.x() } else { p.y() })
    };
    let shift = |d: f64| {
        if along_u {
            Vector2::new(d, 0.)
        } else {
            Vector2::new(0., d)
        }
    };
    let (cf, cl, cm) = (coord(f)?, coord(l)?, coord(0.34 * f + 0.66 * l)?);
    let (min, max) = bounds;
    let mut min_c = cf.min(cl).min(cm);
    let mut max_c = cf.max(cl).max(cm);
    let mut du = 0.;
    if min_c < min - eps {
        du = f64::from(((min - min_c) / period) as i32 + 1) * period;
    }
    if min_c > max + eps {
        du = -f64::from(((min_c - max) / period) as i32 + 1) * period;
    }
    if du != 0. {
        curve.translate(shift(du))?;
        min_c += du;
        max_c += du;
    }
    // Ajuste au mieux la courbe dans le domaine.
    if max_c > max + 100. * eps {
        let d1 = max_c - max;
        let d2 = min - min_c + period;
        if d2 < d1 {
            du = -period;
        }
        if du != 0. {
            curve.translate(shift(du))?;
        }
    }
    Ok(())
}

/// `BRepFeat::IsInside(F1, F2)`: every edge of `f1`, projected on the surface of `f2`
/// (recalibrated into its periods), stays inside `f2`;
/// `brep_feat::is_inside(&f1, &f2)?`.
pub fn is_inside(f1: &Face, f2: &Face) -> Result<bool, Failure> {
    let s = BRepTool::surface(f2)?;
    let (umin, umax, vmin, vmax) = brep_tools::uv_bounds(f2)?;
    let surface = s
        .get()
        .ok_or_else(|| null_object("BRepFeat::IsInside", "face without surface"))?;
    let uperiod = if surface.is_u_periodic() {
        Some(surface.u_period()?)
    } else {
        None
    };
    let vperiod = if surface.is_v_periodic() {
        Some(surface.v_period()?)
    } else {
        None
    };
    let fc = BRepTopAdaptorFClass2d::new(
        &topods::face(&f2.oriented(Orientation::Forward))?,
        precision::confusion(),
    )?;
    for e in Explorer::new(f1, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edg = topods::edge(&e)?;
        let (c0, mut f1p, mut l1p) = BRepTool::curve(&edg)?;
        let c = geom_proj_lib::curve2d_range(&c0, (f1p, l1p), &s)?;
        if uperiod.is_some() || vperiod.is_some() {
            let eps = BRepTool::edge_tolerance(&edg)?;
            (f1p, l1p) = BRepTool::range(&edg)?;
            if let Some(period) = uperiod {
                put_in_bounds((umin, umax), eps, period, (f1p, l1p), &c, true)?;
            }
            if let Some(period) = vperiod {
                put_in_bounds((vmin, vmax), eps, period, (f1p, l1p), &c, false)?;
            }
        }
        let ac = Geom2dAdaptorCurve::with_range(&c, f1p, l1p)?;
        if !is_in(&fc, &ac)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The extreme `(u, v)` parameters of the corners of `bounds` on a surface.
fn corner_parameters(
    bounds: &BoundingBox3,
    parameters: impl Fn(Point3) -> (f64, f64),
) -> Result<(f64, f64, f64, f64), Failure> {
    let [xmin, ymin, zmin, xmax, ymax, zmax] = bounds.get()?;
    let (x, y, z) = ([xmin, xmax], [ymin, ymax], [zmin, zmax]);
    let (mut umin, mut umax, mut vmin, mut vmax) = (f64::MAX, -f64::MAX, f64::MAX, -f64::MAX);
    for xi in x {
        for yj in y {
            for zk in z {
                let (u, v) = parameters(Point3::new(xi, yj, zk));
                if u < umin {
                    umin = u;
                }
                if u > umax {
                    umax = u;
                }
                if v < vmin {
                    vmin = v;
                }
                if v > vmax {
                    vmax = v;
                }
            }
        }
    }
    Ok((umin, umax, vmin, vmax))
}

/// `BRepFeat::FaceUntil(Sbase, FUntil)`: the face `f_until` extended on its plane,
/// cylinder or cone beyond the box of `sbase` (by ten box diagonals); null for other
/// surfaces; `brep_feat::face_until(&base, &mut face)?`.
pub fn face_until(sbase: &Shape, f_until: &mut Face) -> Result<(), Failure> {
    let mut b = BoundingBox3::default();
    BRepBndLib::add(sbase, &mut b, true)?;
    let diam = 10. * b.square_extent().sqrt();
    let mut s = BRepTool::surface(f_until)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(t) = trimmed.get() {
        s = t.basis_surface();
    }
    let str = if let Some(plane) = s.cast::<GeomPlane>().get() {
        let pln = plane.pln();
        let (umin, umax, vmin, vmax) = corner_parameters(&b, |p| elslib::parameters(&pln, p))?;
        GeomRectangularTrimmedSurface::new(
            &s,
            (umin - diam, umax + diam),
            (vmin - diam, vmax + diam),
            (true, true),
        )?
    } else if let Some(cyl) = s.cast::<GeomCylindricalSurface>().get() {
        let cylinder = cyl.cylinder()?;
        let (_, _, vmin, vmax) = corner_parameters(&b, |p| elslib::parameters(&cylinder, p))?;
        GeomRectangularTrimmedSurface::with_one_direction(
            &s,
            vmin - diam,
            vmax + diam,
            false,
            true,
        )?
    } else if let Some(con) = s.cast::<GeomConicalSurface>().get() {
        let cone = con.cone()?;
        let (_, _, vmin, vmax) = corner_parameters(&b, |p| elslib::parameters(&cone, p))?;
        GeomRectangularTrimmedSurface::with_one_direction(
            &s,
            vmin - diam,
            vmax + diam,
            false,
            true,
        )?
    } else {
        f_until.nullify();
        return Ok(());
    };
    let str: Handle<dyn GeomSurface> = Handle::new(str).cast();
    *f_until = BRepLibMakeFace::from_surface(&str, precision::confusion())?.face()?;
    Ok(())
}

/// `BRepFeat::Tool(SRef, Fac, Orf)`: the solid bounded by the single shell of the
/// faces of `sref`, oriented so that `fac` has the orientation `orf` against it (null
/// when the faces do not make a single shell);
/// `brep_feat::tool(&shape, &face, Orientation::Forward)?`.
pub fn tool(sref: &Shape, fac: &Face, orf: Orientation) -> Result<Solid, Failure> {
    let lfaces = Explorer::new(sref, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    let res = BuildShape::with_faces(&lfaces)?.shape().clone();
    let mut sh = match res.shape_type()? {
        // faire un solide
        ShapeEnum::Shell => res.clone(),
        ShapeEnum::Solid => {
            let mut shells = Explorer::new(&res, ShapeEnum::Shell, ShapeEnum::Shape)?;
            let first = Iterator::next(&mut shells).unwrap_or_default();
            if Iterator::next(&mut shells).is_some() {
                Shape::default()
            } else {
                first
            }
        }
        _ => Shape::default(),
    };
    if sh.is_null() {
        return Ok(Solid::default());
    }
    sh.set_orientation(Orientation::Forward);
    let orient = Explorer::new(&sh, ShapeEnum::Face, ShapeEnum::Shape)?
        .find(|x| x.is_same(fac))
        .map_or(Orientation::Forward, |x| x.orientation());
    let reverse = (orient == fac.orientation() && orf == Orientation::Reversed)
        || (orient != fac.orientation() && orf == Orientation::Forward);
    if reverse {
        sh.reverse();
    }
    let b = BRepBuilder::default();
    let mut soc = Solid::default();
    b.make_solid(&mut soc);
    b.add(&mut soc, &sh)?;
    Ok(soc)
}
