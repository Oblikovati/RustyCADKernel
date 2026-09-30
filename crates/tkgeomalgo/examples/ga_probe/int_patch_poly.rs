//! IntPatch polyhedron and polygon probe operations, the Rust twin of
//! tools/oracle/tkgeomalgo_probe_intpatch_poly.cxx.
use super::common::{Reader, attempt, end, flag, int, pnt, pnt2d, real, xyz};
use super::ics_geometry::surface;
use super::int_patch_lines::read_line2s;
use super::int_patch_tools::arc;
use super::intf::interference_print;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::int_patch::{
    IntPatchBVHTraversal, IntPatchInterferencePolyhedron, IntPatchPolyArc, IntPatchPolyLine,
    IntPatchPolygo, IntPatchPolyhedron, IntPatchPolyhedronBVH, IntPatchPolyhedronTool as T,
    IntPatchPrmPrmIntersectionT3Bits, IntPatchRLine, IntPatchWLine,
};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::bvh::{BvhSet, BvhVec3d};
use rustycad_tkmath::gp::Point3;

/// `ga_ip_polyhedron`.
fn polyhedron(r: &mut Reader) -> Result<IntPatchPolyhedron, Failure> {
    let s = surface(r);
    if r.flag() {
        return IntPatchPolyhedron::new(&s);
    }
    let nbu = r.integer();
    IntPatchPolyhedron::with_subdivisions(&s, nbu, r.integer())
}

fn box3(b: &BoundingBox3) {
    flag(b.is_void());
    attempt(|| b.get().map(|v| v.into_iter().for_each(real)));
}

fn points_report(p: &IntPatchPolyhedron) -> Result<(), Failure> {
    pnt(p.point(0)?);
    for i in 1..=p.nb_points() {
        let (q, u, v) = p.point_with_parameters(i)?;
        pnt(q);
        real(u);
        real(v);
        let (u, v) = p.parameters(i)?;
        real(u);
        real(v);
        pnt(p.point(i)?);
        pnt(T::point(p, i)?);
    }
    Ok(())
}

fn triangles_report(p: &IntPatchPolyhedron) -> Result<(), Failure> {
    let boxes = T::components_bounding(p);
    int(boxes.length());
    for t in 1..=p.nb_triangles() {
        let [a, b, c] = p.triangle(t);
        [a, b, c].into_iter().for_each(int);
        let [x, y, z] = T::triangle(p, t);
        int(x + y + z);
        box3(&boxes[t]);
        let (n, d) = p.plane_equation(t, 12345.0)?;
        xyz(n);
        real(d);
        let g = Point3::from((p.point(a)?.xyz() + p.point(b)?.xyz() + p.point(c)?.xyz()) / 3.0);
        flag(p.contain(t, g)?);
        flag(p.contain(t, Point3::new(1e3, 1e3, 1e3))?);
    }
    Ok(())
}

fn connex_report(p: &IntPatchPolyhedron, nconnex: i32) {
    for t in 0..=nconnex.min(p.nb_triangles()) {
        let [a, b, c] = if t > 0 { p.triangle(t) } else { [1, 2, 3] };
        let v = [a, b, c, 0];
        for i in 0..3 {
            for (j, &pedge) in v.iter().enumerate() {
                if i == j {
                    continue;
                }
                attempt(|| {
                    let (res, tc, op) = p.tri_connex(t, v[i], pedge)?;
                    [res, tc, op].into_iter().for_each(int);
                    int(T::tri_connex(p, t, v[i], pedge)?.0);
                    Ok(())
                });
            }
        }
    }
}

/// `ga_ip_polyh_report`.
fn polyh_report(p: &IntPatchPolyhedron, nconnex: i32) -> Result<(), Failure> {
    let (nu, nv) = p.size();
    [nu, nv, p.nb_triangles(), p.nb_points()]
        .into_iter()
        .for_each(int);
    real(p.deflection_over_estimation());
    real(T::deflection_over_estimation(p));
    box3(p.bounding());
    box3(T::bounding(p));
    points_report(p)?;
    triangles_report(p)?;
    connex_report(p, nconnex);
    Ok(())
}

/// `ga_ip_polyh_case`.
pub fn polyh_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let mut p = polyhedron(&mut r)?;
        let (nconnex, defl) = (r.integer(), r.real());
        polyh_report(&p, nconnex)?;
        p.set_point(Point3::new(1.0, 2.0, 3.0), 1, 1, 0.5, 0.5);
        pnt(p.point(1)?);
        p.set_deflection_over_estimation(defl);
        real(p.deflection_over_estimation());
        box3(p.bounding());
        p.fill_bounding()?;
        for t in 1..=p.nb_triangles() {
            box3(&p.components_bounding()[t]);
        }
        Ok(())
    });
    end();
}

/// `ga_ip_bvh_report`.
fn bvh_report(s: &IntPatchPolyhedronBVH<'_>) -> Result<(), Failure> {
    flag(s.is_initialized());
    int(s.size());
    for i in -1..=s.size() {
        int(s.original_index(i));
        let b = s.element_box(i)?;
        flag(b.is_valid());
        if b.is_valid() {
            let (lo, hi) = (b.corner_min(), b.corner_max());
            [lo.x(), lo.y(), lo.z(), hi.x(), hi.y(), hi.z()]
                .into_iter()
                .for_each(real);
        }
        for a in 0..=3 {
            real(s.center(i, a)?);
        }
    }
    Ok(())
}

fn pairs(t: &IntPatchBVHTraversal) {
    int(t.pairs().len() as i32);
    for p in t.pairs() {
        int(p.first);
        int(p.second);
    }
}

/// `ga_ip_bvh_case`.
pub fn bvh_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let (p1, p2) = (polyhedron(&mut r)?, polyhedron(&mut r)?);
        let mut s1 = IntPatchPolyhedronBVH::with_polyhedron(&p1)?;
        let mut s2 = IntPatchPolyhedronBVH::with_polyhedron(&p2)?;
        let mut e = IntPatchPolyhedronBVH::new();
        bvh_report(&s1)?;
        bvh_report(&e)?;
        let mut t = IntPatchBVHTraversal::new();
        int(t.perform(&mut s1, &mut s2, false)?);
        pairs(&t);
        bvh_report(&s1)?;
        bvh_report(&s2)?;
        int(t.perform_self(&mut s1, true)?);
        pairs(&t);
        int(t.perform_self(&mut s1, false)?);
        pairs(&t);
        int(t.perform(&mut s1, &mut e, false)?);
        pairs(&t);
        t.clear();
        pairs(&t);
        s2.clear();
        bvh_report(&s2)?;
        int(t.perform(&mut s1, &mut s2, false)?);
        s2.init(&p1)?;
        int(t.perform(&mut s1, &mut s2, true)?);
        pairs(&t);
        let v = |x: f64| BvhVec3d::new(x, x, x);
        let reject = IntPatchBVHTraversal::reject_node;
        flag(reject(&v(0.0), &v(1.0), &v(1.0), &v(2.0)));
        flag(reject(&v(0.0), &v(1.0), &v(1.5), &v(2.0)));
        flag(reject(&v(1.0), &v(2.0), &v(0.0), &v(1.0)));
        Ok(())
    });
    end();
}

/// `ga_ip_interf_case`.
pub fn interf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    attempt(|| {
        let p1 = polyhedron(&mut r)?;
        if mode <= 1 {
            let p2 = polyhedron(&mut r)?;
            if mode == 0 {
                let i = IntPatchInterferencePolyhedron::between(&p1, &p2)?;
                interference_print(&i);
            } else {
                let mut i = IntPatchInterferencePolyhedron::new();
                interference_print(&i);
                i.perform_self(&p1);
                interference_print(&i);
                i.perform(&p1, &p2)?;
                interference_print(&i);
            }
        } else if mode == 2 {
            let i = IntPatchInterferencePolyhedron::of_self(&p1)?;
            interference_print(&i);
        } else {
            let mut i = IntPatchInterferencePolyhedron::new();
            i.perform_self(&p1);
            interference_print(&i);
        }
        Ok(())
    });
    end();
}

fn box2(b: &BoundingBox2) {
    flag(b.is_void());
    attempt(|| b.get().map(|v| v.into_iter().for_each(real)));
    real(b.gap());
}

/// `ga_ip_polygo`.
fn polygo<P: IntPatchPolygo>(p: &P) -> Result<(), Failure> {
    let n = p.nb_points()?;
    int(n);
    for i in 1..=n {
        pnt2d(p.point(i)?);
    }
    real(p.error());
    real(p.deflection_over_estimation());
    int(p.nb_segments());
    for i in 1..=p.nb_segments() {
        let (a, b) = p.segment(i)?;
        pnt2d(a);
        pnt2d(b);
    }
    flag(p.closed());
    box2(p.bounding());
    Ok(())
}

/// `ga_ip_polyline_case`.
pub fn polyline_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (kind, onfirst, defle) = (r.integer(), r.flag(), r.real());
    let c = read_line2s(&mut r);
    let mut p = if defle < 0.0 {
        IntPatchPolyLine::new()
    } else {
        IntPatchPolyLine::with_deflection(defle)
    };
    real(p.error());
    attempt(|| {
        if kind == 0 {
            p.set_w_line(
                onfirst,
                &Handle::new(IntPatchWLine::new_undecided(&c, false)),
            )?;
        } else {
            let l = IntPatchRLine::new_undecided(false);
            l.add(&c);
            p.set_r_line(onfirst, &Handle::new(l))?;
        }
        polygo(&p)?;
        attempt(|| p.point(0).map(drop));
        attempt(|| p.point(p.nb_points()? + 1).map(drop));
        p.reset_error();
        real(p.error());
        Ok(())
    });
    end();
}

/// `ga_ip_polyarc_case`.
pub fn polyarc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = arc(&mut r)?;
        let (nb, pf, pl) = (r.integer(), r.real(), r.real());
        let mut b = BoundingBox2::default();
        if !r.flag() {
            let (x0, y0, x1, y1) = (r.real(), r.real(), r.real(), r.real());
            b.update_bounds([x0, y0], [x1, y1]);
        }
        let (ox, oy) = (r.real(), r.real());
        let mut p = IntPatchPolyArc::new(&a, nb, pf, pl, &b)?;
        polygo(&p)?;
        for i in 1..=p.nb_points()? {
            real(p.parameter(i)?);
        }
        attempt(|| p.parameter(0).map(drop));
        attempt(|| p.point(p.nb_points()? + 1).map(drop));
        p.set_offset(ox, oy)?;
        polygo(&p)?;
        p.set_offset(0.0, 0.0)?;
        polygo(&p)
    });
    end();
}

/// `ga_ip_t3bits_case`.
pub fn t3bits_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let size = r.integer();
    let (mut a, mut b) = (
        IntPatchPrmPrmIntersectionT3Bits::new(size),
        IntPatchPrmPrmIntersectionT3Bits::new(size),
    );
    for _ in 0..r.integer() {
        let (c, t) = (r.integer(), r.integer());
        match c {
            1 => a.add(t),
            2 => b.add(t),
            3 => a.raz(t),
            4 => int(a.val(t)),
            5 => {
                let mut k = t;
                int(a.and(&mut b, &mut k));
                int(k);
            }
            _ => a.reset_and(),
        }
    }
    end();
}
