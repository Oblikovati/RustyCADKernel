//! IntCurveSurface probe operations over the records, the curve tool, the polygon, the
//! polyhedron and the header utilities; the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intcurvesurface.cxx.
use super::common::{Reader, attempt, captured, dir, end, flag, int, pnt, real, text, xyz};
use super::ics_geometry::{curve, params, surface};
use super::intf_tool::box3d_print;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_curve_surface::polygon_utils::approx_param_on_curve;
use rustycad_tkgeomalgo::int_curve_surface::polyhedron_utils as pu;
use rustycad_tkgeomalgo::int_curve_surface::{
    HCurve, Intersection, IntersectionPoint, IntersectionSegment, TheHCurveTool,
    ThePolygonOfHInter, ThePolygonToolOfHInter, ThePolyhedronOfHInter, ThePolyhedronToolOfHInter,
    TransitionOnCurve, tri_connex,
};
use rustycad_tkgeomalgo::intf::Polygon3dTool;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2, Line3, Point3, Xyz};
use std::fmt;

fn transition(code: i32) -> TransitionOnCurve {
    match code {
        1 => TransitionOnCurve::In,
        2 => TransitionOnCurve::Out,
        _ => TransitionOnCurve::Tangent,
    }
}

/// `ga_ics_ipoint`.
pub fn ipoint(p: &IntersectionPoint) {
    pnt(p.pnt());
    real(p.u());
    real(p.v());
    real(p.w());
    int(p.transition() as i32);
}

/// `ga_ics_read_ipoint`.
pub fn read_ipoint(r: &mut Reader) -> IntersectionPoint {
    let p = r.pnt();
    let (u, v, w) = (r.real(), r.real(), r.real());
    IntersectionPoint::new(p, u, v, w, transition(r.integer()))
}

fn text_failure(e: fmt::Error) -> Failure {
    Failure::new(&format!("probe text: {e}"))
}

/// `p u v w tr p2 u2 v2 w2 tr2`: the point records, the segment records and their dumps.
pub fn records_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (a, b) = (read_ipoint(&mut r), read_ipoint(&mut r));
    let mut d = IntersectionPoint::default();
    ipoint(&d);
    ipoint(&a);
    let (p, u, v, w, tr) = a.values();
    pnt(p);
    real(u);
    real(v);
    real(w);
    int(tr as i32);
    d.set_values(b.pnt(), b.u(), b.v(), b.w(), b.transition());
    ipoint(&d);
    captured(|out| a.dump(out).map_err(text_failure));
    let (mut s0, s) = (
        IntersectionSegment::default(),
        IntersectionSegment::new(a, b),
    );
    ipoint(&s0.first_point());
    ipoint(&s0.second_point());
    let (f, l) = s.values();
    ipoint(&f);
    ipoint(&l);
    ipoint(&s.first_point());
    ipoint(&s.second_point());
    s0.set_values(b, a);
    ipoint(&s0.first_point());
    ipoint(&s0.second_point());
    captured(|out| s.dump(out).map_err(text_failure));
    end();
}

/// `ga_ics_intersection_print`.
pub fn intersection_print(x: &Intersection) {
    flag(x.is_done());
    flag(x.is_parallel());
    attempt(|| {
        let n = x.nb_points()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| x.point(i).map(|p| ipoint(&p)));
        }
        Ok(())
    });
    attempt(|| {
        let n = x.nb_segments()?;
        int(n);
        for i in 0..=n + 1 {
            attempt(|| {
                x.segment(i).map(|s| {
                    ipoint(&s.first_point());
                    ipoint(&s.second_point());
                })
            });
        }
        Ok(())
    });
    captured(|out| x.dump(out).map_err(text_failure));
}

/// `count (target command args)...`: commands on two intersections (see the C++ twin).
pub fn intersection_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut x = [Intersection::new(), Intersection::new()];
    for _ in 0..r.integer() {
        let (t, c) = (r.integer() as usize, r.integer());
        let other = x[1 - t].clone();
        let me = &mut x[t];
        match c {
            0 => me.append_point(read_ipoint(&mut r)),
            1 => {
                let a = read_ipoint(&mut r);
                me.append_segment(IntersectionSegment::new(a, read_ipoint(&mut r)));
            }
            2 => (me.done, me.my_is_parallel) = (r.flag(), r.flag()),
            3 => me.reset_fields(),
            4 => me.set_values(&other),
            5 => {
                let a = r.real();
                me.append(&other, a, r.real());
            }
            _ => intersection_print(me),
        }
    }
    end();
}

fn lin(l: Line3) {
    pnt(l.location());
    dir(l.direction());
}

fn ax2(a: Axis2) {
    pnt(a.location());
    dir(a.direction());
    dir(a.x_direction());
}

fn hcurve_evaluations(c: &HCurve, u: f64) {
    type T = TheHCurveTool;
    attempt(|| T::value(c, u).map(pnt));
    attempt(|| T::d0(c, u).map(pnt));
    attempt(|| T::d1(c, u).map(|(p, v)| (pnt(p), xyz(*v.xyz())).0));
    attempt(|| T::d2(c, u).map(|(p, v1, v2)| (pnt(p), xyz(*v1.xyz()), xyz(*v2.xyz())).0));
    attempt(|| {
        T::d3(c, u).map(|d| {
            pnt(d.0);
            xyz(*d.1.xyz());
            xyz(*d.2.xyz());
            xyz(*d.3.xyz());
        })
    });
    for n in 1..=3 {
        attempt(|| T::dn(c, u, n).map(|v| xyz(*v.xyz())));
    }
}

fn hcurve_conics(c: &HCurve) {
    type T = TheHCurveTool;
    attempt(|| T::line(c).map(lin));
    attempt(|| T::circle(c).map(|x| (ax2(x.position()), real(x.radius())).0));
    attempt(|| {
        T::ellipse(c).map(|x| {
            (
                ax2(x.position()),
                real(x.major_radius()),
                real(x.minor_radius()),
            )
                .0
        })
    });
    attempt(|| {
        T::hyperbola(c).map(|x| {
            (
                ax2(x.position()),
                real(x.major_radius()),
                real(x.minor_radius()),
            )
                .0
        })
    });
    attempt(|| T::parabola(c).map(|x| (ax2(x.position()), real(x.focal())).0));
    attempt(|| T::bezier(c).map(|b| int(b.nb_poles())));
    attempt(|| T::bspline(c).map(|b| (int(b.nb_poles()), int(b.nb_knots())).0));
}

/// `curve u u0 u1 defl nbmin`: every member of IntCurveSurface_TheHCurveTool.
pub fn hcurve_case(values: &[f64]) {
    type T = TheHCurveTool;
    let mut r = Reader::new(values);
    let c = curve(&mut r);
    let (u, u0, u1, defl) = (r.real(), r.real(), r.real(), r.real());
    let nbmin = r.integer();
    attempt(|| T::first_parameter(&c).map(real));
    attempt(|| T::last_parameter(&c).map(real));
    attempt(|| T::continuity(&c).map(|s| int(s as i32)));
    for s in [Shape::C0, Shape::C1, Shape::C2, Shape::C3, Shape::CN] {
        attempt(|| {
            let n = T::nb_intervals(&c, s)?;
            int(n);
            let mut t = rustycad_tkernel::ncollection::Array1::with_bounds(1, n + 1)
                .map_err(|e| e.into_failure("NCollection_Array1"))?;
            T::intervals(&c, &mut t, s)?;
            t.iter().for_each(|&x| real(x));
            Ok(())
        });
    }
    attempt(|| T::is_closed(&c).map(flag));
    attempt(|| T::is_periodic(&c).map(flag));
    attempt(|| T::period(&c).map(real));
    hcurve_evaluations(&c, u);
    attempt(|| T::resolution(&c, 1e-7).map(real));
    attempt(|| T::get_type(&c).map(|t| int(t as i32)));
    hcurve_conics(&c);
    attempt(|| T::nb_samples(&c, u0, u1).map(int));
    attempt(|| {
        let pars = T::sample_pars(&c, u0, u1, defl, nbmin)?;
        int(pars.length());
        pars.iter().for_each(|&x| real(x));
        Ok(())
    });
    end();
}

/// `ga_ics_polygon_dump`: the tool Dump with the call number of each line masked.
fn polygon_dump(p: &ThePolygonOfHInter) -> Result<String, Failure> {
    let mut console = String::new();
    ThePolygonToolOfHInter::dump(p, &mut console)?;
    let mut out = String::new();
    for line in console.lines() {
        let mut rebuilt = String::new();
        for (k, word) in line.split_whitespace().enumerate() {
            let word = if k == 1 && (rebuilt == "pol2d" || rebuilt == "pnt") {
                "#"
            } else {
                word
            };
            if k > 0 {
                rebuilt.push(' ');
            }
            rebuilt.push_str(word);
        }
        out.push_str(&rebuilt);
        out.push('\n');
    }
    Ok(out)
}

/// `ga_ics_polygon`: a polygon `mode ...`.
pub fn polygon(r: &mut Reader, c: &HCurve) -> Result<ThePolygonOfHInter, Failure> {
    match r.integer() {
        0 => ThePolygonOfHInter::new(c, r.integer()),
        1 => {
            let (a, b) = (r.real(), r.real());
            ThePolygonOfHInter::with_range(c, a, b, r.integer())
        }
        _ => ThePolygonOfHInter::with_parameters(c, &params(r)),
    }
}

fn polygon_members(r: &mut Reader, p: &mut ThePolygonOfHInter) -> Result<(), Failure> {
    type T = ThePolygonToolOfHInter;
    box3d_print(p.bounding());
    real(p.deflection_over_estimation());
    flag(p.closed());
    let n = p.nb_segments();
    int(n);
    real(p.inf_parameter());
    real(p.sup_parameter());
    for i in -1..=n + 2 {
        attempt(|| p.begin_of_seg(i).map(pnt));
        attempt(|| p.end_of_seg(i).map(pnt));
    }
    for _ in 0..r.integer() {
        let (i, t) = (r.integer(), r.real());
        attempt(|| p.approx_param_on_curve(i, t).map(real));
    }
    box3d_print(T::bounding(p));
    real(T::deflection_over_estimation(p));
    flag(T::closed(p));
    int(T::nb_segments(p));
    for i in 1..=n {
        pnt(T::begin_of_seg(p, i)?);
        pnt(T::end_of_seg(p, i)?);
    }
    text(polygon_dump(p)?.as_bytes());
    p.dump();
    p.set_closed(true);
    flag(p.closed());
    flag(T::closed(p));
    p.set_closed(false);
    flag(p.closed());
    p.set_deflection_over_estimation(r.real());
    real(p.deflection_over_estimation());
    box3d_print(p.bounding());
    Ok(())
}

/// `curve polygon m (index param)... deflection`: every polygon and tool member.
pub fn polygon_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let c = curve(&mut r);
    attempt(|| {
        let mut p = polygon(&mut r, &c)?;
        polygon_members(&mut r, &mut p)
    });
    end();
}

/// `ga_ics_polyhedron`: a polyhedron `mode ...`.
pub fn polyhedron(
    r: &mut Reader,
    s: &Adaptor3dSurfaceHandle,
) -> Result<ThePolyhedronOfHInter, Failure> {
    if r.integer() == 0 {
        let (nu, nv) = (r.integer(), r.integer());
        let (u1, v1, u2, v2) = (r.real(), r.real(), r.real(), r.real());
        return ThePolyhedronOfHInter::new(s, nu, nv, u1, v1, u2, v2);
    }
    let (up, vp) = (params(r), params(r));
    ThePolyhedronOfHInter::with_parameters(s, &up, &vp)
}

fn polyhedron_points(p: &ThePolyhedronOfHInter) -> Result<(), Failure> {
    let (_, nv) = p.size();
    let n = p.nb_points();
    for i in 1..=n {
        let (q, u, v) = p.point_with_parameters(i)?;
        pnt(q);
        real(u);
        real(v);
        pnt(p.point(i)?);
        pnt(p.point(i)?);
        let (u, v) = p.parameters(i)?;
        real(u);
        real(v);
        for j in [i + 1, i - 1, i + nv + 1, i - nv - 1, i + 2] {
            if (1..=n).contains(&j) {
                flag(p.is_on_bound(i, j));
            }
        }
    }
    Ok(())
}

fn polyhedron_triangles(p: &ThePolyhedronOfHInter, probe: Point3) -> Result<(), Failure> {
    let nt = p.nb_triangles();
    for t in 1..=nt {
        let [a, b, c] = p.triangle(t);
        [a, b, c].into_iter().for_each(int);
        box3d_print(&p.components_bounding()[t]);
        let (nrm, d) = p.plane_equation(t)?;
        xyz(nrm);
        real(d);
        let g = Point3::from((p.point(a)?.xyz() + p.point(b)?.xyz() + p.point(c)?.xyz()) / 3.0);
        flag(p.contain(t, g)?);
        flag(p.contain(t, probe)?);
    }
    for t in 0..=nt {
        let [a, b, c] = p.triangle(t.max(1));
        for pivot in [a, b, c] {
            for pedge in [0, a, b, c] {
                if pedge != pivot {
                    let (ret, tc, o) = p.tri_connex(t, pivot, pedge);
                    [ret, tc, o].into_iter().for_each(int);
                    let (ret, tc, o) = ThePolyhedronToolOfHInter::tri_connex(p, t, pivot, pedge);
                    [ret, tc, o].into_iter().for_each(int);
                }
            }
        }
    }
    Ok(())
}

fn polyhedron_summary(p: &ThePolyhedronOfHInter) -> Result<(), Failure> {
    type T = ThePolyhedronToolOfHInter;
    let (nu, nv) = p.size();
    [nu, nv, p.nb_triangles(), p.nb_points()]
        .into_iter()
        .for_each(int);
    real(p.deflection_over_estimation());
    real(p.get_border_deflection());
    box3d_print(p.bounding());
    int(T::nb_triangles(p));
    real(T::deflection_over_estimation(p));
    real(T::get_border_deflection(p));
    box3d_print(T::bounding(p));
    int(p.components_bounding().length());
    int(T::components_bounding(p).length());
    pnt(T::point(p, 1)?);
    T::triangle(p, 1).into_iter().for_each(int);
    flag(T::is_on_bound(p, 1, 2));
    Ok(())
}

fn polyhedron_members(r: &mut Reader, p: &mut ThePolyhedronOfHInter) -> Result<(), Failure> {
    let detail = r.flag();
    let probe = r.pnt();
    polyhedron_summary(p)?;
    if detail {
        polyhedron_points(p)?;
        polyhedron_triangles(p, probe)?;
    }
    let f = [r.flag(), r.flag(), r.flag(), r.flag()];
    p.u_min_singularity(f[0]);
    p.u_max_singularity(f[1]);
    p.v_min_singularity(f[2]);
    p.v_max_singularity(f[3]);
    let has = [
        p.has_u_min_singularity(),
        p.has_u_max_singularity(),
        p.has_v_min_singularity(),
        p.has_v_max_singularity(),
    ];
    has.into_iter().for_each(flag);
    p.set_point(Point3::new(9.0, 9.0, 9.0), 1, 1, 9.0, 9.0);
    pnt(p.point(1)?);
    p.dump();
    ThePolyhedronToolOfHInter::dump(p);
    p.set_deflection_over_estimation(r.real());
    real(p.deflection_over_estimation());
    box3d_print(p.bounding());
    p.fill_bounding()?;
    box3d_print(&p.components_bounding()[1]);
    box3d_print(&p.components_bounding()[p.nb_triangles()]);
    p.destroy();
    Ok(())
}

/// `surface polyhedron detail probe flags(4) flec`: every polyhedron and tool member.
pub fn polyhedron_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    attempt(|| {
        let mut p = polyhedron(&mut r, &s)?;
        polyhedron_members(&mut r, &mut p)
    });
    end();
}

fn utils_triangle_ops(r: &mut Reader, op: i32) {
    let (p1, p2, p3) = (r.pnt(), r.pnt(), r.pnt());
    match op {
        0 => {
            let (n, d): (Xyz, f64) = pu::plane_equation(p1, p2, p3);
            xyz(n);
            real(d);
        }
        1 => flag(pu::contain(p1, p2, p3, r.pnt())),
        _ => real(pu::compute_deflection_with_center(p1, p2, p3, r.pnt())),
    }
}

/// `op args`: the PolyhedronUtils and PolygonUtils functions on data (see the C++ twin).
pub fn utils_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let op = r.integer();
    match op {
        0..=2 => utils_triangle_ops(&mut r, op),
        3 => {
            let (i, nv) = (r.integer(), r.integer());
            pu::triangle(i, nv).into_iter().for_each(int);
        }
        4 => {
            let (t, pv, pe, nu, nv) = (
                r.integer(),
                r.integer(),
                r.integer(),
                r.integer(),
                r.integer(),
            );
            let (ret, tc, o) = tri_connex(t, pv, pe, (nu, nv));
            [ret, tc, o].into_iter().for_each(int);
        }
        5 => {
            let (nu, nv) = (r.integer(), r.integer());
            int(pu::nb_triangles(nu, nv));
            int(pu::nb_points(nu, nv));
        }
        6 => {
            let (mut d, mut b) = (-1.0, BoundingBox3::default());
            b.add_point(Point3::new(0.0, 0.0, 0.0));
            pu::set_deflection_over_estimation(r.real(), &mut d, &mut b);
            real(d);
            box3d_print(&b);
        }
        _ => utils_approx(&mut r),
    }
    end();
}

fn utils_approx(r: &mut Reader) {
    let (i, t, bi, bs) = (r.integer(), r.real(), r.real(), r.real());
    let n = r.integer();
    let h = params(r);
    let stored = if h.length() > 0 { Some(&h) } else { None };
    attempt(|| approx_param_on_curve(i, t, bi, bs, n, stored).map(real));
}
