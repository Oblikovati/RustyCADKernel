//! Exact native element values, non-reflexive deletion and specialized hashes.
use super::super::{flag, int, real};
use rustycad_tkernel::ncollection::HashKey;
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::{
    BRepMeshEdge, BRepMeshOrientedEdge, BRepMeshTriangle, BRepMeshVertex, DegreeOfFreedom,
    PairOfIndex,
};
fn freedom(index: i32) -> DegreeOfFreedom {
    [
        DegreeOfFreedom::Free,
        DegreeOfFreedom::InVolume,
        DegreeOfFreedom::OnSurface,
        DegreeOfFreedom::OnCurve,
        DegreeOfFreedom::Fixed,
        DegreeOfFreedom::Frontier,
        DegreeOfFreedom::Deleted,
    ][index as usize]
}
fn vertex_coords(index: i32) -> Xy {
    let (u, v) = [
        (0., 0.),
        (1., 2.),
        (-0.000001, 3.),
        (1e-8, -1e-8),
        (-1., -2.),
        (65536., -32768.),
        (f64::INFINITY, 1.),
        (f64::NAN, 2.),
    ][index as usize];
    Xy::new(u, v)
}
pub fn vertex_case(v: &[i32]) {
    let mut vertex = BRepMeshVertex::default();
    report_vertex(&vertex);
    vertex.initialize(vertex_coords(v[0]), v[1], freedom(v[2]));
    report_vertex(&vertex);
    let other = BRepMeshVertex::from_uv(
        vertex.coord().x() + [0., 5e-10, 1e-9, 2e-9][v[4] as usize],
        vertex.coord().y(),
        freedom(v[3]),
    );
    flag(vertex.is_equal(&other));
    flag(vertex == other);
    flag(vertex.is_equal(&vertex));
    vertex.set_movability(freedom(v[3]));
    vertex.change_coord().set_y(-7.5);
    report_vertex(&vertex);
    println!();
}
pub(super) fn report_vertex(vertex: &BRepMeshVertex) {
    real(vertex.coord().x());
    real(vertex.coord().y());
    int(vertex.location_3d());
    int(vertex.movability() as i32);
    print!("{} ", vertex.occt_hash());
}
pub fn edge_case(v: &[i32]) {
    let nodes = [-1, 0, 1, 2, 65535, 65536, 131073, i32::MIN, i32::MAX];
    let first = nodes[v[0] as usize];
    let last = nodes[v[1] as usize];
    report_oriented_edge(first, last);
    let mut edge = BRepMeshEdge::new(first, last, freedom(v[2]));
    let other = BRepMeshEdge::new(last, first, freedom(v[3]));
    report_edge(&edge, &other);
    edge.set_movability(freedom(v[3]));
    report_edge(&edge, &edge);
    report_edge(&BRepMeshEdge::default(), &other);
    println!();
}
fn report_oriented_edge(first: i32, last: i32) {
    let oriented = BRepMeshOrientedEdge::new(first, last);
    let reverse = BRepMeshOrientedEdge::new(last, first);
    int(oriented.first_node());
    int(oriented.last_node());
    flag(oriented.is_equal(&reverse));
    flag(oriented == reverse);
    print!(
        "{} {} ",
        oriented.occt_hash(),
        BRepMeshOrientedEdge::default().occt_hash()
    );
}
fn report_edge(edge: &BRepMeshEdge, other: &BRepMeshEdge) {
    int(edge.movability() as i32);
    flag(edge.is_same_orientation(other));
    flag(edge.is_equal(other));
    flag(edge == other);
    print!("{} ", edge.occt_hash());
}
pub fn triangle_case(v: &[i32]) {
    let orientation = [v[2] & 1 != 0, v[2] & 2 != 0, v[2] & 4 != 0];
    let mut triangle = BRepMeshTriangle::default();
    report_triangle(&triangle);
    triangle.initialize(triangle_order(v[0]), orientation, freedom(v[3]));
    let other = opposite_triangle(triangle_order(v[1]), orientation, freedom(v[4]));
    report_triangle(&triangle);
    flag(triangle.is_equal(&other));
    flag(triangle == other);
    triangle.set_movability(freedom(v[4]));
    flag(triangle.is_equal(&triangle));
    triangle.edges[0] = -7;
    triangle.orientations[1] = true;
    triangle.movability = DegreeOfFreedom::Fixed;
    report_triangle(&triangle);
    println!();
}
fn opposite_triangle(
    edges: [i32; 3],
    orientation: [bool; 3],
    mobility: DegreeOfFreedom,
) -> BRepMeshTriangle {
    BRepMeshTriangle::new(edges, orientation.map(|flag| !flag), mobility)
}
fn triangle_order(index: i32) -> [i32; 3] {
    let orders = [
        [1, 2, 3],
        [2, 3, 1],
        [3, 1, 2],
        [3, 2, 1],
        [1, 1, 2],
        [-1, 65536, i32::MIN],
    ];
    orders[index as usize]
}
fn report_triangle(triangle: &BRepMeshTriangle) {
    let (edges, orientations) = triangle.edges();
    for edge in edges {
        int(edge);
    }
    for orientation in orientations {
        flag(orientation);
    }
    int(triangle.movability() as i32);
    print!("{} ", triangle.occt_hash());
}
pub fn pair_case(v: &[i32]) {
    let mut pair = PairOfIndex::default();
    report_pair(&pair);
    for step in 0..3 {
        let code = (v[0] >> (step * 3)) & 7;
        let value = v[1] + step;
        let result = pair_operation(&mut pair, code, value, v[2]);
        match result {
            Ok(()) => print!("OK "),
            Err(error) => print!("{} ", error.exception_type()),
        }
        report_pair(&pair);
    }
    report_pair_errors(&mut pair, v[2]);
    println!();
}
fn pair_operation(
    pair: &mut PairOfIndex,
    code: i32,
    value: i32,
    position: i32,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    match code {
        0 => pair.append(value),
        1 => pair.prepend(value),
        2 => pair.set_index(1, value),
        3 => pair.set_index(2, value),
        4 => pair.remove_index(1),
        5 => pair.remove_index(2),
        6 => {
            pair.clear();
            Ok(())
        }
        _ => pair.set_index(position, value),
    }
}
fn report_pair(pair: &PairOfIndex) {
    flag(pair.is_empty());
    int(pair.extent());
    int(pair.first_index());
    int(pair.last_index());
    int(pair.index(1).unwrap());
    int(pair.index(2).unwrap());
}
fn report_pair_errors(pair: &mut PairOfIndex, position: i32) {
    match pair.index(position) {
        Ok(value) => {
            print!("OK ");
            int(value);
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
    match pair.remove_index(position) {
        Ok(()) => print!("OK "),
        Err(error) => print!("{} ", error.exception_type()),
    }
    report_pair(pair);
}
