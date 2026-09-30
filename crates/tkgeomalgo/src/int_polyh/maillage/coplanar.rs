// Rust translation of CalculPtsInterTriEdgeCoplanaires of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection points of an edge and a side of a coplanar triangle: one crossing
//! point, or the overlap of two parallel segments.
use super::edge_contact::IntPolyhTriangleEdge;
use super::{MY_CONFUSION_PRECISION as MCP, SQUARE_MY_CONFUSION_PRECISION};
use crate::int_polyh::point::IntPolyhPoint;
use crate::int_polyh::start_point::IntPolyhStartPoint;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;

/// The side of the triangle: its ends `PT1`, `PT2`, its vector `Cote` and its rank
/// `CoteIndex` 1..=3.
pub(crate) struct Coplanar<'a> {
    pub(crate) pt: (&'a IntPolyhPoint, &'a IntPolyhPoint),
    pub(crate) cote: &'a IntPolyhPoint,
    pub(crate) cote_index: i32,
}

/// `CalculPtsInterTriEdgeCoplanaires(TriSurfID, NormaleTri, Tri1, Tri2, PE1, PE2, Edge,
/// EdgeIndex, PT1, PT2, Cote, CoteIndex, SP1, SP2, NbPoints)`: gp_VectorWithNullMagnitude
/// when a scaled vector is null, as the native `IsParallel`.
pub(crate) fn calcul_pts_inter_tri_edge_coplanaires(
    a: &IntPolyhTriangleEdge<'_>,
    c: &Coplanar<'_>,
    sp: (&mut IntPolyhStartPoint, &mut IntPolyhStartPoint),
    nb_points: &mut i32,
) -> Result<(), Failure> {
    let (sp1, sp2) = sp;
    let mut a_ve = Vector3::new(a.edge.x(), a.edge.y(), a.edge.z());
    let mut a_vc = Vector3::new(c.cote.x(), c.cote.y(), c.cote.z());
    let a_de: f64 = a_ve.square_magnitude();
    let a_dc: f64 = a_vc.square_magnitude();
    // The native code divides by the squared magnitudes.
    if a_de > SQUARE_MY_CONFUSION_PRECISION {
        a_ve /= a_de;
    }
    if a_dc > SQUARE_MY_CONFUSION_PRECISION {
        a_vc /= a_dc;
    }
    if !a_ve.is_parallel(a_vc, MCP)? {
        crossing(a, c, (sp1, sp2), nb_points);
    } else {
        overlap(a, c, (sp1, sp2), nb_points);
    }
    filter_vertices(*nb_points, sp1, sp2);
    Ok(())
}

/// The edge and the side are not parallel: the crossing point `PIE` of the edge with
/// the side's line, when the side runs past it (`alpha >= 0`).
fn crossing(
    a: &IntPolyhTriangleEdge<'_>,
    c: &Coplanar<'_>,
    (sp1, sp2): (&mut IntPolyhStartPoint, &mut IntPolyhStartPoint),
    nb_points: &mut i32,
) {
    let mut per = IntPolyhPoint::default();
    per.cross(a.normale, c.cote);
    let (pe1, pe2) = (a.pe[0], a.pe[1]);
    let (pt1, pt2) = c.pt;
    let (p1p, p2p, p0p) = (per.dot(pe1), per.dot(pe2), per.dot(pt1));
    // The edge and PT1 are projected on the perpendicular of the side in its plane.
    let between: bool = (p1p >= p0p && p0p >= p2p) || (p1p <= p0p && p0p <= p2p);
    if !(between && (p1p - p2p).abs() > MCP) {
        return;
    }
    let lambda: f64 = (p1p - p0p) / (p1p - p2p);
    let pie: IntPolyhPoint = if lambda.abs() < MCP {
        *pe1
    } else if lambda.abs() > 1.0 - MCP {
        *pe2
    } else {
        *pe1 + *a.edge * lambda
    };
    let cote = c.cote;
    let alpha: f64 = if cote.x() != 0.0 {
        (pie.x() - pt1.x()) / cote.x()
    } else if cote.y() != 0.0 {
        (pie.y() - pt1.y()) / cote.y()
    } else if cote.z() != 0.0 {
        (pie.z() - pt1.z()) / cote.z()
    } else {
        f64::MAX
    };
    if alpha < -MCP {
        return;
    }
    let point = CrossingPoint {
        pie,
        alpha,
        ends: (pt1, pt2),
    };
    if *nb_points == 0 {
        point.store(a, c, sp1, true);
        *nb_points += i32::from(a.tri_surf_id == 1 || a.tri_surf_id == 2);
    } else if *nb_points == 1 {
        point.store(a, c, sp2, false);
        *nb_points += i32::from(a.tri_surf_id == 1 || a.tri_surf_id == 2);
    }
}

/// A crossing point on the side, at `alpha` from `PT1`.
struct CrossingPoint<'a> {
    pie: IntPolyhPoint,
    alpha: f64,
    ends: (&'a IntPolyhPoint, &'a IntPolyhPoint),
}

impl CrossingPoint<'_> {
    /// Writes the point into `sp` (the first start point when `first`): the parameters
    /// of the side and of the edge on each surface. As natively, an end of the side at
    /// `alpha == 0` is overwritten by the inner case (the second test has no `else`), and
    /// the second point of surface 2 reads the orientation of the first triangle.
    fn store(
        &self,
        a: &IntPolyhTriangleEdge<'_>,
        c: &Coplanar<'_>,
        sp: &mut IntPolyhStartPoint,
        first: bool,
    ) {
        let (pie, alpha) = (self.pie, self.alpha);
        sp.set_xyz(pie.x(), pie.y(), pie.z());
        let on_first: bool = a.tri_surf_id == 1;
        if a.tri_surf_id != 1 && a.tri_surf_id != 2 {
            return;
        }
        let side_uv = |sp: &mut IntPolyhStartPoint, p: &IntPolyhPoint| {
            if on_first {
                sp.set_uv1(p.u(), p.v());
                sp.set_uv2(pie.u(), pie.v());
                sp.set_edge1(-1);
            } else {
                sp.set_uv2(p.u(), p.v());
                sp.set_uv1(pie.u(), pie.v());
                sp.set_edge2(-1);
            }
        };
        if alpha.abs() < MCP {
            side_uv(sp, self.ends.0);
        }
        if alpha.abs() > 1.0 - MCP {
            side_uv(sp, self.ends.1);
            return;
        }
        let (pt1, cote) = (self.ends.0, c.cote);
        let (u, v) = (pt1.u() + cote.u() * alpha, pt1.v() + cote.v() * alpha);
        if on_first {
            sp.set_uv1(u, v);
            sp.set_uv2(pie.u(), pie.v());
            sp.set_edge1(a.tris.0.get_edge_number(c.cote_index));
            let forward: bool = a.tris.0.get_edge_orientation(c.cote_index) > 0;
            sp.set_lambda1(if forward { alpha } else { 1.0 - alpha });
            return;
        }
        sp.set_uv1(pie.u(), pie.v());
        sp.set_uv2(u, v);
        sp.set_edge2(a.tris.1.get_edge_number(c.cote_index));
        let orientation_tri = if first { a.tris.1 } else { a.tris.0 };
        let forward: bool = orientation_tri.get_edge_orientation(c.cote_index) > 0;
        sp.set_lambda2(if forward { alpha } else { 1.0 - alpha });
    }
}

/// The ends of the overlap of the edge and the side: `(lambda, PEP)` on the edge and
/// `(alpha, PTP)` on the side, for the first and the second point.
#[derive(Default)]
struct Overlap {
    lambda1: f64,
    lambda2: f64,
    alpha1: f64,
    alpha2: f64,
    pep1: IntPolyhPoint,
    ptp1: IntPolyhPoint,
    pep2: IntPolyhPoint,
    ptp2: IntPolyhPoint,
}

/// The edge and the side are parallel (and on the same line after the rejections): the
/// ends of their overlap, projected on the side.
fn overlap(
    a: &IntPolyhTriangleEdge<'_>,
    c: &Coplanar<'_>,
    (sp1, sp2): (&mut IntPolyhStartPoint, &mut IntPolyhStartPoint),
    nb_points: &mut i32,
) {
    let (pe1, pe2) = (a.pe[0], a.pe[1]);
    let (pt1, pt2) = c.pt;
    let cote = c.cote;
    let p = [cote.dot(pe1), cote.dot(pe2), cote.dot(pt1), cote.dot(pt2)];
    let mut o = Overlap::default();
    if p[0] > p[1] {
        overlap_decreasing(a, c, p, &mut o, nb_points);
    }
    if p[0] < p[1] {
        overlap_increasing(a, c, p, &mut o, nb_points);
    }
    if *nb_points == 0 {
        return;
    }
    store_overlap(a, c, &o, sp1, true);
    // It is checked that PEP1 != PEP2.
    if *nb_points == 2
        && (o.pep1.u() - o.pep2.u()).abs() < MCP
        && (o.pep1.v() - o.pep2.v()).abs() < MCP
    {
        *nb_points = 1;
    }
    if *nb_points == 2 {
        store_overlap(a, c, &o, sp2, false);
    }
}

/// The overlap when the edge runs against the side (`pe1p > pe2p`).
fn overlap_decreasing(
    a: &IntPolyhTriangleEdge<'_>,
    c: &Coplanar<'_>,
    p: [f64; 4],
    o: &mut Overlap,
    nb: &mut i32,
) {
    let [pe1p, pe2p, pt1p, pt2p] = p;
    let (pe1, pe2, edge) = (a.pe[0], a.pe[1], a.edge);
    let (pt1, pt2) = c.pt;
    let cote = c.cote;
    if pt1p < pe1p && pe1p <= pt2p {
        (o.lambda1, o.pep1) = (0.0, *pe1);
        o.alpha1 = (pe1p - pt1p) / (pt2p - pt1p);
        o.ptp1 = *pt1 + *cote * o.alpha1;
        *nb = 1;
        if pt1p <= pe2p {
            (o.lambda2, o.pep2) = (1.0, *pe2);
            o.alpha2 = (pe2p - pt1p) / (pt2p - pt1p);
            o.ptp2 = *pt1 + *cote * o.alpha2;
        } else {
            o.lambda2 = (pt1p - pe1p) / (pe2p - pe1p);
            o.pep2 = *pe1 + *edge * o.lambda2;
            (o.alpha2, o.ptp2) = (0.0, *pt1);
        }
        *nb = 2;
    } else if pt2p < pe1p && pe1p <= pt1p {
        (o.lambda1, o.pep1) = (0.0, *pe1);
        o.alpha1 = (pt1p - pe1p) / (pt1p - pt2p);
        o.ptp1 = *pt1 + *cote * o.alpha1;
        *nb = 1;
        if pt2p <= pe2p {
            (o.lambda2, o.pep2) = (1.0, *pe2);
            o.alpha2 = (pe2p - pt1p) / (pt2p - pt1p);
            o.ptp2 = *pt1 + *cote * o.alpha2;
        } else {
            o.lambda2 = (pt2p - pe1p) / (pe2p - pe1p);
            o.pep2 = *pe1 + *edge * o.lambda2;
            (o.alpha2, o.ptp2) = (1.0, *pt2);
        }
        *nb = 2;
    }
}

/// The overlap when the edge runs with the side (`pe1p < pe2p`); the second point of the
/// first case starts from PE2, as natively.
fn overlap_increasing(
    a: &IntPolyhTriangleEdge<'_>,
    c: &Coplanar<'_>,
    p: [f64; 4],
    o: &mut Overlap,
    nb: &mut i32,
) {
    let [pe1p, pe2p, pt1p, pt2p] = p;
    let (pe1, pe2, edge) = (a.pe[0], a.pe[1], a.edge);
    let (pt1, pt2) = c.pt;
    let cote = c.cote;
    if pt1p < pe2p && pe2p <= pt2p {
        (o.lambda1, o.pep1) = (1.0, *pe2);
        o.alpha1 = (pe2p - pt1p) / (pt2p - pt1p);
        o.ptp1 = *pt1 + *cote * o.alpha1;
        *nb = 1;
        if pt1p <= pe1p {
            (o.lambda2, o.pep2) = (0.0, *pe1);
            o.alpha2 = (pe1p - pt1p) / (pt2p - pt1p);
            o.ptp2 = *pt1 + *cote * o.alpha2;
        } else {
            o.lambda2 = (pt1p - pe1p) / (pe2p - pe1p);
            o.pep2 = *pe2 + *edge * o.lambda2;
            (o.alpha2, o.ptp2) = (0.0, *pt1);
        }
        *nb = 2;
    } else if pt2p < pe2p && pe2p <= pt1p {
        (o.lambda1, o.pep1) = (1.0, *pe2);
        o.alpha1 = (pt1p - pe2p) / (pt1p - pt2p);
        o.ptp1 = *pt1 + *cote * o.alpha1;
        *nb = 1;
        if pt2p <= pe1p {
            (o.lambda2, o.pep2) = (0.0, *pe1);
            o.alpha2 = (pe1p - pt1p) / (pt2p - pt1p);
            o.ptp2 = *pt1 + *cote * o.alpha2;
        } else {
            o.lambda2 = (pt2p - pe1p) / (pe2p - pe1p);
            o.pep2 = *pe1 + *edge * o.lambda2;
            (o.alpha2, o.ptp2) = (1.0, *pt2);
        }
        *nb = 2;
    }
}

/// Writes an end of the overlap into `sp` (the first end when `first`). Both points take
/// the side parameter `alpha1` and the edge parameter `lambda1`, and on surface 2 the
/// parameters mix the U of one point with the V of the other, as natively; the second
/// point of surface 2 reads the orientation of the side from the first triangle.
fn store_overlap(
    a: &IntPolyhTriangleEdge<'_>,
    c: &Coplanar<'_>,
    o: &Overlap,
    sp: &mut IntPolyhStartPoint,
    first: bool,
) {
    let (pep, ptp) = if first {
        (o.pep1, o.ptp1)
    } else {
        (o.pep2, o.ptp2)
    };
    sp.set_xyz(pep.x(), pep.y(), pep.z());
    let (tri1, tri2) = a.tris;
    let pick = |forward: bool, value: f64| if forward { value } else { 1.0 - value };
    if a.tri_surf_id == 1 {
        // The side belongs to Tri1.
        sp.set_uv1(ptp.u(), ptp.v());
        sp.set_uv2(pep.u(), pep.v());
        sp.set_edge1(tri1.get_edge_number(c.cote_index));
        sp.set_lambda1(pick(tri1.get_edge_orientation(c.cote_index) > 0, o.alpha1));
        sp.set_lambda2(pick(tri2.get_edge_orientation(a.edge_index) > 0, o.lambda1));
    }
    if a.tri_surf_id == 2 {
        // The side belongs to Tri2.
        sp.set_uv1(pep.u(), ptp.v());
        sp.set_uv2(ptp.u(), pep.v());
        sp.set_edge2(tri2.get_edge_number(c.cote_index));
        let (side_tri, edge_tri) = if first { (tri2, tri1) } else { (tri1, tri2) };
        sp.set_lambda1(pick(
            side_tri.get_edge_orientation(c.cote_index) > 0,
            o.alpha1,
        ));
        sp.set_lambda2(pick(
            edge_tri.get_edge_orientation(a.edge_index) > 0,
            o.lambda1,
        ));
    }
}

/// A point at an end of an edge (parameter 0 or 1) is a vertex: its edge becomes -1.
fn filter_vertices(nb_points: i32, sp1: &mut IntPolyhStartPoint, sp2: &mut IntPolyhStartPoint) {
    let filter = |sp: &mut IntPolyhStartPoint| {
        if sp.lambda1().abs() < MCP {
            sp.set_edge1(-1);
        }
        if (sp.lambda1() - 1.0).abs() < MCP {
            sp.set_edge1(-1);
        }
        if sp.lambda2().abs() < MCP {
            sp.set_edge2(-1);
        }
        if (sp.lambda2() - 1.0).abs() < MCP {
            sp.set_edge2(-1);
        }
    };
    if nb_points > 0 {
        filter(sp1);
    }
    if nb_points == 2 {
        filter(sp2);
    }
}
