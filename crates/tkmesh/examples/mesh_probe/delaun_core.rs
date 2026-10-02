//! Nonempty existing-mesh Delaun algorithms and private polygon primitives.
use super::super::{flag, int, real};
use super::vertex_inspector::report_unit;
use rustycad_tkernel::{
    ncollection::{DynamicArray, Map, Sequence},
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::{Point2, Xy};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun as Mesh, BRepMeshDelaun as Delaun, BRepMeshVertex as Vertex,
    DegreeOfFreedom as Freedom,
};
#[path = "../../tests/support/delaun_core.rs"]
mod fixture;
pub fn core_case(v: &[i32]) {
    report_unit(core(v));
    println!();
}
pub fn use_edge_case(index: i32) {
    let mesh = Handle::new(Mesh::new(&Handle::new(Default::default()), 0));
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    flag(delaun.use_edge(index));
    println!();
}
pub fn sort_vertices_case(values: &[i32]) {
    let mesh = Handle::new(Mesh::new(&Handle::new(Default::default()), values[0]));
    let mut indices = Vec::new();
    for point in values[1..].as_chunks::<2>().0 {
        indices.push(
            mesh.add_node(
                &Vertex::new(
                    Xy::new(f64::from(point[0]), f64::from(point[1])),
                    0,
                    Freedom::Free,
                ),
                true,
            )
            .unwrap(),
        );
    }
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false).unwrap();
    delaun.sort_vertex_indices(&mut indices).unwrap();
    for index in indices {
        int(index);
    }
    println!();
}
pub fn polygon_mesh_case(v: &[i32]) {
    print!("OK ");
    if let Err(error) = polygon_mesh(v) {
        print!("{}", error.exception_type());
    }
    println!();
}
pub fn super_mesh_case(v: &[i32]) {
    print!("OK ");
    if let Err(error) = report_super_mesh(v) {
        print!("{}", error.exception_type());
    }
    println!();
}
pub fn super_fan_case(v: &[i32]) {
    print!("OK ");
    if let Err(error) = report_super_fan(v) {
        print!("{}", error.exception_type());
    }
    println!();
}
fn report_super_fan(v: &[i32]) -> Result<(), Failure> {
    let allocator = Handle::new(Default::default());
    let mesh = Handle::new(Mesh::new(&allocator, 4));
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false)?;
    delaun.build_super_fan([0., 0., 2., 2.], Xy::new(f64::from(v[0]), f64::from(v[1])))?;
    polygon_mesh_snapshot(&delaun)
}
fn report_super_mesh(v: &[i32]) -> Result<(), Failure> {
    let allocator = Handle::new(Default::default());
    let mesh = Handle::new(Mesh::new(&allocator, 3));
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false)?;
    let bounds = [v[0], v[1], v[2], v[3]].map(f64::from);
    let (nodes, edges) = delaun.build_super_mesh(bounds)?;
    report_super_nodes(&delaun, nodes)?;
    report_super_edges(&delaun, edges)?;
    Ok(())
}
fn report_super_nodes(delaun: &Delaun, nodes: [i32; 3]) -> Result<(), Failure> {
    for index in nodes {
        let node = delaun.get_vertex(index)?;
        int(index);
        real(node.coord().x());
        real(node.coord().y());
    }
    Ok(())
}
fn report_super_edges(delaun: &Delaun, edges: [i32; 3]) -> Result<(), Failure> {
    for signed_index in edges {
        let index = signed_index.abs();
        let edge = delaun.get_edge(index)?;
        int(index);
        int(edge.first_node());
        int(edge.last_node());
        int(edge.movability() as i32);
    }
    Ok(())
}
fn polygon_mesh(v: &[i32]) -> Result<(), Failure> {
    let mesh = fixture::polygon_mesh(v[0], v[2] != 0);
    let mut delaun = Delaun::from_mesh(&mesh, -1, -1, false)?;
    let mut links = polygon_links(v[1]);
    delaun.mesh_polygon_links(&mut links)?;
    polygon_mesh_snapshot(&delaun)?;
    Ok(())
}
fn polygon_links(reverse: i32) -> Sequence<i32> {
    let mut links = Sequence::default();
    for index in 1..=4 {
        links.append(if reverse == 0 { index } else { -(5 - index) });
    }
    links
}
fn polygon_mesh_snapshot(delaun: &Delaun) -> Result<(), Failure> {
    let mesh = delaun.result();
    let mut links: Vec<i32> = mesh.links_of_domain().iter().copied().collect();
    links.sort_unstable();
    let mut triangles: Vec<i32> = mesh.elements_of_domain().iter().copied().collect();
    triangles.sort_unstable();
    report_sorted_indices(&links);
    report_sorted_indices(&triangles);
    snapshot_polygon_links(delaun, &triangles)?;
    Ok(())
}
fn report_sorted_indices(indices: &[i32]) {
    int(indices.len() as i32);
    for index in indices {
        int(*index);
    }
}
fn snapshot_polygon_links(delaun: &Delaun, triangles: &[i32]) -> Result<(), Failure> {
    let mut links: Vec<i32> = delaun.result().links_of_domain().iter().copied().collect();
    links.sort_unstable();
    for index in links {
        let edge = delaun.get_edge(index)?;
        int(index);
        int(edge.first_node());
        int(edge.last_node());
        int(edge.movability() as i32);
    }
    for index in triangles {
        let triangle = delaun.get_triangle(*index)?;
        int(*index);
        for edge in triangle.edges {
            int(edge);
        }
        for orientation in triangle.orientations {
            flag(orientation);
        }
        int(triangle.movability() as i32);
    }
    Ok(())
}
fn core(v: &[i32]) -> Result<(), Failure> {
    let mesh = fixture::mesh(v[0], v[1]);
    let mut delaun = Delaun::from_mesh(&mesh, v[2], v[3], v[4] != 0)?;
    flag(delaun.result() == &mesh);
    flag(delaun.circles().is_empty());
    flag(delaun.use_edge(-1));
    mesh_snapshot(&delaun)?;
    for triangle in 1..=2 {
        containment(&delaun, triangle, v[0]);
    }
    let mut auxiliary = DynamicArray::with_increment(3);
    for index in 1..=4 {
        if v[5] & (1 << (index - 1)) != 0 {
            auxiliary.append(index);
        }
    }
    delaun.set_aux_vertices(&auxiliary);
    auxiliary.clear(false);
    report_unit(delaun.remove_aux_elements());
    mesh_snapshot(&delaun)?;
    report_unit(delaun.init_circles_tool(v[2], v[3]));
    flag(delaun.circles().is_empty());
    Ok(())
}
fn map_values(map: Map<i32>) {
    int(map.extent() as i32);
    for index in map.iter() {
        int(*index);
    }
}
fn mesh_snapshot(delaun: &Delaun) -> Result<(), Failure> {
    map_values(delaun.frontier()?);
    map_values(delaun.internal_edges()?);
    map_values(delaun.free_edges()?);
    let mesh = delaun.result();
    map_values(mesh.elements_of_domain());
    map_values(mesh.links_of_domain());
    for index in 1..=4 {
        let vertex = delaun.get_vertex(index)?;
        real(vertex.coord().x());
        real(vertex.coord().y());
        int(vertex.movability() as i32);
    }
    for index in 1..=5 {
        let edge = delaun.get_edge(index)?;
        int(edge.first_node());
        int(edge.last_node());
        int(edge.movability() as i32);
    }
    for index in 1..=2 {
        let triangle = delaun.get_triangle(index)?;
        for edge in triangle.edges {
            int(edge);
        }
        for orientation in triangle.orientations {
            flag(orientation);
        }
        int(triangle.movability() as i32);
    }
    Ok(())
}
fn containment(delaun: &Delaun, triangle: i32, kind: i32) {
    let offset = if kind == 4 {
        Xy::new(1e9, -1e9)
    } else {
        Xy::default()
    };
    for point in [
        Xy::new(1., 1.),
        Xy::new(1., 0.),
        Xy::new(0., 0.),
        Xy::new(-1., 1.),
        Xy::new(1., 1e-10),
        Xy::new(1., 0.3),
    ] {
        for tolerance in [-1., 0., 1e-18, 0.1] {
            let mut edge = 99;
            match delaun.contains(
                triangle,
                &Vertex::new(point + offset, 0, Freedom::Free),
                tolerance,
                &mut edge,
            ) {
                Ok(value) => flag(value),
                Err(e) => print!("{} ", e.exception_type()),
            }
            int(edge);
        }
    }
}
pub fn polygon_case(v: &[i32]) {
    report_unit(polygon(v));
    println!();
}
fn polygon(v: &[i32]) -> Result<(), Failure> {
    let mesh = fixture::mesh(v[0], 0);
    let center = if v[1] == 0 {
        Xy::new(1., 1.)
    } else if v[1] == 1 {
        Xy::new(0., 0.)
    } else {
        Xy::new(-1., 1.)
    };
    let index = mesh.add_node(&Vertex::new(center, 0, Freedom::Free), true)?;
    let delaun = Delaun::from_mesh(&mesh, -1, -1, false)?;
    let mut vertices = DynamicArray::with_increment(5);
    for i in 0..v[2] {
        vertices.append(i % 4 + 1);
    }
    flag(delaun.polygon_contains_vertex(index, &vertices)?);
    polygon_area(&delaun, v[3]);
    for a in 1..=5 {
        for b in 1..=5 {
            intersection(&delaun, a, b, v[4])?;
        }
    }
    Ok(())
}
fn polygon_area(delaun: &Delaun, reverse: i32) {
    let mut polygon = Sequence::default();
    for edge in 1..=4 {
        polygon.append(if reverse == 0 { edge } else { -(5 - edge) });
    }
    for (start, end) in [(1, 4), (1, 3), (2, 4), (4, 1), (5, 6), (0, 2), (1, 5)] {
        match delaun.polygon_area(&polygon, start, end) {
            Ok(value) => real(value),
            Err(e) => print!("{} ", e.exception_type()),
        }
    }
}
fn intersection(delaun: &Delaun, a: i32, b: i32, flags: i32) -> Result<(), Failure> {
    let mut point = Point2::new(7., 8.);
    int(delaun.edge_intersection(
        &delaun.get_edge(a)?,
        &delaun.get_edge(b)?,
        flags & 1 != 0,
        flags & 2 != 0,
        &mut point,
    )? as i32);
    real(point.x());
    real(point.y());
    Ok(())
}
pub fn null_case(v: &[i32]) {
    report_unit(null_query(v[0]));
    println!();
}
fn null_query(operation: i32) -> Result<(), Failure> {
    let mesh = rustycad_tkernel::standard::Handle::default();
    let mut delaun = Delaun::from_mesh(&mesh, 1, 1, false)?;
    match operation {
        0 => Delaun::from_mesh(&mesh, 1, 1, true).map(|_| ()),
        1 => delaun.init_circles_tool(1, 1),
        2 => delaun.get_vertex(1).map(|_| ()),
        3 => delaun.get_edge(1).map(|_| ()),
        4 => delaun.get_triangle(1).map(|_| ()),
        5 => delaun.frontier().map(|_| ()),
        6 => delaun.internal_edges().map(|_| ()),
        7 => delaun.free_edges().map(|_| ()),
        _ => delaun
            .contains(1, &Vertex::default(), 0., &mut 99)
            .map(|_| ()),
    }
}
