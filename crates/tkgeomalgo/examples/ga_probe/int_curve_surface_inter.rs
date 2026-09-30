//! IntCurveSurface probe operations over the interference, the CS function and exact
//! intersector, the quadric intersector, HInter and the InterUtils functions; the Rust
//! twin of tools/oracle/tkgeomalgo_probe_intcurvesurface_inter.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::ics_geometry::{curve, params, surface};
use super::int_curve_surface::{intersection_print, ipoint, polygon, polyhedron};
use super::int_imp::{matrix, vector};
use super::int_surf::quadric;
use super::intf::{interference_print, spoint};
use super::intf_poly::lines;
use super::intf_tool::box3d_print;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_curve_surface::inter_utils::{
    self as iu, InfFlags, SortedStartPoints, UvLimits,
};
use rustycad_tkgeomalgo::int_curve_surface::{
    HCurve, HInter, IntersectionPoint, TheCSFunctionOfHInter, TheExactHInter, TheHCurveTool,
    TheInterferenceOfHInter, ThePolygonOfHInter, ThePolyhedronOfHInter, TheQuadCurvExactHInter,
    TheQuadCurvFuncOfTheQuadCurvExactHInter,
};
use rustycad_tkgeomalgo::int_imp::HSurfaceTool;
use rustycad_tkgeombase::int_ana::{IntConicQuad, Quadric};
use rustycad_tkgeombase::proj_lib::{Plane as ProjLibPlane, Project};
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{
    Axis2, Axis3, Circle3, Direction3, Ellipse3, Hyperbola3, Line3, Parabola3, Plane3, Point3,
    Sphere3,
};
use rustycad_tkmath::math::{FunctionSetRoot, Matrix, Vector};

type C = TheHCurveTool;
type S = HSurfaceTool;

fn grid_of(ph: &ThePolyhedronOfHInter) -> BoundSortBox {
    let mut grid = BoundSortBox::default();
    grid.initialize_with_enclosing(ph.bounding(), ph.components_bounding().clone())
        .expect("probe polyhedron grid");
    grid
}

fn interference_operation(
    r: &mut Reader,
    x: &mut TheInterferenceOfHInter,
    pg: &ThePolygonOfHInter,
    ph: &ThePolyhedronOfHInter,
    kind: i32,
) {
    type X = TheInterferenceOfHInter;
    let (base, make) = if kind >= 8 {
        (kind - 8, true)
    } else {
        (kind, false)
    };
    let mut grid = grid_of(ph);
    let replace = |x: &mut X, made: Result<X, Failure>| made.map(|made| *x = made);
    if kind == 6 {
        return attempt(|| x.interference(pg, ph));
    }
    if kind == 7 {
        return attempt(|| x.interference_grid(pg, ph, &mut grid));
    }
    match base {
        0 => attempt(|| {
            if make {
                replace(x, X::with_polygon(pg, ph))
            } else {
                x.perform_polygon(pg, ph)
            }
        }),
        3 => attempt(|| {
            if make {
                replace(x, X::with_polygon_grid(pg, ph, &mut grid))
            } else {
                x.perform_polygon_grid(pg, ph, &mut grid)
            }
        }),
        1 | 4 => {
            let p = r.pnt();
            let l = Line3::new(p, r.dir());
            attempt(|| match (base, make) {
                (1, true) => replace(x, X::with_line(l, ph)),
                (1, false) => x.perform_line(l, ph),
                (_, true) => replace(x, X::with_line_grid(l, ph, &mut grid)),
                _ => x.perform_line_grid(l, ph, &mut grid),
            })
        }
        _ => {
            let ls = lines(r);
            attempt(|| match (base, make) {
                (2, true) => replace(x, X::with_lines(&ls, ph)),
                (2, false) => x.perform_lines(&ls, ph),
                (_, true) => replace(x, X::with_lines_grid(&ls, ph, &mut grid)),
                _ => x.perform_lines_grid(&ls, ph, &mut grid),
            })
        }
    }
}

/// `curve polygon surface polyhedron count (kind operand)...`: the interference members.
pub fn interference_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let c = curve(&mut r);
    let pg = polygon(&mut r, &c).expect("probe polygon");
    let s = surface(&mut r);
    let ph = polyhedron(&mut r, &s).expect("probe polyhedron");
    let mut x = TheInterferenceOfHInter::new();
    for _ in 0..r.integer() {
        let kind = r.integer();
        interference_operation(&mut r, &mut x, &pg, &ph, kind);
        interference_print(&x);
    }
    end();
}

fn exact_print(e: &mut TheExactHInter) {
    flag(e.is_done());
    attempt(|| e.is_empty().map(flag));
    attempt(|| e.point().map(pnt));
    attempt(|| e.parameter_on_surface().map(|(a, b)| (real(a), real(b)).0));
    attempt(|| e.parameter_on_curve().map(real));
    real(e.function().root());
}

fn exact_function(f: &mut TheCSFunctionOfHInter, [u, v, w]: [f64; 3]) -> Result<(), Failure> {
    int(f.nb_variables());
    int(f.nb_equations());
    let mut x: Vector<'static> = Vector::filled(1, 3, 0.0)?;
    (x[1], x[2], x[3]) = (u, v, w);
    let mut fv: Vector<'static> = Vector::filled(1, 3, 0.0)?;
    let mut d: Matrix<'static> = Matrix::filled(1, 3, 1, 3, 0.0).expect("probe matrix");
    attempt(|| {
        f.value(&x, &mut fv)
            .map(|ok| (flag(ok), vector(&fv), pnt(f.point()), real(f.root())).0)
    });
    attempt(|| f.derivatives(&x, &mut d).map(|ok| (flag(ok), matrix(&d)).0));
    x[1] += 0.01;
    attempt(|| {
        f.values(&x, &mut fv, &mut d)
            .map(|ok| (flag(ok), vector(&fv), matrix(&d), real(f.root())).0)
    });
    real(f.auxillar_surface().first_u_parameter()?);
    real(f.auxillar_curve().last_parameter()?);
    Ok(())
}

/// `surface curve u v w tol margin bounds(6)`: the CS function, the exact intersector
/// from the point, and Perform of a bare intersector within the bounds.
pub fn exact_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s, c) = (surface(&mut r), curve(&mut r));
    let (u, v, w, tol, margin) = (r.real(), r.real(), r.real(), r.real(), r.real());
    let b: [f64; 6] = [0; 6].map(|_| r.real());
    attempt(|| {
        let mut f = TheCSFunctionOfHInter::new(&s, &c);
        exact_function(&mut f, [u, v, w])?;
        attempt(|| {
            TheExactHInter::new([u, v, w], &f, tol, margin).map(|mut e| exact_print(&mut e))
        });
        attempt(|| {
            let mut e = TheExactHInter::from_function(&f, tol);
            flag(e.is_done());
            attempt(|| e.is_empty().map(flag));
            let mut rs = FunctionSetRoot::without_tolerance(e.function(), 100)?;
            e.perform([u, v, w], &mut rs, b)?;
            exact_print(&mut e);
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `surface curve quadric n params...`: the quadric intersector, then the distance
/// function on the quadric at each parameter.
pub fn quadcurv_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (s, c) = (surface(&mut r), curve(&mut r));
    let q = quadric(&mut r);
    let pars = params(&mut r);
    attempt(|| {
        let e = TheQuadCurvExactHInter::new(&s, &c)?;
        flag(e.is_done());
        let (n, m) = (e.nb_roots(), e.nb_intervals());
        int(n);
        int(m);
        for i in 0..=n.max(0) + 1 {
            attempt(|| e.root(i).map(real));
        }
        for i in 0..=m.max(0) + 1 {
            attempt(|| e.intervals(i).map(|(a, b)| (real(a), real(b)).0));
        }
        Ok(())
    });
    let mut f = TheQuadCurvFuncOfTheQuadCurvExactHInter::new(&q, &c);
    for &t in pars.iter() {
        attempt(|| f.value(t).map(|y| (flag(true), real(y)).0));
        attempt(|| f.derivative(t).map(|y| (flag(true), real(y)).0));
        attempt(|| {
            let mut y = -7.0;
            let z = f.values(t, &mut y)?;
            flag(true);
            real(y);
            real(z);
            Ok(())
        });
    }
    end();
}

fn hinter_perform(
    r: &mut Reader,
    x: &mut HInter,
    c: &HCurve,
    s: &Adaptor3dSurfaceHandle,
    mode: i32,
) -> Result<(), Failure> {
    if mode == 0 {
        return x.perform(c, s);
    }
    if mode == 4 {
        let ph = polyhedron(r, s)?;
        return x.perform_polyhedron(c, s, &ph);
    }
    let pg = polygon(r, c)?;
    if mode == 1 {
        return x.perform_polygon(c, &pg, s);
    }
    let ph = polyhedron(r, s)?;
    if mode == 2 {
        return x.perform_polygon_polyhedron(c, &pg, s, &ph);
    }
    let mut grid = grid_of(&ph);
    x.perform_polygon_polyhedron_grid(c, &pg, s, &ph, &mut grid)
}

/// `curve surface mode ...`: HInter::Perform and its polygon and polyhedron variants.
pub fn hinter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (c, s) = (curve(&mut r), surface(&mut r));
    let mut x = HInter::new();
    let mode = r.integer();
    attempt(|| hinter_perform(&mut r, &mut x, &c, &s, mode));
    intersection_print(&x);
    end();
}

fn conic_surf(
    r: &mut Reader,
    x: &mut HInter,
    c: &HCurve,
    s: &Adaptor3dSurfaceHandle,
) -> Result<(), Failure> {
    let k = r.integer();
    let (p, d) = (r.pnt(), r.dir());
    let (a, b) = (r.real(), r.real());
    let (u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real());
    let ax = Axis2::new(p, d)?;
    match k {
        0 => x.perform_conic_surf_line(Line3::new(p, d), c, s, u1, v1, u2, v2),
        1 => x.perform_conic_surf_circle(Circle3::new(ax, a)?, c, s, u1, v1, u2, v2),
        2 => x.perform_conic_surf_ellipse(Ellipse3::new(ax, a, b)?, c, s, u1, v1, u2, v2),
        3 => x.perform_conic_surf_parabola(Parabola3::new(ax, a)?, c, s, u1, v1, u2, v2),
        _ => x.perform_conic_surf_hyperbola(Hyperbola3::new(ax, a, b)?, c, s, u1, v1, u2, v2),
    }
}

fn int_ana(r: &mut Reader) -> Result<IntConicQuad, Failure> {
    let p = r.pnt();
    let l = Line3::new(p, r.dir());
    let k = r.integer();
    let q = r.pnt();
    if k == 0 {
        let pl = Plane3::new(q, r.dir())?;
        return IntConicQuad::line_plane(l, pl, r.real(), 0.0, 0.0);
    }
    let sphere = Sphere3::new(
        Axis3::from_normal(q, Direction3::new(0.0, 0.0, 1.0)?)?,
        r.real(),
    )?;
    IntConicQuad::line_quadric(l, &Quadric::from_sphere(sphere))
}

fn hinter_protected(
    r: &mut Reader,
    x: &mut HInter,
    c: &HCurve,
    s: &Adaptor3dSurfaceHandle,
    op: i32,
) -> Result<(), Failure> {
    let four = |r: &mut Reader| (r.real(), r.real(), r.real(), r.real());
    match op {
        0 => {
            let (u1, v1, u2, v2) = four(r);
            x.perform_bounds(c, s, u1, v1, u2, v2)
        }
        1 => x.internal_perform_curve_quadric(c, s),
        2..=4 => {
            let pg = polygon(r, c)?;
            if op == 4 {
                let (u1, v1, u2, v2) = four(r);
                return x.internal_perform_polygon(c, &pg, s, u1, v1, u2, v2);
            }
            let ph = polyhedron(r, s)?;
            let (u1, v1, u2, v2) = four(r);
            if op == 2 {
                return x.internal_perform_polyhedron(c, &pg, s, &ph, u1, v1, u2, v2);
            }
            let mut grid = grid_of(&ph);
            x.internal_perform_polyhedron_grid(c, &pg, s, &ph, [u1, v1, u2, v2], &mut grid)
        }
        5 => conic_surf(r, x, c, s),
        6 => {
            let ana = int_ana(r)?;
            x.append_int_ana(c, s, &ana)
        }
        7 => {
            let (w, u) = (r.real(), r.real());
            x.append_curve_point(c, w, s, u, r.real())
        }
        _ => {
            let a = r.real();
            x.append_curve_segment(c, a, r.real(), s);
            Ok(())
        }
    }
}

/// `curve surface op ...` on a done HInter: its protected members.
pub fn hinter_protected_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (c, s) = (curve(&mut r), surface(&mut r));
    let mut x = HInter::new();
    x.done = true;
    let op = r.integer();
    attempt(|| hinter_protected(&mut r, &mut x, &c, &s, op));
    intersection_print(&x);
    end();
}

/// `surface line op flags(4) bounds nbsu offsurf`: the EstLimForInf* functions.
pub fn estlim_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    let p = r.pnt();
    let l = Line3::new(p, r.dir());
    let op = r.integer();
    let inf = InfFlags {
        u1: r.flag(),
        u2: r.flag(),
        v1: r.flag(),
        v2: r.flag(),
    };
    let mut b = UvLimits {
        u1: r.real(),
        u2: r.real(),
        v1: r.real(),
        v2: r.real(),
    };
    let (nbsu, off) = (r.integer(), r.flag());
    let mut no = false;
    attempt(|| {
        no = match op {
            0 => iu::est_lim_for_inf_extr::<S>(l, &s, off, nbsu, inf, &mut b)?,
            1 => iu::est_lim_for_inf_revl::<S>(l, &s, inf, &mut b)?,
            2 => iu::est_lim_for_inf_offs::<S>(l, &s, nbsu, inf, &mut b)?,
            _ => {
                iu::est_lim_for_inf_surf(&mut b);
                false
            }
        };
        Ok(())
    });
    flag(no);
    [b.u1, b.u2, b.v1, b.v2].into_iter().for_each(real);
    end();
}

/// `line plane basis-plane conic vmin vmax`: ProjectIntersectAndEstLim.
pub fn project_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let p = r.pnt();
    let l = Line3::new(p, r.dir());
    let q = r.pnt();
    let pl = Plane3::new(q, r.dir()).expect("probe plane");
    let q2 = r.pnt();
    let bpl = Plane3::new(q2, r.dir()).expect("probe plane");
    let k = r.integer();
    let (o, d) = (r.pnt(), r.dir());
    let (a, b) = (r.real(), r.real());
    let (mut vmin, mut vmax) = (r.real(), r.real());
    let mut no = false;
    attempt(|| {
        let mut proj = ProjLibPlane::with_plane(&bpl);
        let ax = Axis2::new(o, d)?;
        match k {
            0 => proj.project_line(&Line3::new(o, d))?,
            3 => proj.project_parabola(&Parabola3::new(ax, a)?)?,
            4 => proj.project_hyperbola(&Hyperbola3::new(ax, a, b)?)?,
            _ => proj.project_circle(&Circle3::new(ax, a)?)?,
        }
        no = iu::project_intersect_and_est_lim(l, &pl, &proj, &mut vmin, &mut vmax)?;
        Ok(())
    });
    flag(no);
    real(vmin);
    real(vmax);
    end();
}

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
