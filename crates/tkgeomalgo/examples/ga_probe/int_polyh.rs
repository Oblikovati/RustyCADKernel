//! IntPolyh probe operations over the algorithm, the Rust twin of the first half of
//! tools/oracle/tkgeomalgo_probe_intpolyh.cxx (the records are in `int_polyh_records.rs`).
use super::common::{Reader, attempt, captured, end, flag, int, real, show};
use super::ics_geometry::{params, surface};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeomalgo::int_polyh::{
    IntPolyhArray, IntPolyhArrayOfPointNormal, IntPolyhArrayOfSectionLines,
    IntPolyhArrayOfTangentZones, IntPolyhEdge, IntPolyhIntersection, IntPolyhMaillageAffinage,
    IntPolyhPoint, IntPolyhStartPoint, IntPolyhTools, IntPolyhTriangle, IntPolyhTriangleEdge,
};
use rustycad_tkmath::bnd::BoundingBox3;

pub fn read_point(r: &mut Reader) -> IntPolyhPoint {
    let (x, y, z, u) = (r.real(), r.real(), r.real(), r.real());
    IntPolyhPoint::new(x, y, z, u, r.real())
}
pub fn point(p: &IntPolyhPoint) {
    for v in [p.x(), p.y(), p.z(), p.u(), p.v()] {
        real(v);
    }
    int(p.part_of_common());
    flag(p.degenerated());
}
pub fn sp(s: &IntPolyhStartPoint) {
    for v in [s.x(), s.y(), s.z(), s.u1(), s.v1(), s.u2(), s.v2()] {
        real(v);
    }
    int(s.t1());
    int(s.e1());
    real(s.lambda1());
    int(s.t2());
    int(s.e2());
    real(s.lambda2());
    real(s.get_angle());
    int(s.chain_list());
}
pub fn read_sp(r: &mut Reader) -> IntPolyhStartPoint {
    let xyz = [r.real(), r.real(), r.real()];
    let uv = [r.real(), r.real(), r.real(), r.real()];
    let (t1, e1, l1) = (r.integer(), r.integer(), r.real());
    let (t2, e2, l2) = (r.integer(), r.integer(), r.real());
    IntPolyhStartPoint::new(xyz, uv, (t1, e1, l1), (t2, e2, l2), r.integer())
}
pub fn triangle(t: &IntPolyhTriangle) {
    int(t.first_point());
    int(t.second_point());
    int(t.third_point());
    for k in 0..=4 {
        int(t.get_edge_number(k));
        int(t.get_edge_orientation(k));
    }
    real(t.deflection());
    flag(t.is_intersection_possible());
    flag(t.has_intersection());
    flag(t.is_degenerated());
}
pub fn edge(e: &IntPolyhEdge) {
    int(e.first_point());
    int(e.second_point());
    int(e.first_triangle());
    int(e.second_triangle());
}
fn bbox(b: &BoundingBox3) {
    flag(b.is_void());
    if let Ok(values) = b.get() {
        values.into_iter().for_each(real);
    }
}
/// `ga_ph_items`: the items, and the one just past them inside the allocated storage.
pub fn items<T>(a: &IntPolyhArray<T>, print: impl Fn(&T)) {
    int(a.nb_items());
    int(a.get_n());
    let n = a.nb_items() + i32::from(a.nb_items() < a.get_n());
    (0..n).for_each(|i| print(&a[i]));
}
fn side(m: &IntPolyhMaillageAffinage, s: i32) {
    items(m.get_array_of_points(s), point);
    items(m.get_array_of_edges(s), edge);
    items(m.get_array_of_triangles(s), triangle);
    bbox(&m.get_box(s));
    real(m.get_min_deflection(s));
    real(m.get_max_deflection(s));
}
fn couples(m: &mut IntPolyhMaillageAffinage) {
    let list = m.get_couples();
    int(list.extent());
    for c in list.iter() {
        int(c.first_value());
        int(c.second_value());
        flag(c.is_analyzed());
        real(c.angle());
    }
}
fn lines(l: &IntPolyhArrayOfSectionLines, z: &IntPolyhArrayOfTangentZones) {
    int(l.nb_items());
    for i in 0..l.nb_items() {
        let line = &l[i];
        int(line.nb_start_points());
        (0..line.nb_start_points()).for_each(|k| sp(line.value(k).expect("probe point")));
    }
    int(z.nb_items());
    (0..z.nb_items()).for_each(|i| sp(&z[i]));
}

fn intersection(
    r: &mut Reader,
    mode: i32,
    s1: &Adaptor3dSurfaceHandle,
    s2: &Adaptor3dSurfaceHandle,
) -> Result<IntPolyhIntersection, Failure> {
    match mode {
        0 => IntPolyhIntersection::new(s1, s2),
        1 => {
            let (a, b, c, d) = (r.integer(), r.integer(), r.integer(), r.integer());
            IntPolyhIntersection::with_samples(s1, (a, b), s2, (c, d))
        }
        _ => {
            let (u1, v1, u2, v2) = (params(r), params(r), params(r), params(r));
            IntPolyhIntersection::with_parameters(s1, (&u1, &v1), s2, (&u2, &v2))
        }
    }
}

/// `ga_ph_inter_case`.
pub fn inter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mode = r.integer();
    let (s1, s2) = (surface(&mut r), surface(&mut r));
    match intersection(&mut r, mode, &s1, &s2) {
        Ok(x) => inter_report(&x),
        Err(e) => print!("{} ", e.exception_type()),
    }
    end();
}

fn inter_report(x: &IntPolyhIntersection) {
    flag(x.is_done());
    flag(x.is_parallel());
    let n = x.nb_section_lines();
    int(n);
    for i in 1..=n {
        let np = x.nb_points_in_line(i).expect("probe line");
        int(np);
        for k in 1..=np + 1 {
            show(x.get_line_point(i, k), |a| a.into_iter().for_each(real));
        }
        attempt(|| x.get_line_point(i, 0).map(|_| ()));
        attempt(|| x.get_line_point(i, np + 2).map(|_| ()));
    }
    int(x.nb_points_in_line(n + 1).expect("probe line"));
    let nz = x.nb_tangent_zones();
    int(nz);
    for i in 1..=nz + 1 {
        x.get_tangent_zone_point(i, 1)
            .expect("probe zone")
            .into_iter()
            .for_each(real);
    }
    int(x.nb_points_in_tangent_zone(1));
}

fn opt_tol(r: &mut Reader) -> Option<f64> {
    let has = r.flag();
    let t = r.real();
    has.then_some(t)
}

/// The step `op` of `ga_ph_maillage_case`.
fn step(
    m: &mut IntPolyhMaillageAffinage,
    r: &mut Reader,
    surfaces: (&Adaptor3dSurfaceHandle, &Adaptor3dSurfaceHandle),
    store: (
        &mut IntPolyhArrayOfSectionLines,
        &mut IntPolyhArrayOfTangentZones,
    ),
) {
    match r.integer() {
        0 => {
            let id = r.integer();
            attempt(|| m.fill_array_of_pnt(id));
        }
        1 => {
            let (id, f) = (r.integer(), r.flag());
            attempt(|| m.fill_array_of_pnt_shift(id, f));
        }
        2 => {
            let id = r.integer();
            let (u, v) = (params(r), params(r));
            let t = opt_tol(r);
            attempt(|| m.fill_array_of_pnt_params(id, &u, &v, t));
        }
        3 => {
            let (id, f) = (r.integer(), r.flag());
            let (u, v) = (params(r), params(r));
            let t = opt_tol(r);
            attempt(|| m.fill_array_of_pnt_shift_params(id, f, &u, &v, t));
        }
        4 => attempt(|| m.common_box()),
        5 => m.fill_array_of_triangles(r.integer()),
        6 => m.fill_array_of_edges(r.integer()),
        7 => attempt(|| m.triangles_deflections_refinement_bsb()),
        8 => attempt(|| m.common_part_refinement()),
        9 => {
            let id = r.integer();
            attempt(|| m.local_surface_refinement(id));
        }
        10 => {
            let id = r.integer();
            attempt(|| m.compute_deflections(id));
        }
        11 => attempt(|| m.triangle_compare().map(int)),
        12 => attempt(|| m.start_points_chain(store.0, store.1).map(int)),
        13 => m.set_enlarge_zone(r.flag()),
        14 => {
            let (f1, f2) = (r.flag(), r.flag());
            attempt(|| advanced_fill(m, surfaces, (f1, f2)));
        }
        _ => attempt(|| {
            m.common_box()?;
            m.fill_array_of_triangles(1);
            m.fill_array_of_triangles(2);
            m.fill_array_of_edges(1);
            m.fill_array_of_edges(2);
            m.triangles_deflections_refinement_bsb()?;
            int(m.triangle_compare()?);
            Ok(())
        }),
    }
}

/// The step 14: the advanced points of both surfaces from the normals of the sampling.
fn advanced_fill(
    m: &mut IntPolyhMaillageAffinage,
    (s1, s2): (&Adaptor3dSurfaceHandle, &Adaptor3dSurfaceHandle),
    (f1, f2): (bool, bool),
) -> Result<(), Failure> {
    let (mut u1, mut v1, mut u2, mut v2) = (
        Array1::default(),
        Array1::default(),
        Array1::default(),
        Array1::default(),
    );
    m.make_sampling(1, &mut u1, &mut v1)?;
    m.make_sampling(2, &mut u2, &mut v2)?;
    let (mut p1, mut p2) = (
        IntPolyhArrayOfPointNormal::default(),
        IntPolyhArrayOfPointNormal::default(),
    );
    IntPolyhTools::fill_array_of_point_normal(s1, &u1, &v1, &mut p1)?;
    IntPolyhTools::fill_array_of_point_normal(s2, &u2, &v2, &mut p2)?;
    let t1 = IntPolyhTools::compute_deflection(s1, &u1, &v1)?;
    m.fill_array_of_pnt_normals(1, f1, &p1, (&u1, &v1), t1)?;
    let t2 = IntPolyhTools::compute_deflection(s2, &u2, &v2)?;
    m.fill_array_of_pnt_normals(2, f2, &p2, (&u2, &v2), t2)
}

/// `ga_ph_maillage_case`.
pub fn maillage_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s1 = surface(&mut r);
    let (a, b) = (r.integer(), r.integer());
    let s2 = surface(&mut r);
    let (c, d) = (r.integer(), r.integer());
    let mut m = IntPolyhMaillageAffinage::with_samples(&s1, (a, b), &s2, (c, d), 0);
    let (mut l, mut z) = (
        IntPolyhArrayOfSectionLines::default(),
        IntPolyhArrayOfTangentZones::default(),
    );
    l.init(1000);
    z.init(10000);
    for _ in 0..r.integer() {
        step(&mut m, &mut r, (&s1, &s2), (&mut l, &mut z));
    }
    flag(m.get_enlarge_zone());
    side(&m, 1);
    side(&m, 2);
    couples(&mut m);
    lines(&l, &z);
    let (b1, b2) = (m.get_box(1), m.get_box(2));
    show(m.common_box_of(&b1, &b2), |z| z.into_iter().for_each(real));
    end();
}

/// `ga_ph_tricontact_case`.
pub fn tricontact_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut angle = r.real();
    let p: Vec<IntPolyhPoint> = (0..6).map(|_| read_point(&mut r)).collect();
    let m = IntPolyhMaillageAffinage::new(&Handle::default(), &Handle::default(), 0);
    int(m.tri_contact([&p[0], &p[1], &p[2]], [&p[3], &p[4], &p[5]], &mut angle));
    real(angle);
    end();
}

fn read_edges(r: &mut Reader) -> IntPolyhTriangle {
    let mut t = IntPolyhTriangle::default();
    let e = [r.integer(), r.integer(), r.integer()];
    let o = [r.integer(), r.integer(), r.integer()];
    t.set_first_edge(e[0], o[0]);
    t.set_second_edge(e[1], o[1]);
    t.set_third_edge(e[2], o[2]);
    t
}

/// `ga_ph_edgecontact_case`.
pub fn edgecontact_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (id, ei) = (r.integer(), r.integer());
    let (t1, t2) = (read_edges(&mut r), read_edges(&mut r));
    let p: Vec<IntPolyhPoint> = (0..5).map(|_| read_point(&mut r)).collect();
    let (mut sp1, mut sp2) = (read_sp(&mut r), read_sp(&mut r));
    let (c12, c23, c31, e) = (p[1] - p[0], p[2] - p[1], p[0] - p[2], p[4] - p[3]);
    let mut nn = IntPolyhPoint::default();
    nn.cross(&c12, &c23);
    let n = nn.divide(nn.square_modulus().sqrt());
    let args = IntPolyhTriangleEdge {
        tri_surf_id: id,
        edge_index: ei,
        tris: (&t1, &t2),
        pt: [&p[0], &p[1], &p[2]],
        cote: [&c12, &c23, &c31],
        pe: [&p[3], &p[4]],
        edge: &e,
        normale: &n,
    };
    let m = IntPolyhMaillageAffinage::new(&Handle::default(), &Handle::default(), 0);
    show(m.triangle_edge_contact(&args, &mut sp1, &mut sp2), int);
    sp(&sp1);
    sp(&sp2);
    end();
}

/// `ga_ph_triangle_case`.
pub fn triangle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = surface(&mut r);
    let (nsu, nsv, num, multiple) = (r.integer(), r.integer(), r.integer(), r.integer());
    let crit = r.real();
    let mut bb = BoundingBox3::default();
    let lo = [r.real(), r.real(), r.real()];
    bb.update_bounds(lo, [r.real(), r.real(), r.real()]);
    let mut m = IntPolyhMaillageAffinage::with_samples(&s, (nsu, nsv), &s, (nsu, nsv), 0);
    attempt(|| {
        m.fill_array_of_pnt(1)?;
        m.fill_array_of_triangles(1);
        m.fill_array_of_edges(1);
        triangle_members(&m, &s, (num, multiple != 0, crit, &bb))
    });
    end();
}

fn triangle_members(
    m: &IntPolyhMaillageAffinage,
    s: &Adaptor3dSurfaceHandle,
    (num, multiple, crit, bb): (i32, bool, f64, &BoundingBox3),
) -> Result<(), Failure> {
    let mut pts = m.get_array_of_points(1).clone();
    let mut tris = m.get_array_of_triangles(1).clone();
    let mut edges = m.get_array_of_edges(1).clone();
    for k in 0..tris.nb_items() {
        show(tris[k].compute_deflection(&**s, &pts), real);
        triangle(&tris[k]);
        bbox(tris[k].bounding_box(&pts));
        (0..=4).for_each(|e| int(tris[k].get_next_triangle(k, e, &edges)));
    }
    captured(|out| tris[0].dump(0, out).map_err(|_| Failure::new("fmt")));
    let mut l = tris[0].clone();
    l.link_edges_2_triangle(&edges, 0, 1, 2);
    triangle(&l);
    l.link_edges_2_triangle(&edges, -1, 1, 2);
    triangle(&l);
    free_triangle();
    let mut t = tris[num].clone();
    let refined = if multiple {
        t.multiple_middle_refinement(crit, bb, num, &**s, &mut pts, &mut tris, &mut edges)
    } else {
        t.middle_refinement(num, &**s, &mut pts, &mut tris, &mut edges)
    };
    tris[num] = t;
    if let Err(e) = refined {
        print!("{} ", e.exception_type());
    }
    items(&pts, point);
    items(&tris, triangle);
    items(&edges, edge);
    Ok(())
}

fn free_triangle() {
    let mut t = IntPolyhTriangle::new(0, 1, 2);
    t.set_edge(0, 5);
    t.set_edge(2, 7);
    t.set_edge_orientation(4, 1);
    t.set_edge_orientation(3, -1);
    t.set_deflection(0.5);
    t.set_first_edge(3, 1);
    t.set_second_edge(4, -1);
    t.set_third_edge(6, 1);
    t.set_first_point(9);
    t.set_second_point(8);
    t.set_third_point(7);
    t.set_intersection_possible(false);
    t.set_intersection(true);
    t.set_degenerated(true);
    triangle(&t);
}
