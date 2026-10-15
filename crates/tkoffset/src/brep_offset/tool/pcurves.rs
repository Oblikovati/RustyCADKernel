// Rust translation of the static FindPeriod, PutInBounds and BuildPCurves of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The p-curves of section edges: built on a bound of a spline surface or projected,
//! and moved into the parameter domain of a periodic face.
use super::live;
use rustycad_tkbool::top_ope_brep_tool::make_pcurve;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d as Adaptor2dCurve;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomOffsetSurface, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tktopalgo::brep_lib::{BRepLibMakeFace, same_range};

/// `FindPeriod(F, umin, umax, vmin, vmax)`: the parameter box of the p-curves of the
/// face (20 samples per curve, 2 for a line); `None` when an edge has no p-curve
/// before any box was read (the native outputs stay uninitialized then).
fn find_period(f: &Face) -> Result<Option<[f64; 4]>, Failure> {
    let mut b = BoundingBox2::default();
    let mut bounds = None;
    let mut exp = Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exp.more() {
        let e = topods::edge(exp.current())?;
        exp.next()?;
        let pc = BRepTool::curve_on_face(&e, f)?;
        if pc.pcurve.is_null() {
            return Ok(bounds);
        }
        add_samples(&mut b, &pc.pcurve, pc.first, pc.last)?;
        let [umin, vmin, umax, vmax] = b.get()?;
        bounds = Some([umin, umax, vmin, vmax]);
    }
    Ok(bounds)
}

/// Adds the sample points of `FindPeriod` of one p-curve to the box.
fn add_samples(
    b: &mut BoundingBox2,
    c: &Handle<dyn Geom2dCurve>,
    pf: f64,
    pl: f64,
) -> Result<(), Failure> {
    let pc = Geom2dAdaptorCurve::with_range(c, pf, pl)?;
    let nbp: f64 = if pc.get_type()? == CurveType::Line {
        2.
    } else {
        20.
    };
    let step = (pl - pf) / nbp;
    let mut u = pf;
    b.add_point(pc.value(u)?);
    let mut i = 2.;
    while i < nbp {
        u += step;
        b.add_point(pc.value(u)?);
        i += 1.;
    }
    b.add_point(pc.value(pl)?);
    Ok(())
}

/// `PutInBounds(F, E, C2d)`: translates the p-curve by periods so that it lies in the
/// parameter box of the face of a periodic surface.
pub(crate) fn put_in_bounds(
    f: &Face,
    e: &Edge,
    c2d: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    let (first, last) = BRepTool::range(e)?;
    let (mut s, _) = BRepTool::surface_with_location(f)?;
    if s.is_instance_name("Geom_RectangularTrimmedSurface") {
        s = live(
            &s.clone().cast::<GeomRectangularTrimmedSurface>(),
            "surface",
        )?
        .basis_surface();
    }
    let surface = live(&s, "surface")?;
    if !surface.is_u_periodic() && !surface.is_v_periodic() {
        return Ok(());
    }
    let Some([umin, umax, vmin, vmax]) = find_period(f)? else {
        return Ok(());
    };
    if surface.is_u_periodic() {
        recadre(c2d, (first, last), (umin, umax), surface.u_period()?, true)?;
    }
    if surface.is_v_periodic() {
        recadre(c2d, (first, last), (vmin, vmax), surface.v_period()?, false)?;
    }
    Ok(())
}

/// The U (or V) move of `PutInBounds`: whole periods bringing the curve into the
/// domain, then one period back when it overflows more than it would underflow.
fn recadre(
    c2d: &Handle<dyn Geom2dCurve>,
    (f, l): (f64, f64),
    (dmin, dmax): (f64, f64),
    period: f64,
    along_u: bool,
) -> Result<(), Failure> {
    let c = live(c2d, "pcurve")?;
    let eps = period * 1.0e-6;
    let coord = |p: Point2| if along_u { p.x() } else { p.y() };
    let (pf, pl, pm) = (
        coord(c.value(f)?),
        coord(c.value(l)?),
        coord(c.value(0.34 * f + 0.66 * l)?),
    );
    let mut min_c = pf.min(pl).min(pm);
    let mut max_c = pf.max(pl).max(pm);
    let mut du = 0.;
    if min_c < dmin - eps {
        du = (((dmin - min_c) / period) as i32 + 1) as f64 * period;
    }
    if min_c > dmax + eps {
        du = -((((min_c - dmax) / period) as i32 + 1) as f64) * period;
    }
    if du != 0. {
        c.translate(shift(du, along_u))?;
        min_c += du;
        max_c += du;
    }
    if max_c > dmax + 100. * eps {
        let d1 = max_c - dmax;
        let d2 = dmin - min_c + period;
        if d2 < d1 {
            du = -period;
        }
        if du != 0. {
            c.translate(shift(du, along_u))?;
        }
    }
    Ok(())
}

/// The translation of `du` along U or V.
fn shift(du: f64, along_u: bool) -> Vector2 {
    if along_u {
        Vector2::new(du, 0.)
    } else {
        Vector2::new(0., du)
    }
}

/// `BuildPCurves(E, F)`: nothing when the edge has a p-curve on the face; else the
/// p-curve on a bound of a B-spline or Bezier surface when the edge lies on one (within
/// 1e-4), or the projection of the edge (Standard_ConstructionError when it has
/// none), moved into the periodic domain.
pub(crate) fn build_pcurves(e: &Edge, f: &Face) -> Result<(), Failure> {
    if !BRepTool::curve_on_face(e, f)?.pcurve.is_null() {
        return Ok(());
    }
    let tolerance = precision::confusion();
    let as_surface = BRepAdaptorSurface::with_face(f, false)?;
    let ac = BRepAdaptorCurve::with_edge(e)?;
    let the_surf = BRepTool::surface(f)?;
    if is_spline_surface(&the_surf)? && bound_pcurve(e, f, &the_surf, &ac)? {
        return Ok(());
    }
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(BRepAdaptorSurface::with_face(f, false)?).cast();
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(ac).cast();
    let proj = ProjectedCurve::with_tolerance(&hs, &hc, tolerance)?;
    let c2d = projected_pcurve(&proj)?;
    if as_surface.is_u_periodic()? || as_surface.is_v_periodic()? {
        put_in_bounds(f, e, &c2d)?;
    }
    if c2d.is_null() {
        return Err(Failure::with_message(
            "Standard_ConstructionError",
            "BRepOffset_Tool::BuildPCurves",
        ));
    }
    BRepBuilder::default().update_edge_pcurve_on_face(e, &c2d, f, BRepTool::edge_tolerance(e)?)
}

/// The Geom2d curve of a projection of an elementary or spline type, a null handle
/// for any other type.
fn projected_pcurve(proj: &ProjectedCurve) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    match proj.get_type()? {
        CurveType::Line
        | CurveType::Circle
        | CurveType::Ellipse
        | CurveType::Parabola
        | CurveType::Hyperbola
        | CurveType::BSplineCurve => make_pcurve(proj),
        CurveType::BezierCurve => Ok(proj.bezier()?.cast()),
        _ => Ok(Handle::default()),
    }
}

/// Whether the surface (or the basis of an offset surface) is a Bezier or B-spline
/// surface.
fn is_spline_surface(the_surf: &Handle<dyn GeomSurface>) -> Result<bool, Failure> {
    let mut typ = the_surf.clone();
    if typ.is_instance_name("Geom_OffsetSurface") {
        typ = live(&typ.clone().cast::<GeomOffsetSurface>(), "surface")?.basis_surface();
    }
    Ok(typ.is_instance_name("Geom_BezierSurface") || typ.is_instance_name("Geom_BSplineSurface"))
}

/// The nearest-point parameter of the point on a boundary edge curve and its squared
/// distance, when the extrema find one.
fn nearest_on(
    point: rustycad_tkmath::gp::Point3,
    curve: &BRepAdaptorCurve,
) -> Result<Option<(f64, f64)>, Failure> {
    let extr = ExtPC::with_curve(point, curve, 1.0e-10)?;
    if !extr.is_done() || extr.nb_ext()? < 1 {
        return Ok(None);
    }
    let mut best: Option<(f64, f64)> = None;
    let mut dist2min = f64::MAX;
    for i in 1..=extr.nb_ext()? {
        let dist2 = extr.square_distance(i)?;
        if dist2 < dist2min {
            dist2min = dist2;
            best = Some((extr.point(i)?.parameter(), dist2));
        }
    }
    Ok(best)
}

/// The spline-bound branch of `BuildPCurves`: the trimmed p-curve of the boundary edge
/// of the natural face of the surface on which both ends of the edge lie; whether one
/// was found and set.
fn bound_pcurve(
    e: &Edge,
    f: &Face,
    the_surf: &Handle<dyn GeomSurface>,
    ac: &BRepAdaptorCurve,
) -> Result<bool, Failure> {
    let fpoint = ac.value(ac.first_parameter()?)?;
    let lpoint = ac.value(ac.last_parameter()?)?;
    let the_face = BRepLibMakeFace::from_surface(the_surf, precision::confusion())?.face()?;
    let tol_proj: f64 = 1.0e-4;
    let mut explo = Explorer::new(the_face.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explo.more() {
        let an_edge = topods::edge(explo.current())?;
        explo.next()?;
        let a_curve = BRepAdaptorCurve::with_edge(&an_edge)?;
        let Some((u1, d1)) = nearest_on(fpoint, &a_curve)? else {
            continue;
        };
        if d1 > tol_proj * tol_proj {
            continue;
        }
        let Some((u2, d2)) = nearest_on(lpoint, &a_curve)? else {
            continue;
        };
        if d2 <= tol_proj * tol_proj {
            set_bound_pcurve(e, f, (&an_edge, &the_face), (u1, u2))?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Sets the p-curve of the boundary edge trimmed to `[min(u1, u2), max(u1, u2)]` on
/// the edge, then makes its range the same.
fn set_bound_pcurve(
    e: &Edge,
    f: &Face,
    (the_edge, the_face): (&Edge, &Face),
    (u1, u2): (f64, f64),
) -> Result<(), Failure> {
    let (u1, u2) = if u2 < u1 { (u2, u1) } else { (u1, u2) };
    let basis = BRepTool::curve_on_face(the_edge, the_face)?.pcurve;
    let c2d: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&basis, u1, u2, true, true)?).cast();
    let surface = BRepTool::surface(f)?;
    let s = live(&surface, "surface")?;
    if s.is_u_periodic() || s.is_v_periodic() {
        put_in_bounds(f, e, &c2d)?;
    }
    BRepBuilder::default().update_edge_pcurve_on_face(e, &c2d, f, BRepTool::edge_tolerance(e)?)?;
    same_range(e, 1.0e-5)
}
