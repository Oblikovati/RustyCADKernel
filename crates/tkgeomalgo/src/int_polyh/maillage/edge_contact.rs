// Rust translation of TriangleEdgeContact of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection points of a triangle and an edge of the other triangulation: two at
//! most when they are coplanar, else the point where the edge crosses the plane, when it
//! is inside the triangle.
use super::coplanar::{Coplanar, calcul_pts_inter_tri_edge_coplanaires};
use super::{IntPolyhMaillageAffinage, MY_CONFUSION_PRECISION as MCP};
use crate::int_polyh::point::IntPolyhPoint;
use crate::int_polyh::start_point::IntPolyhStartPoint;
use crate::int_polyh::triangle::IntPolyhTriangle;
use rustycad_tkernel::standard::Failure;
use std::io::Write;

/// The arguments of `TriangleEdgeContact`: the surface of the triangle (1 or 2), the
/// rank 1..=3 of the edge in the other triangle, both triangles, the triangle points
/// `PT1..PT3`, its sides `Cote12, Cote23, Cote31`, the edge ends `PE1, PE2`, the edge
/// vector and the unit normal of the triangle.
#[derive(Clone, Copy)]
pub struct IntPolyhTriangleEdge<'a> {
    /// `TriSurfID`.
    pub tri_surf_id: i32,
    /// `EdgeIndex`.
    pub edge_index: i32,
    /// `Tri1`, `Tri2`.
    pub tris: (&'a IntPolyhTriangle, &'a IntPolyhTriangle),
    /// `PT1`, `PT2`, `PT3`.
    pub pt: [&'a IntPolyhPoint; 3],
    /// `Cote12`, `Cote23`, `Cote31`.
    pub cote: [&'a IntPolyhPoint; 3],
    /// `PE1`, `PE2`.
    pub pe: [&'a IntPolyhPoint; 2],
    /// `Edge`.
    pub edge: &'a IntPolyhPoint,
    /// `NormaleT`.
    pub normale: &'a IntPolyhPoint,
}

impl IntPolyhMaillageAffinage {
    /// `TriangleEdgeContact(TriSurfID, EdgeIndex, Tri1, Tri2, PT1, PT2, PT3, Cote12,
    /// Cote23, Cote31, PE1, PE2, Edge, NormaleT, SP1, SP2)`: the number of intersection
    /// points of the triangle and the edge written in `sp1` and `sp2` (the edge number of
    /// the other surface is written first in both); gp_VectorWithNullMagnitude from the
    /// coplanar case, as natively. `m.triangle_edge_contact(&args, &mut sp1, &mut sp2)?`.
    pub fn triangle_edge_contact(
        &self,
        args: &IntPolyhTriangleEdge<'_>,
        sp1: &mut IntPolyhStartPoint,
        sp2: &mut IntPolyhStartPoint,
    ) -> Result<i32, Failure> {
        triangle_edge_contact(args, sp1, sp2)
    }
}

/// The body of `TriangleEdgeContact`, which reads nothing of the algorithm.
pub(crate) fn triangle_edge_contact(
    a: &IntPolyhTriangleEdge<'_>,
    sp1: &mut IntPolyhStartPoint,
    sp2: &mut IntPolyhStartPoint,
) -> Result<i32, Failure> {
    // One of the edges on which the point lies is known.
    if a.tri_surf_id == 1 {
        let edge: i32 = a.tris.1.get_edge_number(a.edge_index);
        sp1.set_edge2(edge);
        sp2.set_edge2(edge);
    } else if a.tri_surf_id == 2 {
        let edge: i32 = a.tris.0.get_edge_number(a.edge_index);
        sp1.set_edge1(edge);
        sp2.set_edge1(edge);
    }
    let [c12, c23, c31] = a.cote;
    if a.normale.square_modulus() == 0.0
        || c12.square_modulus() == 0.0
        || c23.square_modulus() == 0.0
        || c31.square_modulus() == 0.0
        || a.edge.square_modulus() == 0.0
    {
        return Ok(0);
    }
    let pe1: f64 = a.normale.dot(a.pe[0]);
    let pe2: f64 = a.normale.dot(a.pe[1]);
    let pt1: f64 = a.normale.dot(a.pt[0]);
    if (pe1 - pt1).abs() < MCP && (pe2 - pt1).abs() < MCP {
        // The edge and the triangle are coplanar: two contact points at most.
        return coplanar_contact(a, sp1, sp2);
    }
    if (pe1 >= pt1 && pt1 >= pe2) || (pe1 <= pt1 && pt1 <= pe2) {
        return Ok(crossing_contact(a, (pe1, pe2, pt1), sp1));
    }
    Ok(0)
}

/// Whether the vertex `pk` is alone on its side of the edge line (`pk >= ppe1` with the
/// two others `<= ppe1`, or the reverse): its two sides may cut the edge.
fn two_sides_cut(pk: f64, others: [f64; 2], ppe1: f64) -> bool {
    (pk >= ppe1 && others[0] <= ppe1 && others[1] <= ppe1)
        || (pk <= ppe1 && others[0] >= ppe1 && others[1] >= ppe1)
}

/// `NbPoints > 1` with the same first parameters on both points: one point.
fn merge_same_uv1(nb_points: &mut i32, sp1: &IntPolyhStartPoint, sp2: &IntPolyhStartPoint) {
    if *nb_points > 1 && (sp2.u1() - sp1.u1()).abs() < MCP && (sp1.v1() - sp2.v1()).abs() < MCP {
        *nb_points = 1;
    }
}

/// The coplanar case: the vertices of the triangle are projected on the perpendicular
/// of the edge, and the two sides of the lone vertex are cut with the edge.
fn coplanar_contact(
    a: &IntPolyhTriangleEdge<'_>,
    sp1: &mut IntPolyhStartPoint,
    sp2: &mut IntPolyhStartPoint,
) -> Result<i32, Failure> {
    let mut perp_edge = IntPolyhPoint::default();
    perp_edge.cross(a.normale, a.edge);
    let pp: [f64; 3] = [
        perp_edge.dot(a.pt[0]),
        perp_edge.dot(a.pt[1]),
        perp_edge.dot(a.pt[2]),
    ];
    let ppe1: f64 = perp_edge.dot(a.pe[0]);
    let mut nb_points: i32 = 0;
    if (pp[0] - pp[1]).abs() < MCP && (pp[0] - pp[2]).abs() < MCP {
        return Ok(nb_points);
    }
    // The sides as the native calls pass them: (PT1, PT2, Cote12, 1), (PT2, PT3, Cote23,
    // 2) and (PT3, PT1, Cote31, 3); the lone vertex PT1 cuts its sides 1 then 3, PT2 its
    // sides 1 then 2, PT3 its sides 3 then 2.
    for (lone, sides) in [(0, [0, 2]), (1, [0, 1]), (2, [2, 1])] {
        let others: [f64; 2] = match lone {
            0 => [pp[1], pp[2]],
            1 => [pp[0], pp[2]],
            _ => [pp[0], pp[1]],
        };
        if two_sides_cut(pp[lone], others, ppe1) && (lone == 0 || nb_points < 2) {
            calcul_side(a, sides[0], (sp1, sp2), &mut nb_points)?;
            merge_same_uv1(&mut nb_points, sp1, sp2);
            if nb_points < 2 {
                calcul_side(a, sides[1], (sp1, sp2), &mut nb_points)?;
            }
        }
        if lone == 0 {
            // The native test compares V2 of the first point with V1 of the second one.
            if nb_points > 1
                && (sp1.u1() - sp2.u1()).abs() < MCP
                && (sp1.v2() - sp2.v1()).abs() < MCP
            {
                nb_points = 1;
            }
        } else {
            merge_same_uv1(&mut nb_points, sp1, sp2);
        }
        if nb_points >= 2 {
            return Ok(nb_points);
        }
    }
    Ok(nb_points)
}

/// `CalculPtsInterTriEdgeCoplanaires` on the side `side` (0: PT1PT2, 1: PT2PT3, 2: PT3PT1).
fn calcul_side(
    a: &IntPolyhTriangleEdge<'_>,
    side: usize,
    sp: (&mut IntPolyhStartPoint, &mut IntPolyhStartPoint),
    nb_points: &mut i32,
) -> Result<(), Failure> {
    let cote = Coplanar {
        pt: (a.pt[side], a.pt[(side + 1) % 3]),
        cote: a.cote[side],
        cote_index: side as i32 + 1,
    };
    calcul_pts_inter_tri_edge_coplanaires(a, &cote, sp, nb_points)
}

/// The point `PI` where the edge crosses the plane of the triangle (`lambda` on the
/// edge), with the edge parameter of the other surface.
fn crossing_point(
    a: &IntPolyhTriangleEdge<'_>,
    (pe1, pe2, pt1): (f64, f64, f64),
    sp1: &mut IntPolyhStartPoint,
) -> IntPolyhPoint {
    let lambda: f64 = (pe1 - pt1) / (pe1 - pe2);
    if lambda < -MCP {
        return IntPolyhPoint::default();
    }
    if lambda.abs() < MCP || (lambda - 1.0).abs() < MCP {
        // lambda == 0 or lambda == 1: an end of the edge.
        let at_start: bool = lambda.abs() < MCP;
        if a.tri_surf_id == 1 {
            sp1.set_edge2(-1);
        } else {
            sp1.set_edge1(-1);
        }
        return if at_start { *a.pe[0] } else { *a.pe[1] };
    }
    let pi: IntPolyhPoint = *a.pe[0] + *a.edge * lambda;
    if a.tri_surf_id == 1 {
        let forward: bool = a.tris.1.get_edge_orientation(a.edge_index) > 0;
        sp1.set_lambda2(if forward { lambda } else { 1.0 - lambda });
    }
    if a.tri_surf_id == 2 {
        let forward: bool = a.tris.0.get_edge_orientation(a.edge_index) > 0;
        sp1.set_lambda1(if forward { lambda } else { 1.0 - lambda });
    }
    pi
}

/// The non coplanar case: `PI` inside the triangle, located by the parameters `alpha`
/// (along `Cote12`) and `beta` (along `Cote23`) solved from two of the coordinates.
fn crossing_contact(
    a: &IntPolyhTriangleEdge<'_>,
    planes: (f64, f64, f64),
    sp1: &mut IntPolyhStartPoint,
) -> i32 {
    let pi: IntPolyhPoint = crossing_point(a, planes, sp1);
    let Some((alpha, beta)) = triangle_coordinates(a, &pi) else {
        return 0;
    };
    if beta < -MCP || beta > (alpha + MCP) {
        return 0;
    }
    sp1.set_xyz(pi.x(), pi.y(), pi.z());
    let (pt1, c12, c23) = (a.pt[0], a.cote[0], a.cote[1]);
    let u: f64 = pt1.u() + c12.u() * alpha + c23.u() * beta;
    let v: f64 = pt1.v() + c12.v() * alpha + c23.v() * beta;
    if a.tri_surf_id == 1 {
        sp1.set_uv2(pi.u(), pi.v());
        sp1.set_uv1(u, v);
        locate_on_triangle(a, (alpha, beta), sp1, true);
        return 1;
    }
    if a.tri_surf_id == 2 {
        sp1.set_uv1(pi.u(), pi.v());
        sp1.set_uv2(u, v);
        locate_on_triangle(a, (alpha, beta), sp1, false);
        return 1;
    }
    0
}

/// The vertex or side of the triangle the point lies on (surface 1 when `first`): its
/// point and parameters for a vertex (edge -1), its edge and parameter for a side.
fn locate_on_triangle(
    a: &IntPolyhTriangleEdge<'_>,
    (alpha, beta): (f64, f64),
    sp1: &mut IntPolyhStartPoint,
    first: bool,
) {
    let tri: &IntPolyhTriangle = if first { a.tris.0 } else { a.tris.1 };
    let vertex = |sp1: &mut IntPolyhStartPoint, p: &IntPolyhPoint| {
        sp1.set_xyz(p.x(), p.y(), p.z());
        if first {
            sp1.set_uv1(p.u(), p.v());
            sp1.set_edge1(-1);
        } else {
            sp1.set_uv2(p.u(), p.v());
            sp1.set_edge2(-1);
        }
    };
    let side = |sp1: &mut IntPolyhStartPoint, rank: i32, forward_value: f64, reverse_value: f64| {
        let value: f64 = if tri.get_edge_orientation(rank) > 0 {
            forward_value
        } else {
            reverse_value
        };
        if first {
            sp1.set_edge1(tri.get_edge_number(rank));
            sp1.set_lambda1(value);
        } else {
            sp1.set_edge2(tri.get_edge_number(rank));
            sp1.set_lambda2(value);
        }
    };
    if alpha < MCP {
        // alpha == 0, so beta == 0.
        vertex(sp1, a.pt[0]);
    } else if beta < MCP && (1.0 - alpha).abs() < MCP {
        vertex(sp1, a.pt[1]);
    } else if (beta - 1.0).abs() < MCP && (1.0 - alpha).abs() < MCP {
        vertex(sp1, a.pt[2]);
    } else if beta < MCP {
        side(sp1, 1, alpha, 1.0 - alpha);
    } else if (beta - alpha).abs() < MCP {
        side(sp1, 3, 1.0 - alpha, alpha);
    } else if (alpha - 1.0).abs() < MCP {
        // On the side PT2PT3 the first surface takes beta, the second one alpha, as
        // natively.
        let t: f64 = if first { beta } else { alpha };
        side(sp1, 2, t, 1.0 - t);
    }
}

/// `alpha` and `beta` of `PI = PT1 + alpha Cote12 + beta Cote23`, from the first pair of
/// coordinate equations that is well conditioned; `None` when `alpha` is outside
/// `[0, 1]` (the native `return 0`). `beta` stays 0 where the native branch sets none.
fn triangle_coordinates(a: &IntPolyhTriangleEdge<'_>, pi: &IntPolyhPoint) -> Option<(f64, f64)> {
    let (pt1, c12, c23) = (a.pt[0], a.cote[0], a.cote[1]);
    let outside = |alpha: f64| alpha < -MCP || alpha > (1.0 + MCP);
    let c23x: f64 = c23.x();
    let mut d1: f64 = 0.0;
    // Combination of the equations 1 and 2.
    if c23x.abs() > MCP {
        d1 = c12.y() - c12.x() * c23.y() / c23x;
    }
    if d1.abs() > MCP {
        let alpha: f64 = (pi.y() - pt1.y() - (pi.x() - pt1.x()) * c23.y() / c23x) / d1;
        return (!outside(alpha)).then(|| (alpha, (pi.x() - pt1.x() - alpha * c12.x()) / c23x));
    }
    if c12.x().abs() > MCP && c23x.abs() < MCP {
        let alpha: f64 = (pi.x() - pt1.x()) / c12.x();
        if outside(alpha) {
            return None;
        }
        if c23.y().abs() > MCP {
            return Some((alpha, (pi.y() - pt1.y() - alpha * c12.y()) / c23.y()));
        }
        if c23.z().abs() > MCP {
            return Some((alpha, (pi.z() - pt1.z() - alpha * c12.z()) / c23.z()));
        }
        return Some((alpha, 0.0));
    }
    other_coordinates(a, pi)
}

/// The remaining combinations of `triangle_coordinates`: equations 1 and 3, 2 and 3,
/// then the degenerate ones; RealLast for both when no pair is usable.
fn other_coordinates(a: &IntPolyhTriangleEdge<'_>, pi: &IntPolyhPoint) -> Option<(f64, f64)> {
    let (pt1, c12, c23) = (a.pt[0], a.cote[0], a.cote[1]);
    let outside = |alpha: f64| alpha < -MCP || alpha > (1.0 + MCP);
    if c23.x().abs() > MCP {
        let d3: f64 = c12.z() - c12.x() * c23.z() / c23.x();
        if d3.abs() > MCP {
            let alpha: f64 = (pi.z() - pt1.z() - (pi.x() - pt1.x()) * c23.z() / c23.x()) / d3;
            return (!outside(alpha))
                .then(|| (alpha, (pi.x() - pt1.x() - alpha * c12.x()) / c23.x()));
        }
    }
    if c23.y().abs() > MCP {
        let d4: f64 = c12.z() - c12.y() * c23.z() / c23.y();
        if d4.abs() > MCP {
            let alpha: f64 = (pi.z() - pt1.z() - (pi.y() - pt1.y()) * c23.z() / c23.y()) / d4;
            return (!outside(alpha))
                .then(|| (alpha, (pi.y() - pt1.y() - alpha * c12.y()) / c23.y()));
        }
    }
    if c12.y().abs() > MCP && c23.y().abs() < MCP {
        let alpha: f64 = (pi.y() - pt1.y()) / c12.y();
        if outside(alpha) {
            return None;
        }
        if c23.z().abs() > MCP {
            return Some((alpha, (pi.z() - pt1.z() - alpha * c12.z()) / c23.z()));
        }
        null_side_warning(a);
        return Some((alpha, 0.0));
    }
    if c12.z().abs() > MCP && c23.z().abs() < MCP {
        let alpha: f64 = (pi.z() - pt1.z()) / c12.z();
        if outside(alpha) {
            return None;
        }
        if c23.x().abs() > MCP {
            return Some((alpha, (pi.x() - pt1.x() - alpha * c12.x()) / c23.x()));
        }
        return Some((alpha, 0.0));
    }
    // A particular case not processed.
    Some((f64::MAX, f64::MAX))
}

/// The native diagnostic of a null side `PT2PT3`, printed to standard output.
fn null_side_warning(a: &IntPolyhTriangleEdge<'_>) {
    let mut text = String::from("\nCote PT2PT3 nul1\n");
    let _ = a.pt[1].dump_index(2004, &mut text);
    let _ = a.pt[2].dump_index(3004, &mut text);
    let _ = std::io::stdout().write_all(text.as_bytes());
}
