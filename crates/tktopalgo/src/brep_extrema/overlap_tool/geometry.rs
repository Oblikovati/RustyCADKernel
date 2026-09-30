// Rust translation of the anonymous namespace of
// ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_OverlapTool.cxx (BRepExtrema_BoundingPrism,
// sign, pointInTriangle, segmentsIntersected, trianglesIntersected, prismsIntersected,
// overlapBoxes) and getProjectionAxes of BRepExtrema_SelfIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exact and tolerant triangle overlap tests of the overlap tool.
use rustycad_tkernel::precision;
use rustycad_tkmath::bvh::{BvhVec2d, BvhVec3d, std_max, std_min};

/// `DBL_EPSILON`.
const DBL_EPSILON: f64 = f64::EPSILON;

/// `BRepExtrema_BoundingPrism`: the prism of a triangle offset by the deflection
/// (three vertices above the triangle, three below), its edges and normals.
#[derive(Clone, Copy, Default)]
pub(super) struct BoundingPrism {
    vertices: [BvhVec3d; 6],
    edges: [BvhVec3d; 3],
    normal: BvhVec3d,
    edge_normals: [BvhVec3d; 3],
    pub(super) is_inited: bool,
}

impl BoundingPrism {
    /// `BRepExtrema_BoundingPrism(v0, v1, v2, theDeflect)`.
    pub(super) fn new(vertices: [BvhVec3d; 3], deflect: f64) -> Self {
        let mut prism = Self::default();
        prism.init(vertices, deflect);
        prism
    }

    /// `Init(v0, v1, v2, theDeflect)`.
    pub(super) fn init(&mut self, [v0, v1, v2]: [BvhVec3d; 3], deflect: f64) {
        self.edges = [v1 - v0, v2 - v0, v2 - v1];
        self.normal = BvhVec3d::cross(&self.edges[0], &self.edges[1]);
        for (edge_normal, edge) in self.edge_normals.iter_mut().zip(&self.edges) {
            *edge_normal = BvhVec3d::cross(edge, &self.normal);
            *edge_normal *= 1.0 / std_max(edge_normal.modulus(), precision::confusion());
        }
        let [n0, n1, n2] = self.edge_normals;
        let direct01: BvhVec3d = n0 - n1;
        let direct02: BvhVec3d = n0 + n2;
        let direct12: BvhVec3d = n2 - n1;
        let bases: [BvhVec3d; 3] = [
            v0 + direct01 * (deflect / direct01.dot(&n0)),
            v1 + direct02 * (deflect / direct02.dot(&n2)),
            v2 + direct12 * (deflect / direct12.dot(&n2)),
        ];
        let norm_offset: BvhVec3d =
            self.normal * (deflect / std_max(self.normal.modulus(), precision::confusion()));
        for (i, base) in bases.iter().enumerate() {
            self.vertices[i] = *base + norm_offset;
            self.vertices[i + 3] = *base - norm_offset;
        }
        self.is_inited = true;
    }

    /// `Separated(thePrism, theAxis)`: whether the projections of the two prisms on the
    /// axis are disjoint; false as soon as the running intervals overlap.
    fn separated(&self, prism: &Self, axis: &BvhVec3d) -> bool {
        let (mut min1, mut max1, mut min2, mut max2) = (f64::MAX, -f64::MAX, f64::MAX, -f64::MAX);
        for (vertex1, vertex2) in self.vertices.iter().zip(&prism.vertices) {
            let proj1: f64 = vertex1.dot(axis);
            min1 = std_min(min1, proj1);
            max1 = std_max(max1, proj1);
            let proj2: f64 = vertex2.dot(axis);
            min2 = std_min(min2, proj2);
            max2 = std_max(max2, proj2);
            if min1 <= max2 && max1 >= min2 {
                return false;
            }
        }
        min1 > max2 || max1 < min2
    }
}

/// `sign(v0, v1, v2, theX, theY)`: the 2D orientation of three points in the plane of
/// the axes `x` and `y`.
fn sign(v0: &BvhVec3d, v1: &BvhVec3d, v2: &BvhVec3d, x: usize, y: usize) -> f64 {
    (v0[x] - v2[x]) * (v1[y] - v2[y]) - (v1[x] - v2[x]) * (v0[y] - v2[y])
}

/// `pointInTriangle`: the three orientations of the point have the same sign.
fn point_in_triangle(point: &BvhVec3d, [t0, t1, t2]: [&BvhVec3d; 3], x: usize, y: usize) -> bool {
    let sign0: bool = sign(point, t0, t1, x, y) <= 0.0;
    let sign1: bool = sign(point, t1, t2, x, y) <= 0.0;
    let sign2: bool = sign(point, t2, t0, x, y) <= 0.0;
    sign0 == sign1 && sign1 == sign2
}

/// `segmentsIntersected(theOriginSeg0, theOriginSeg1, theDirectSeg0, theDirectSeg1)`.
fn segments_intersected(
    origin0: &BvhVec2d,
    origin1: &BvhVec2d,
    direct0: &BvhVec2d,
    direct1: &BvhVec2d,
) -> bool {
    let det: f64 = -direct1.x * direct0.y + direct0.x * direct1.y;
    if det.abs() < DBL_EPSILON {
        return parallel_segments_intersected(origin0, origin1, direct0, direct1);
    }
    let delta: BvhVec2d = *origin0 - *origin1;
    let u: f64 = (-direct0.y * delta.x + direct0.x * delta.y) / det;
    let v: f64 = (direct1.x * delta.y - direct1.y * delta.x) / det;
    (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)
}

/// The parallel case of `segmentsIntersected`: overlapping intervals on the common
/// direction and the same supporting line.
fn parallel_segments_intersected(
    origin0: &BvhVec2d,
    origin1: &BvhVec2d,
    direct0: &BvhVec2d,
    direct1: &BvhVec2d,
) -> bool {
    let direct: BvhVec2d = *direct0 * (1.0 / direct0.modulus());
    let edge0_time0: f64 = origin0.dot(&direct);
    let edge1_time0: f64 = origin1.dot(&direct);
    let edge0_time1: f64 = edge0_time0 + direct0.dot(&direct);
    let edge1_time1: f64 = edge1_time0 + direct1.dot(&direct);
    let edge0_min: f64 = std_min(edge0_time0, edge0_time1);
    let edge1_min: f64 = std_min(edge1_time0, edge1_time1);
    let edge0_max: f64 = std_max(edge0_time0, edge0_time1);
    let edge1_max: f64 = std_max(edge1_time0, edge1_time1);
    if std_max(edge0_min, edge1_min) > std_min(edge0_max, edge1_max) {
        return false;
    }
    let normal = BvhVec2d::new(-direct.y, direct.x);
    (origin0.dot(&normal) - origin1.dot(&normal)).abs() < DBL_EPSILON
}

/// `getProjectionAxes(theNorm, theAxisX, theAxisY)` (and the same selection in
/// `trianglesIntersected`): the two axes of the projection plane of a normal.
pub(crate) fn projection_axes(normal: &BvhVec3d) -> (usize, usize) {
    let (n0, n1, n2) = (normal[0].abs(), normal[1].abs(), normal[2].abs());
    if n0 > n1 {
        return (if n0 > n2 { 1 } else { 0 }, if n0 > n2 { 2 } else { 1 });
    }
    // Both branches of the native X axis are 0.
    (0, if n1 > n2 { 2 } else { 1 })
}

/// The signed distances of the three vertices of a triangle to a plane.
fn plane_distances(normal: &BvhVec3d, plane_dist: f64, [a, b, c]: [&BvhVec3d; 3]) -> [f64; 3] {
    [
        normal.dot(a) + plane_dist,
        normal.dot(b) + plane_dist,
        normal.dot(c) + plane_dist,
    ]
}

/// True when the three distances are strictly on one side.
fn one_side([d0, d1, d2]: [f64; 3]) -> bool {
    (d0 < 0.0 && d1 < 0.0 && d2 < 0.0) || (d0 > 0.0 && d1 > 0.0 && d2 > 0.0)
}

/// The interval of a triangle on the intersection line of the two planes: its vertex
/// on the other side is moved to the middle (the native swaps), then the two
/// crossings of its edges with the plane, as `(min, max)`.
fn line_interval(
    vertices: [&BvhVec3d; 3],
    mut dist: [f64; 3],
    cross_line: &BvhVec3d,
) -> (f64, f64) {
    let mut proj: [f64; 3] = [
        vertices[0].dot(cross_line),
        vertices[1].dot(cross_line),
        vertices[2].dot(cross_line),
    ];
    if dist[0] * dist[1] > 0.0 {
        dist.swap(1, 2);
        proj.swap(1, 2);
    } else if dist[1] * dist[2] > 0.0 {
        dist.swap(1, 0);
        proj.swap(1, 0);
    }
    let time1: f64 = if dist[0].abs() <= DBL_EPSILON {
        proj[0]
    } else {
        proj[0] + (proj[1] - proj[0]) * dist[0] / (dist[0] - dist[1])
    };
    let time2: f64 = if dist[2].abs() <= DBL_EPSILON {
        proj[2]
    } else {
        proj[2] + (proj[1] - proj[2]) * dist[2] / (dist[2] - dist[1])
    };
    (std_min(time1, time2), std_max(time1, time2))
}

/// `trianglesIntersected`: the exact overlap test of two triangles (the interval test
/// on the line of the two planes, or the 2D edge and containment tests of coplanar
/// triangles).
pub(super) fn triangles_intersected(trng0: [&BvhVec3d; 3], trng1: [&BvhVec3d; 3]) -> bool {
    let trng1_normal: BvhVec3d =
        BvhVec3d::cross(&(*trng1[1] - *trng1[0]), &(*trng1[2] - *trng1[0])).normalized();
    let trng1_plane_dist: f64 = trng1_normal.dot(&-*trng1[0]);
    let dist0: [f64; 3] = plane_distances(&trng1_normal, trng1_plane_dist, trng0);
    if one_side(dist0) {
        return false; // 1st triangle lies on one side of the 2nd triangle
    }
    if !dist0.iter().any(|d| d.abs() > precision::confusion()) {
        return coplanar_intersected(trng0, trng1, &trng1_normal);
    }
    // General 3D case.
    let trng0_normal: BvhVec3d =
        BvhVec3d::cross(&(*trng0[1] - *trng0[0]), &(*trng0[2] - *trng0[0])).normalized();
    let trng0_plane_dist: f64 = trng0_normal.dot(&-*trng0[0]);
    let dist1: [f64; 3] = plane_distances(&trng0_normal, trng0_plane_dist, trng1);
    if one_side(dist1) {
        return false; // 2nd triangle lies on one side of the 1st triangle
    }
    let cross_line: BvhVec3d = BvhVec3d::cross(&trng0_normal, &trng1_normal);
    let (time_min1, time_max1) = line_interval(trng0, dist0, &cross_line);
    let (time_min2, time_max2) = line_interval(trng1, dist1, &cross_line);
    let time1: f64 = std_max(time_min1, time_min2);
    let time2: f64 = std_min(time_max1, time_max2);
    time1 <= time2 // intervals intersected --> triangles overlapped
}

/// The origins and directions of the three edges of a triangle projected on the axes
/// `x` and `y`.
fn flat_edges(trng: [&BvhVec3d; 3], x: usize, y: usize) -> ([BvhVec2d; 3], [BvhVec2d; 3]) {
    let flat = |v: &BvhVec3d| BvhVec2d::new(v[x], v[y]);
    let origin: [BvhVec2d; 3] = [flat(trng[0]), flat(trng[1]), flat(trng[2])];
    let direct: [BvhVec2d; 3] = [
        origin[1] - origin[0],
        origin[2] - origin[1],
        origin[0] - origin[2],
    ];
    (origin, direct)
}

/// The coplanar case of `trianglesIntersected`: the triangles projected on the plane
/// of the axes of the second normal.
fn coplanar_intersected(
    trng0: [&BvhVec3d; 3],
    trng1: [&BvhVec3d; 3],
    trng1_normal: &BvhVec3d,
) -> bool {
    let (x, y) = projection_axes(trng1_normal);
    let (origin0, direct0) = flat_edges(trng0, x, y);
    let (origin1, direct1) = flat_edges(trng1, x, y);
    for edge0 in 0..3 {
        for edge1 in 0..3 {
            let (o0, o1) = (&origin0[edge0], &origin1[edge1]);
            if segments_intersected(o0, o1, &direct0[edge0], &direct1[edge1]) {
                return true; // edges intersected --> triangles overlapped
            }
        }
    }
    if point_in_triangle(trng1[0], trng0, x, y) {
        return true; // 1st triangle inside 2nd --> triangles overlapped
    }
    point_in_triangle(trng0[0], trng1, x, y) // 2nd triangle inside 1st
}

/// `prismsIntersected`: the separating axis test of two prisms (both normals, the six
/// edge normals, and the 16 cross products of the edges and normals).
pub(super) fn prisms_intersected(prism1: &BoundingPrism, prism2: &BoundingPrism) -> bool {
    if prism1.separated(prism2, &prism1.normal) || prism1.separated(prism2, &prism2.normal) {
        return false;
    }
    let edge_normals = prism1.edge_normals.iter().chain(&prism2.edge_normals);
    if edge_normals
        .clone()
        .any(|axis| prism1.separated(prism2, axis))
    {
        return false;
    }
    for idx1 in 0..4 {
        let edge1: &BvhVec3d = if idx1 == 3 {
            &prism1.normal
        } else {
            &prism1.edges[idx1]
        };
        for idx2 in 0..4 {
            let edge2: &BvhVec3d = if idx2 == 3 {
                &prism2.normal
            } else {
                &prism2.edges[idx2]
            };
            if prism1.separated(prism2, &BvhVec3d::cross(edge1, edge2)) {
                return false;
            }
        }
    }
    true
}

/// `overlapBoxes`: the boxes overlap with the tolerance on each axis.
pub(super) fn overlap_boxes(
    min1: &BvhVec3d,
    max1: &BvhVec3d,
    min2: &BvhVec3d,
    max2: &BvhVec3d,
    tolerance: f64,
) -> bool {
    min1.x <= max2.x + tolerance
        && max1.x >= min2.x - tolerance
        && min1.y <= max2.y + tolerance
        && max1.y >= min2.y - tolerance
        && min1.z <= max2.z + tolerance
        && max1.z >= min2.z - tolerance
}
