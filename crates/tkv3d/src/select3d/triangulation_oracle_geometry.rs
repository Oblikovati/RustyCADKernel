//! Rust-only geometry mirrors the committed pinned native oracle inputs.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3, Vector3};
use rustycad_tkmath::poly::{Triangle, Triangulation};
use rustycad_tkmath::toploc::Location;

pub(super) fn oracle_mesh(kind: i32) -> Handle<Triangulation> {
    let mesh = if kind == 3 || kind == 4 {
        Triangulation::default()
    } else {
        sized_oracle_mesh(kind)
    };
    if kind == 3 || kind == 5 {
        cache_oracle_bounds(&mesh, kind);
    }
    Handle::new(mesh)
}
fn sized_oracle_mesh(kind: i32) -> Triangulation {
    let (nodes, triangles) = if kind == 0 {
        (36, 50)
    } else {
        (4, if kind == 2 { 4 } else { 2 })
    };
    let mesh = Triangulation::with_sizes(nodes, triangles, false, false).unwrap();
    for index in 1..=nodes {
        mesh.set_node(index, oracle_node(kind, index)).unwrap();
    }
    for index in 1..=triangles {
        mesh.set_triangle(index, oracle_triangle(kind, index))
            .unwrap();
    }
    mesh
}
fn oracle_node(kind: i32, index: i32) -> Point3 {
    if kind == 0 {
        return Point3::new(
            f64::from((index - 1) % 6 - 3),
            f64::from((index - 1) / 6 - 3),
            -f64::from((index - 1) % 4) * 0.25,
        );
    }
    Point3::new(
        if index == 2 {
            3.0
        } else if index == 3 {
            1.0
        } else {
            0.0
        },
        if index >= 3 { 2.0 } else { 0.0 },
        if index == 4 && kind == 2 { 1.0 } else { 0.0 },
    )
}
fn oracle_triangle(kind: i32, index: i32) -> Triangle {
    if kind == 0 {
        let cell = (index - 1) / 2;
        let origin = (cell / 5) * 6 + cell % 5 + 1;
        return if index % 2 != 0 {
            Triangle::new(origin, origin + 1, origin + 6)
        } else {
            Triangle::new(origin + 1, origin + 7, origin + 6)
        };
    }
    let vertices = if kind == 2 {
        [(1, 2, 3), (1, 4, 2), (2, 4, 3), (3, 4, 1)][(index - 1) as usize]
    } else {
        [(1, 2, 3), (1, 3, 4)][(index - 1) as usize]
    };
    Triangle::new(vertices.0, vertices.1, vertices.2)
}
fn cache_oracle_bounds(mesh: &Triangulation, kind: i32) {
    let mut bounds = BoundingBox3::default();
    let (min, max) = if kind == 3 {
        (Point3::new(-2., -1., -0.5), Point3::new(2., 3., 0.5))
    } else {
        (Point3::new(-4., -3., -2.), Point3::new(5., 6., 2.))
    };
    bounds.add_point(min);
    bounds.add_point(max);
    mesh.set_cached_min_max(&bounds);
}
pub(super) fn oracle_location(kind: i32) -> Location {
    let mut transform = Transform3::default();
    if kind == 1 {
        transform.set_translation(Vector3::new(2., -1., 0.5));
    }
    if kind == 2 {
        transform
            .set_rotation(Axis1::new(Point3::default(), Direction3::Z), 0.7)
            .unwrap();
    }
    if kind == 0 {
        return Location::default();
    }
    Location::new(transform)
}
pub(super) fn oracle_edges(policy: i32, length: usize) -> Handle<HArray1<i32>> {
    if policy == 0 {
        return Handle::default();
    }
    let lower = if policy == 1 {
        1
    } else if policy == 2 {
        0
    } else {
        2
    };
    let values: Vec<i32> = [1, 2, 2, 3, 3, 4, 4, 1]
        .into_iter()
        .cycle()
        .take(length)
        .collect();
    Handle::new(HArray1::from_array(&Array1::from_shared(&values, lower)))
}
