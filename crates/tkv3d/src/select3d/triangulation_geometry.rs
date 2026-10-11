//! Geometry extraction retains native triangle/side order and centroid arithmetic.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkmath::poly::{Connect, Triangulation};
/// The node of a triangulation; invalid topology must retain the native checked failure.
pub(super) fn node(triangulation: &Triangulation, index: i32) -> Result<Point3, Failure> {
    // OCCT 3d097a0 checks aliased node storage; an origin fallback invents selectable geometry.
    triangulation.node(index)
}

/// The three nodes of a one-based triangle.
pub(super) fn triangle_nodes(
    triangulation: &Triangulation,
    index: i32,
) -> Result<[Point3; 3], Failure> {
    let (n1, n2, n3) = triangulation.triangle(index)?.get();
    Ok([
        node(triangulation, n1)?,
        node(triangulation, n2)?,
        node(triangulation, n3)?,
    ])
}

/// `NbOfFreeEdges` and the free edges: the node pairs of the triangle sides without
/// neighbour (one-based array natively), and the sum of the triangle centers.
pub(super) fn free_edges(
    triangulation: &Handle<Triangulation>,
) -> Result<(Vec<i32>, Xyz), Failure> {
    let Some(trg) = triangulation.get() else {
        return Ok((Vec::new(), Xyz::new(0.0, 0.0, 0.0)));
    };
    let connect = Connect::new(triangulation)?;
    let (mut edges, mut center) = (Vec::new(), Xyz::new(0.0, 0.0, 0.0));
    for index in 1..=trg.nb_triangles() {
        let adjacent = connect.triangles(index)?;
        let nodes = trg.triangle(index)?.get();
        let [p0, p1, p2] = triangle_nodes(trg, index)?;
        center += (p0.xyz() + p1.xyz() + p2.xyz()) / 3.0;
        append_unshared_edges(&mut edges, adjacent, nodes);
    }
    Ok((edges, center))
}

/// The sum of the triangle centers.
pub(super) fn triangle_centers(trg: &Triangulation) -> Result<Xyz, Failure> {
    let mut center = Xyz::new(0.0, 0.0, 0.0);
    for index in 1..=trg.nb_triangles() {
        let [p0, p1, p2] = triangle_nodes(trg, index)?;
        center += (p0.xyz() + p1.xyz() + p2.xyz()) / 3.0;
    }
    Ok(center)
}

fn append_unshared_edges(edges: &mut Vec<i32>, adjacent: (i32, i32, i32), nodes: (i32, i32, i32)) {
    let (adjacent, nodes) = (
        [adjacent.0, adjacent.1, adjacent.2],
        [nodes.0, nodes.1, nodes.2],
    );
    for vert in 0..3 {
        if adjacent[vert] == 0 {
            edges.extend([nodes[vert], nodes[(vert + 1) % 3]]);
        }
    }
}
