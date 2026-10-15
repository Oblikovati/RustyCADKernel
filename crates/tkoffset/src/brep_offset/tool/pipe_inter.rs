// Rust translation of BRepOffset_Tool::PipeInter, TryProject and InterOrExtent (with the
// static ToSmall and IsOnSurface) of ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The section edges of two faces by the general surface intersection, and the
//! edges of a cap that already lie on another face.
use super::pcurves::{build_pcurves, put_in_bounds};
use super::{BRepOffsetTool, live};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomCurve, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, State};
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_lib::{BRepLibMakeEdge, build_curve3d};

/// `ToSmall(C)`: whether the ends and a point near the middle of the curve are within
/// ten times Precision::Confusion().
fn to_small(c: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
    let curve = live(c, "curve")?;
    let tol = 10. * precision::confusion();
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    let m = f * 0.668 + l * 0.332;
    let (p1, p2, p3) = (curve.value(f)?, curve.value(l)?, curve.value(m)?);
    Ok(p1.distance(&p2) <= tol && p2.distance(&p3) <= tol)
}

/// `IsOnSurface(C, S, TolConf, TolReached)`: whether five points of the curve lie on
/// the elementary surface within the tolerance (false for any other surface); the
/// distance of the last tested point.
fn is_on_surface(
    c: &Handle<dyn GeomCurve>,
    s: &Handle<dyn GeomSurface>,
    tol_conf: f64,
) -> Result<(bool, f64), Failure> {
    let curve = live(c, "curve")?;
    let (f, l) = (curve.first_parameter(), curve.last_parameter());
    let n = 5;
    let du = (f - l) / f64::from(n - 1);
    let as_ = GeomAdaptorSurface::with_surface(s)?;
    let project: Box<dyn Fn(Point3) -> Point3> = match as_.get_type()? {
        SurfaceType::Plane => {
            let ax = as_.plane()?.position();
            Box::new(move |p| {
                let (u, v) = elslib::plane_parameters(ax, p);
                elslib::plane_value(u, v, ax)
            })
        }
        SurfaceType::Cylinder => {
            let cyl = as_.cylinder()?;
            let (ax, rad) = (cyl.position(), cyl.radius());
            Box::new(move |p| {
                let (u, v) = elslib::cylinder_parameters(ax, rad, p);
                elslib::cylinder_value(u, v, ax, rad)
            })
        }
        SurfaceType::Cone => {
            let cone = as_.cone()?;
            let (ax, rad, alp) = (cone.position(), cone.ref_radius(), cone.semi_angle());
            Box::new(move |p| {
                let (u, v) = elslib::cone_parameters(ax, rad, alp, p);
                elslib::cone_value(u, v, ax, rad, alp)
            })
        }
        SurfaceType::Sphere => {
            let sphere = as_.sphere()?;
            let (ax, rad) = (sphere.position(), sphere.radius());
            Box::new(move |p| {
                let (u, v) = elslib::sphere_parameters(ax, rad, p);
                elslib::sphere_value(u, v, ax, rad)
            })
        }
        SurfaceType::Torus => {
            let torus = as_.torus()?;
            let (ax, r1, r2) = (torus.position(), torus.major_radius(), torus.minor_radius());
            Box::new(move |p| {
                let (u, v) = elslib::torus_parameters(ax, r1, r2, p);
                elslib::torus_value(u, v, ax, r1, r2)
            })
        }
        _ => return Ok((false, 0.)),
    };
    let mut tol_reached = 0.;
    for i in 0..n {
        let p = curve.value(f + f64::from(i) * du)?;
        tol_reached = p.distance(&project(p));
        if tol_reached > tol_conf {
            return Ok((false, tol_reached));
        }
    }
    Ok((true, tol_reached))
}

/// Appends the section edge oriented in each face (both reversed on the OUT side).
pub(super) fn append_section(
    e: &Edge,
    f1: &Face,
    f2: &Face,
    side: State,
    (l1, l2): (&mut List<Shape>, &mut List<Shape>),
) -> Result<(), Failure> {
    let (mut o1, mut o2) = BRepOffsetTool::orient_section(e, f1, f2)?;
    if side == State::Out {
        o1 = top_abs::reverse(o1);
        o2 = top_abs::reverse(o2);
    }
    l1.append(e.oriented(o1));
    l2.append(e.oriented(o2));
    Ok(())
}

impl BRepOffsetTool {
    /// `PipeInter(F1, F2, L1, L2, Side)`: the section edges of the surfaces of two faces
    /// (approximated with their p-curves), the small ones skipped, oriented in each face;
    /// `BRepOffsetTool::pipe_inter(&f1, &f2, &mut l1, &mut l2, State::Out)?`.
    pub fn pipe_inter(
        f1: &Face,
        f2: &Face,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
        side: State,
    ) -> Result<(), Failure> {
        l1.clear();
        l2.clear();
        let s1 = BRepTool::surface(f1)?;
        let s2 = BRepTool::surface(f2)?;
        let approx = GeomIntIntSSApprox {
            approx: true,
            approx_s1: true,
            approx_s2: true,
        };
        let inter = GeomIntIntSS::with_surfaces(&s1, &s2, precision::confusion(), approx)?;
        if !inter.is_done() {
            return Ok(());
        }
        let builder = BRepBuilder::default();
        for i in 1..=inter.nb_lines() {
            let ci = inter.line(i)?;
            if to_small(&ci)? {
                continue;
            }
            let e = BRepLibMakeEdge::from_curve(&ci)?.edge()?;
            for (on_first, f) in [(true, f1), (false, f2)] {
                let has = if on_first {
                    inter.has_line_on_s1(i)?
                } else {
                    inter.has_line_on_s2(i)?
                };
                if !has {
                    build_pcurves(&e, f)?;
                    continue;
                }
                let c2 = if on_first {
                    inter.line_on_s1(i)?
                } else {
                    inter.line_on_s2(i)?
                };
                put_in_bounds(f, &e, &c2)?;
                builder.update_edge_pcurve_on_face(&e, &c2, f, BRepTool::edge_tolerance(&e)?)?;
            }
            append_section(&e, f1, f2, side, (l1, l2))?;
        }
        Ok(())
    }

    /// `TryProject(F1, F2, Edges, LInt1, LInt2, Side, TolConf)`: the edges lying on the
    /// elementary surface of `F1` (their tolerance raised to the reached one, given
    /// p-curves on `F1`) oriented as sections; whether all edges did (edges without 3d
    /// curve are skipped); `BRepOffsetTool::try_project(&f1, &f2, &edges, &mut l1, &mut l2, State::Out, 1e-7)?`.
    pub fn try_project(
        f1: &Face,
        f2: &Face,
        edges: &List<Shape>,
        l_int1: &mut List<Shape>,
        l_int2: &mut List<Shape>,
        side: State,
        tol_conf: f64,
    ) -> Result<bool, Failure> {
        l_int1.clear();
        l_int2.clear();
        let mut ok = true;
        let bouchon = BRepTool::surface(f1)?;
        for shape in edges.iter() {
            let cur_e = topods::edge(shape)?;
            let Some(c) = located_trimmed_curve(&cur_e)? else {
                continue;
            };
            let (is_ok, tol_reached) = is_on_surface(&c, &bouchon, tol_conf)?;
            if !is_ok {
                ok = false;
                continue;
            }
            BRepBuilder::default().update_edge_tolerance(&cur_e, tol_reached)?;
            build_pcurves(&cur_e, f1)?;
            append_section(&cur_e, f1, f2, side, (l_int1, l_int2))?;
        }
        Ok(ok)
    }

    /// `InterOrExtent(F1, F2, L1, L2, Side)`: the section edges of the surfaces (the
    /// planes of trimmed planes) with projected p-curves, the small ones skipped;
    /// `BRepOffsetTool::inter_or_extent(&f1, &f2, &mut l1, &mut l2, State::Out)?`.
    pub fn inter_or_extent(
        f1: &Face,
        f2: &Face,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
        side: State,
    ) -> Result<(), Failure> {
        l1.clear();
        l2.clear();
        let s1 = untrimmed_plane(BRepTool::surface(f1)?)?;
        let s2 = untrimmed_plane(BRepTool::surface(f2)?)?;
        let inter = GeomIntIntSS::with_surfaces(
            &s1,
            &s2,
            precision::confusion(),
            GeomIntIntSSApprox::default(),
        )?;
        if !inter.is_done() {
            return Ok(());
        }
        for i in 1..=inter.nb_lines() {
            let ci = inter.line(i)?;
            if to_small(&ci)? {
                continue;
            }
            let e = BRepLibMakeEdge::from_curve(&ci)?.edge()?;
            build_pcurves(&e, f1)?;
            build_pcurves(&e, f2)?;
            append_section(&e, f1, f2, side, (l1, l2))?;
        }
        Ok(())
    }
}

/// The 3d curve of the edge trimmed to its range and located, built when missing;
/// `None` when the edge still has none (a degenerated edge).
fn located_trimmed_curve(cur_e: &Edge) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
    let mut found = BRepTool::curve_with_location(cur_e)?;
    if found.curve.is_null() {
        let (continuity, max_degree, max_segment) =
            rustycad_tktopalgo::brep_lib::BUILD_CURVE3D_DEFAULTS;
        build_curve3d(
            cur_e,
            BRepTool::edge_tolerance(cur_e)?,
            continuity,
            max_degree,
            max_segment,
        )?;
        found = BRepTool::curve_with_location(cur_e)?;
        if found.curve.is_null() {
            return Ok(None);
        }
    }
    let c: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
        &found.curve,
        found.first,
        found.last,
        true,
        true,
    )?)
    .cast();
    if !found.location.is_identity() {
        live(&c, "curve")?.transform(found.location.transformation())?;
    }
    Ok(Some(c))
}

/// The basis plane of a rectangular trimmed plane, any other surface unchanged.
fn untrimmed_plane(s: Handle<dyn GeomSurface>) -> Result<Handle<dyn GeomSurface>, Failure> {
    if !s.is_instance_name("Geom_RectangularTrimmedSurface") {
        return Ok(s);
    }
    let rts = s.clone().cast::<GeomRectangularTrimmedSurface>();
    let basis = live(&rts, "surface")?.basis_surface();
    if basis.is_instance_name("Geom_Plane") {
        return Ok(basis);
    }
    Ok(s)
}
