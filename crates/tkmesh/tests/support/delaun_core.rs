//! Populated Delaunay fixtures with independent edge constraints and signed triangles.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun as Mesh, BRepMeshEdge as Edge, BRepMeshTriangle as Triangle,
    BRepMeshVertex as Vertex, DegreeOfFreedom as Freedom,
};
pub fn coordinates(kind: i32) -> [Xy; 4] {
    let mut points = [
        Xy::new(0., 0.),
        Xy::new(2., 0.),
        Xy::new(2., 2.),
        Xy::new(0., 2.),
    ];
    match kind {
        1 => points.swap(1, 3),
        2 => points[1] = Xy::new(1e-10, 0.),
        3 => {
            points[2] = Xy::new(4., 0.);
            points[3] = Xy::new(6., 0.);
        }
        4 => {
            for point in &mut points {
                *point += Xy::new(1e9, -1e9);
            }
        }
        5 => points[2] = Xy::new(0.5, 0.5),
        _ => (),
    }
    points
}
pub fn mesh(kind: i32, flags: i32) -> Handle<Mesh> {
    let mesh = Handle::new(Mesh::new(&Handle::new(Default::default()), 10));
    for point in coordinates(kind) {
        mesh.add_node(&Vertex::new(point, 0, Freedom::Free), true)
            .unwrap();
    }
    for (i, (a, b)) in [(1, 2), (2, 3), (3, 4), (4, 1), (1, 3)]
        .into_iter()
        .enumerate()
    {
        let freedom = if flags & (1 << i) == 0 {
            Freedom::Free
        } else if i % 2 == 0 {
            Freedom::Frontier
        } else {
            Freedom::Fixed
        };
        mesh.add_link(&Edge::new(a, b, freedom)).unwrap();
    }
    mesh.add_element(&Triangle::new(
        [1, 2, 5],
        [true, true, false],
        Freedom::Free,
    ))
    .unwrap();
    mesh.add_element(&Triangle::new([5, 3, 4], [true, true, true], Freedom::Free))
        .unwrap();
    mesh
}
pub fn polygon_mesh(kind: i32, frontier: bool) -> Handle<Mesh> {
    let mesh = Handle::new(Mesh::new(&Handle::new(Default::default()), 8));
    let mut points = coordinates(0);
    if kind == 1 {
        points[2] = Xy::new(1., 1.);
    }
    for point in points {
        mesh.add_node(&Vertex::new(point, 0, Freedom::Free), true)
            .unwrap();
    }
    let freedom = if frontier {
        Freedom::Frontier
    } else {
        Freedom::Free
    };
    for (first, last) in [(1, 2), (2, 3), (3, 4), (4, 1)] {
        mesh.add_link(&Edge::new(first, last, freedom)).unwrap();
    }
    mesh
}
