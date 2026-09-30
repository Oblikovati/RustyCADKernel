// Rust translation of FillArrayOfEdges and FillArrayOfTriangles of
// ModelingAlgorithms/TKGeomAlgo/IntPolyh/IntPolyh_MaillageAffinage.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The triangles and edges of the regular grid of a surface: two triangles per cell
//! (`(U,V) (U,V+1) (U+1,V+1)` and `(U,V) (U+1,V+1) (U+1,V)`), numbered cell by cell,
//! and the edges numbered as the native loops create them.
use super::{IntPolyhMaillageAffinage, Side};
use crate::int_polyh::edge::IntPolyhEdge;

/// One edge of the grid: its points and its (up to two) triangles, -1 when unset.
struct GridEdge {
    points: (i32, i32),
    first: i32,
    second: i32,
}

impl IntPolyhMaillageAffinage {
    /// `FillArrayOfTriangles(SurfID)`: two triangles per grid cell; a triangle whose
    /// three sides each join points with a common out code of the common box cannot
    /// intersect. `m.fill_array_of_triangles(1)`.
    pub fn fill_array_of_triangles(&mut self, surf_id: i32) {
        let side: &mut Side = &mut self.sides[Self::rank(surf_id)];
        let (nb_u, nb_v) = (side.nb_samples_u, side.nb_samples_v);
        side.triangles.init(2 * (nb_u - 1) * (nb_v - 1));
        let (mut cpteur, mut pnt_init) = (0, 0);
        for _ in 0..nb_u - 1 {
            for _ in 0..nb_v - 1 {
                // The first triangle, then the second one of the cell.
                for [a, b, c] in [
                    [pnt_init, pnt_init + 1, pnt_init + nb_v + 1],
                    [pnt_init, pnt_init + nb_v + 1, pnt_init + nb_v],
                ] {
                    let outside: bool = shares_out_codes(side, [a, b, c]);
                    let triangle = &mut side.triangles[cpteur];
                    triangle.set_first_point(a);
                    triangle.set_second_point(b);
                    triangle.set_third_point(c);
                    if outside {
                        triangle.set_intersection_possible(false);
                    }
                    cpteur += 1;
                }
                pnt_init += 1;
            }
            pnt_init += 1;
        }
        side.triangles.set_nb_items(cpteur);
    }

    /// `FillArrayOfEdges(SurfID)`: the edges of the grid triangles with their
    /// triangles, and each triangle's edge numbers and orientations.
    /// `m.fill_array_of_edges(1)`.
    pub fn fill_array_of_edges(&mut self, surf_id: i32) {
        let side: &mut Side = &mut self.sides[Self::rank(surf_id)];
        let (nb_u, nb_v) = (side.nb_samples_u, side.nb_samples_v);
        // NbSamplesU and NbSamplesV cannot be less than 2.
        let nb_edges: i32 = 3 * nb_u * nb_v - 2 * (nb_u + nb_v) + 1;
        side.edges.init(nb_edges);
        let mut count: i32 = 0;
        for edge in grid_edges(nb_u, nb_v) {
            add_edge(side, &mut count, edge);
        }
        side.edges.set_nb_items(count);
    }
}

/// The test of `FillArrayOfTriangles`: every side of the triangle joins two points with
/// a common out code.
fn shares_out_codes(side: &Side, [a, b, c]: [i32; 3]) -> bool {
    let (pa, pb, pc) = (
        side.points[a].part_of_common(),
        side.points[b].part_of_common(),
        side.points[c].part_of_common(),
    );
    (pa & pb) != 0 && (pb & pc) != 0 && (pc & pa) != 0
}

/// Stores `edge` as the edge `count`: its first triangle (when set) gets the edge first,
/// then the second one, as the native code sets each side after writing its triangle.
fn add_edge(side: &mut Side, count: &mut i32, edge: GridEdge) {
    let index: i32 = *count;
    let slot: &mut IntPolyhEdge = &mut side.edges[index];
    slot.set_first_point(edge.points.0);
    slot.set_second_point(edge.points.1);
    for (triangle, is_first) in [(edge.first, true), (edge.second, false)] {
        if triangle < 0 {
            continue;
        }
        if is_first {
            side.edges[index].set_first_triangle(triangle);
        } else {
            side.edges[index].set_second_triangle(triangle);
        }
        let stored: IntPolyhEdge = side.edges[index];
        side.triangles[triangle].set_edge_and_orientation(&stored, index);
    }
    *count += 1;
}

/// The edges of the grid in the native creation order: the first cell, the column
/// U = u0, the row V = v0, the inner points (three edges each), then the closing column
/// U = u1 and row V = v1.
fn grid_edges(nb_u: i32, nb_v: i32) -> Vec<GridEdge> {
    let e = |p1: i32, p2: i32, first: i32, second: i32| GridEdge {
        points: (p1, p2),
        first,
        second,
    };
    let mut edges: Vec<GridEdge> = vec![e(0, 1, -1, 0), e(0, nb_v, 1, -1), e(0, nb_v + 1, 0, 1)];
    for (k, b) in (1..nb_v - 1).enumerate() {
        let p: i32 = 1 + k as i32;
        edges.push(e(p, p + 1, -1, b * 2));
        edges.push(e(p, p + nb_v + 1, b * 2, b * 2 + 1));
        edges.push(e(p, p + nb_v, b * 2 + 1, b * 2 - 2));
    }
    for (k, b) in (1..nb_u - 1).enumerate() {
        let p: i32 = nb_v * (1 + k as i32);
        let (t0, t1) = ((b - 1) * (nb_v - 1) * 2, b * (nb_v - 1) * 2);
        edges.push(e(p, p + 1, t0 + 1, t1));
        edges.push(e(p, p + nb_v + 1, t1, t1 + 1));
        edges.push(e(p, p + nb_v, t1 + 1, -1));
    }
    inner_edges(nb_u, nb_v, &mut edges);
    let last_column: i32 = (nb_u - 1) * nb_v;
    for b in 0..nb_v - 1 {
        let t: i32 = (nb_u - 2) * (nb_v - 1) * 2 + b * 2 + 1;
        edges.push(e(last_column + b, last_column + b + 1, t, -1));
    }
    for b in 0..nb_u - 1 {
        let (p1, p2) = (nb_v - 1 + b * nb_v, nb_v - 1 + (b + 1) * nb_v);
        edges.push(e(p1, p2, -1, b * 2 * (nb_v - 1) + (nb_v - 2) * 2));
    }
    edges
}

/// The three edges of each inner point, from `(U, V)` to `(U, V+1)`, `(U+1, V+1)` and
/// `(U+1, V)`.
fn inner_edges(nb_u: i32, nb_v: i32, edges: &mut Vec<GridEdge>) {
    let mut p: i32 = nb_v + 1;
    for bu in 1..nb_u - 1 {
        for bv in 1..nb_v - 1 {
            let t: i32 = (nb_v - 1) * 2 * bu + bv * 2;
            let t_prev: i32 = (nb_v - 1) * 2 * (bu - 1) + bv * 2 + 1;
            edges.push(GridEdge {
                points: (p, p + 1),
                first: t_prev,
                second: t,
            });
            edges.push(GridEdge {
                points: (p, p + nb_v + 1),
                first: t,
                second: t + 1,
            });
            edges.push(GridEdge {
                points: (p, p + nb_v),
                first: t + 1,
                second: t - 2,
            });
            p += 1;
        }
        p += 2;
    }
}
