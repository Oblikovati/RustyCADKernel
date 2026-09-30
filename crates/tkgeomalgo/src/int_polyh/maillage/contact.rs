// Rust translation of maxSR, minSR, project6, TriContact and TriangleCompare of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contact test of two triangles (the separating axis test on the normals, the edge
//! cross products and the outward side normals) and the couples of triangles in contact.
use super::interfering::get_interfering_triangles;
use super::{IntPolyhMaillageAffinage, SQUARE_MY_CONFUSION_PRECISION};
use crate::int_polyh::edge::IntPolyhCouple;
use crate::int_polyh::point::IntPolyhPoint;
use rustycad_tkernel::standard::Failure;

/// `maxSR(a, b, c)`.
fn max_sr(a: f64, b: f64, c: f64) -> f64 {
    let mut t: f64 = a;
    if b > t {
        t = b;
    }
    if c > t {
        t = c;
    }
    t
}

/// `minSR(a, b, c)`.
fn min_sr(a: f64, b: f64, c: f64) -> f64 {
    let mut t: f64 = a;
    if b < t {
        t = b;
    }
    if c < t {
        t = c;
    }
    t
}

/// `project6(ax, p1, p2, p3, q1, q2, q3)`: 1 when the projections of the two triangles
/// on the axis overlap, 0 when the axis separates them.
fn project6(ax: &IntPolyhPoint, p: &[IntPolyhPoint; 3], q: &[IntPolyhPoint; 3]) -> bool {
    let (p1, p2, p3) = (ax.dot(&p[0]), ax.dot(&p[1]), ax.dot(&p[2]));
    let (q1, q2, q3) = (ax.dot(&q[0]), ax.dot(&q[1]), ax.dot(&q[2]));
    let (mx1, mn1) = (max_sr(p1, p2, p3), min_sr(p1, p2, p3));
    let (mx2, mn2) = (max_sr(q1, q2, q3), min_sr(q1, q2, q3));
    !(mn1 > mx2 || mn2 > mx1)
}

/// The axis-aligned rejection of `TriContact`.
fn boxes_apart(p: &[IntPolyhPoint; 3], q: &[IntPolyhPoint; 3]) -> bool {
    let coords = |t: &[IntPolyhPoint; 3], axis: usize| -> [f64; 3] {
        let c = |pt: &IntPolyhPoint| [pt.x(), pt.y(), pt.z()][axis];
        [c(&t[0]), c(&t[1]), c(&t[2])]
    };
    let max_of = |v: [f64; 3]| max_sr(v[0], v[1], v[2]);
    let min_of = |v: [f64; 3]| min_sr(v[0], v[1], v[2]);
    (0..3).any(|a| max_of(coords(p, a)) < min_of(coords(q, a)))
        || (0..3).any(|a| min_of(coords(p, a)) > max_of(coords(q, a)))
}

/// A point with only the 3D coordinates `a - o` (`SetX(A.X() - O.X())`...).
fn relative(a: &IntPolyhPoint, o: &IntPolyhPoint) -> IntPolyhPoint {
    let mut r = IntPolyhPoint::default();
    r.set_x(a.x() - o.x());
    r.set_y(a.y() - o.y());
    r.set_z(a.z() - o.z());
    r
}

/// `a.Cross(b, c)` on a fresh point.
fn crossed(b: &IntPolyhPoint, c: &IntPolyhPoint) -> IntPolyhPoint {
    let mut r = IntPolyhPoint::default();
    r.cross(b, c);
    r
}

impl IntPolyhMaillageAffinage {
    /// `TriContact(P1, P2, P3, Q1, Q2, Q3, Angle)`: 1 when the triangles are in contact,
    /// 0 otherwise, and then the cosine of the angle between their normals, written only
    /// when both normals are not null (`angle` keeps its value otherwise).
    /// `m.tri_contact([&p1, &p2, &p3], [&q1, &q2, &q3], &mut angle)`.
    pub fn tri_contact(
        &self,
        big_p: [&IntPolyhPoint; 3],
        big_q: [&IntPolyhPoint; 3],
        angle: &mut f64,
    ) -> i32 {
        contact_of(big_p, big_q, angle)
    }

    /// `TriangleCompare()`: the couples of triangles in contact among those whose boxes
    /// interfere, with the cosine of their angle (the value of the previous contact when
    /// a normal is null), both triangles marked; returns the number of couples.
    /// `m.triangle_compare()?`.
    pub fn triangle_compare(&mut self) -> Result<i32, Failure> {
        let [side1, side2] = &mut self.sides;
        let couples = get_interfering_triangles(
            (&mut side1.triangles, &side1.points),
            (&mut side2.triangles, &side2.points),
        )?;
        if couples.is_empty() {
            return Ok(0);
        }
        let mut couple_angle: f64 = -2.0;
        for (i_s1, list) in &couples {
            let p: [IntPolyhPoint; 3] = side1.triangles[*i_s1].corners(&side1.points);
            for &i_s2 in list {
                let q: [IntPolyhPoint; 3] = side2.triangles[i_s2].corners(&side2.points);
                let contact: i32 = contact_of(
                    [&p[0], &p[1], &p[2]],
                    [&q[0], &q[1], &q[2]],
                    &mut couple_angle,
                );
                if contact != 0 {
                    self.t_triangles_contacts.append(IntPolyhCouple::new(
                        *i_s1,
                        i_s2,
                        couple_angle,
                    ));
                    side1.triangles[*i_s1].set_intersection(true);
                    side2.triangles[i_s2].set_intersection(true);
                }
            }
        }
        Ok(self.t_triangles_contacts.extent())
    }
}

/// The body of `TriContact`, which reads nothing of the algorithm.
fn contact_of(big_p: [&IntPolyhPoint; 3], big_q: [&IntPolyhPoint; 3], angle: &mut f64) -> i32 {
    let (bp, bq) = (
        [*big_p[0], *big_p[1], *big_p[2]],
        [*big_q[0], *big_q[1], *big_q[2]],
    );
    if boxes_apart(&bp, &bq) {
        return 0;
    }
    let o: IntPolyhPoint = bp[0];
    let p = [
        relative(&bp[0], &o),
        relative(&bp[1], &o),
        relative(&bp[2], &o),
    ];
    let q = [
        relative(&bq[0], &o),
        relative(&bq[1], &o),
        relative(&bq[2], &o),
    ];
    let e = [
        relative(&p[1], &p[0]),
        relative(&p[2], &p[1]),
        relative(&p[0], &p[2]),
    ];
    let f = [
        relative(&q[1], &q[0]),
        relative(&q[2], &q[1]),
        relative(&q[0], &q[2]),
    ];
    // The normals n1 and m1, then the outward normals g and h of the sides.
    let n1: IntPolyhPoint = crossed(&e[0], &e[1]);
    let m1: IntPolyhPoint = crossed(&f[0], &f[1]);
    let g = [
        crossed(&e[0], &n1),
        crossed(&e[1], &n1),
        crossed(&e[2], &n1),
    ];
    let h = [
        crossed(&f[0], &m1),
        crossed(&f[1], &m1),
        crossed(&f[2], &m1),
    ];
    let mut axes: Vec<IntPolyhPoint> = vec![n1, m1];
    for ei in &e {
        axes.extend(f.iter().map(|fj| crossed(ei, fj)));
    }
    axes.extend(g);
    axes.extend(h);
    if axes.iter().any(|ax| !project6(ax, &p, &q)) {
        return 0;
    }
    let mut sq_mod_m1: f64 = -1.0;
    let sq_mod_n1: f64 = n1.square_modulus();
    if sq_mod_n1 > SQUARE_MY_CONFUSION_PRECISION {
        sq_mod_m1 = m1.square_modulus();
    }
    if sq_mod_m1 > SQUARE_MY_CONFUSION_PRECISION {
        *angle = n1.dot(&m1) / (sq_mod_n1.sqrt() * sq_mod_m1.sqrt());
    }
    1
}
