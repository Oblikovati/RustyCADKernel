//! The InterUtils operation of the IntCurveSurface probe (`ics_interutils`), split from
//! int_curve_surface_inter.rs for size; the Rust twin of `ga_ics_interutils_case` and its
//! helpers in tools/oracle/tkgeomalgo_probe_intcurvesurface_inter.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::ics_geometry::{curve, surface};
use super::ics_polyhedron::polyhedron;
use super::int_curve_surface::{ipoint, polygon};
use super::intf::spoint;
use super::intf_tool::box3d_print;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_curve_surface::inter_utils::{
    self as iu, SortedStartPoints, UvLimits,
};
use rustycad_tkgeomalgo::int_curve_surface::{
    HCurve, IntersectionPoint, TheCSFunctionOfHInter, TheExactHInter, TheHCurveTool,
    TheInterferenceOfHInter,
};
use rustycad_tkgeomalgo::int_imp::HSurfaceTool;
use rustycad_tkgeombase::int_ana::{IntConicQuad, Quadric};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math::FunctionSetRoot;

type C = TheHCurveTool;
type S = HSurfaceTool;

fn points_print(pts: &[IntersectionPoint]) {
    int(pts.len() as i32);
    pts.iter().for_each(ipoint);
}

fn start_print(sp: &SortedStartPoints) {
    int(sp.size());
    for i in 0..sp.tab_u.len() {
        real(sp.tab_u[i]);
        real(sp.tab_v[i]);
        real(sp.tab_w[i]);
    }
}

fn line_quadric(c: &HCurve, s: &Adaptor3dSurfaceHandle) -> Result<IntConicQuad, Failure> {
    let l = c.line()?;
    match s.get_type()? {
        SurfaceType::Plane => IntConicQuad::line_plane(l, s.plane()?, 1e-12, 0.0, 0.0),
        SurfaceType::Cylinder => {
            IntConicQuad::line_quadric(l, &Quadric::from_cylinder(s.cylinder()?))
        }
        SurfaceType::Cone => IntConicQuad::line_quadric(l, &Quadric::from_cone(s.cone()?)),
        _ => IntConicQuad::line_quadric(l, &Quadric::from_sphere(s.sphere()?)),
    }
}

fn sampling(r: &mut Reader, s: &Adaptor3dSurfaceHandle) -> Result<(), Failure> {
    let (u0, u1, v0, v1) = (r.real(), r.real(), r.real(), r.real());
    let mut xyz_bounds: [Array1<'static, f64>; 3] =
        [0; 3].map(|_| Array1::with_bounds(1, 2).expect("probe array"));
    for a in xyz_bounds.iter_mut() {
        a[1] = r.real();
        a[2] = r.real();
    }
    let mut pts: Array2<'static, Point3> = Array2::with_bounds(1, 50, 1, 50).expect("probe grid");
    let mut bnd = BoundingBox3::default();
    let gap = iu::do_surface::<S>(s, [u0, u1, v0, v1], &mut pts, &mut bnd)?;
    real(gap);
    box3d_print(&bnd);
    for (i, j) in [(1, 1), (1, 50), (25, 25), (13, 37), (50, 1), (50, 50)] {
        pnt(pts[(i, j)]);
    }
    let [x, y, z] = &xyz_bounds;
    iu::do_new_bounds::<S>(s, [u0, u1, v0, v1], &pts, [x, y, z])?
        .into_iter()
        .for_each(real);
    Ok(())
}

fn refine(r: &mut Reader, c: &HCurve, s: &Adaptor3dSurfaceHandle) -> Result<(), Failure> {
    let pg = polygon(r, c)?;
    let ph = polyhedron(r, s)?;
    let (u0, u1, v0, v1) = (r.real(), r.real(), r.real(), r.real());
    let itf = TheInterferenceOfHInter::with_polygon(&pg, &ph)?;
    let mut sp = iu::collect_interference_points(&itf, &ph, &pg)?;
    start_print(&sp);
    iu::sort_start_points(&mut sp);
    start_print(&sp);
    let f = TheCSFunctionOfHInter::new(s, c);
    let mut e = TheExactHInter::from_function(&f, 1e-8);
    let mut rs = FunctionSetRoot::without_tolerance(e.function(), 100)?;
    let bounds = [u0, u1, v0, v1, pg.inf_parameter(), pg.sup_parameter()];
    let pts = iu::process_sorted_points::<C, S>(&mut e, &mut rs, &sp, bounds, c, s)?;
    points_print(&pts);
    Ok(())
}

fn interutils(
    r: &mut Reader,
    c: &HCurve,
    s: &Adaptor3dSurfaceHandle,
    op: i32,
) -> Result<(), Failure> {
    match op {
        0 => {
            let (w, u, v) = (r.real(), r.real(), r.real());
            int(iu::compute_transitions::<C, S>(c, w, s, u, v)? as i32);
        }
        1 => {
            let p = r.pnt();
            let (mut u, mut v) = (r.real(), r.real());
            iu::compute_params_on_quadric::<S>(s, p, &mut u, &mut v)?;
            real(u);
            real(v);
        }
        2 => {
            let (w, u, v) = (r.real(), r.real(), r.real());
            let point = iu::compute_append_point::<C, S>(c, w, s, u, v)?;
            flag(point.is_some());
            ipoint(&point.unwrap_or_default());
        }
        3 => {
            let result = iu::process_int_ana::<C, S>(c, s, &line_quadric(c, s)?)?;
            flag(result.is_some());
            flag(matches!(result, Some(None)));
            points_print(&result.flatten().unwrap_or_default());
        }
        4 => points_print(&iu::perform_curve_quadric::<C, S>(c, s)?),
        5 => {
            let found = iu::process_lin_torus::<C, S>(c.line()?, c, s)?;
            flag(found.is_some());
            points_print(&found.unwrap_or_default());
        }
        _ => return interutils_rest(r, c, s, op),
    }
    Ok(())
}

fn interutils_rest(
    r: &mut Reader,
    c: &HCurve,
    s: &Adaptor3dSurfaceHandle,
    op: i32,
) -> Result<(), Failure> {
    match op {
        6 => {
            let mut sp = SortedStartPoints::default();
            for _ in 0..r.integer() {
                let (u, v) = (r.real(), r.real());
                sp.append(u, v, r.real());
            }
            iu::sort_start_points(&mut sp);
            start_print(&sp);
        }
        7 => {
            let mut b = UvLimits {
                u1: r.real(),
                u2: r.real(),
                v1: r.real(),
                v2: r.real(),
            };
            iu::clamp_uv_parameters(&mut b);
            [b.u1, b.u2, b.v1, b.v2].into_iter().for_each(real);
        }
        8 => {
            let intervals = iu::decompose_surface_intervals::<S>(s)?;
            int(intervals.len() as i32);
            for b in intervals {
                [b.u0, b.u1, b.v0, b.v1].into_iter().for_each(real);
            }
        }
        9 => sampling(r, s)?,
        10 => {
            let pg = polygon(r, c)?;
            let ph = polyhedron(r, s)?;
            for _ in 0..r.integer() {
                let sp = spoint(r);
                attempt(|| {
                    iu::section_point_to_parameters(&sp, &ph, &pg)
                        .map(|(u, v, w)| (real(u), real(v), real(w)).0)
                });
            }
        }
        _ => refine(r, c, s)?,
    }
    Ok(())
}

/// `curve surface op ...`: the IntCurveSurface_InterUtils point functions.
pub fn interutils_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (c, s) = (curve(&mut r), surface(&mut r));
    let op = r.integer();
    attempt(|| interutils(&mut r, &c, &s, op));
    end();
}
