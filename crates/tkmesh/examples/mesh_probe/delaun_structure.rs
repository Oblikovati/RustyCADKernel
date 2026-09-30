use super::super::{flag, int};
use super::elements::report_vertex;
use super::vertex_inspector::{report_index, report_unit};
use rustycad_tkernel::{
    ncollection::Map,
    standard::{Failure, Handle},
};
use rustycad_tkmesh::brep_mesh::{
    BRepMeshDataStructureOfDelaun as Mesh, BRepMeshEdge, BRepMeshTriangle, BRepMeshVertex,
    DegreeOfFreedom as Freedom,
};
fn freedom(index: i32) -> Freedom {
    [
        Freedom::Free,
        Freedom::InVolume,
        Freedom::OnSurface,
        Freedom::OnCurve,
        Freedom::Fixed,
        Freedom::Frontier,
        Freedom::Deleted,
    ][index as usize]
}
fn vertex(index: i32, movability: Freedom) -> BRepMeshVertex {
    BRepMeshVertex::from_uv(f64::from(index), f64::from(index % 2), movability)
}
fn triangle(movability: Freedom) -> BRepMeshTriangle {
    BRepMeshTriangle::new([1, 2, 3], [true, false, true], movability)
}
pub fn delaun_case(v: &[i32]) {
    let allocator = Handle::new(Default::default());
    let mesh = Handle::new(Mesh::new(&allocator, v[0]));
    flag(mesh.allocator() == &allocator);
    flag(mesh.is_instance_name("BRepMesh_DataStructureOfDelaun"));
    flag(mesh.is_kind_name("Standard_Transient"));
    populate(&mesh, v);
    snapshot(&mesh);
    report_unit(alter(&mesh, v[4]));
    snapshot(&mesh);
    println!();
}
fn populate(mesh: &Mesh, v: &[i32]) {
    mesh.vertex_tool().set_cell_size(0.5).unwrap();
    for index in 1..=5 {
        report_index(mesh.add_node(&vertex(index, freedom(v[1])), true));
    }
    for (first, last) in [(1, 2), (2, 3), (3, 1), (3, 4), (4, 1), (5, 5)] {
        report_index(mesh.add_link(&BRepMeshEdge::new(first, last, freedom(v[2]))));
    }
    report_index(mesh.add_element(&triangle(freedom(v[3]))));
    report_index(mesh.add_element(&BRepMeshTriangle::new(
        [3, 4, 5],
        [false, true, false],
        freedom(v[3]),
    )));
}
fn alter(mesh: &Mesh, operation: i32) -> Result<(), Failure> {
    match operation {
        0 => report_index(mesh.add_link(&BRepMeshEdge::new(2, 1, Freedom::Free))),
        1 => flag(mesh.substitute_node(1, &vertex(2, Freedom::Free))?),
        2 => flag(mesh.substitute_node(1, &vertex(9, Freedom::Free))?),
        3 => mesh.remove_node(1, true)?,
        4 => mesh.remove_link(6, false)?,
        5 => mesh.remove_link(6, true)?,
        6 => flag(mesh.substitute_link(6, &BRepMeshEdge::new(2, 5, Freedom::Free))?),
        7 => flag(mesh.substitute_link(1, &BRepMeshEdge::new(4, 2, Freedom::Free))?),
        8 => mesh.remove_element(1)?,
        9 => flag(mesh.substitute_element(
            1,
            &BRepMeshTriangle::new([4, 5, 6], [false; 3], Freedom::Fixed),
        )?),
        10 => mesh.clear_domain()?,
        _ => alter_compaction(mesh, operation)?,
    }
    Ok(())
}
fn alter_compaction(mesh: &Mesh, operation: i32) -> Result<(), Failure> {
    match operation {
        11 => {
            mesh.remove_link(6, true)?;
            mesh.clear_deleted()?;
        }
        12 => {
            mesh.remove_link(6, true)?;
            mesh.remove_node(5, true)?;
            mesh.clear_deleted()?;
        }
        13 => {
            mesh.remove_element(1)?;
            mesh.remove_link(1, true)?;
            mesh.clear_deleted()?;
        }
        _ => alter_replacements(mesh, operation)?,
    }
    Ok(())
}
fn alter_replacements(mesh: &Mesh, operation: i32) -> Result<(), Failure> {
    match operation {
        20 => compact_attached(mesh)?,
        14 => {
            mesh.remove_link(6, true)?;
            report_index(mesh.add_link(&BRepMeshEdge::new(2, 5, Freedom::Free)));
        }
        15 => report_index(mesh.add_element(&triangle(Freedom::Free))),
        16 => replace_deleted_element(mesh)?,
        17 => report_index(mesh.add_link(&BRepMeshEdge::new(1, 99, Freedom::Free))),
        18 => flag(mesh.substitute_link(6, &BRepMeshEdge::new(2, 1, Freedom::Free))?),
        _ => {
            mesh.remove_link(6, true)?;
            flag(mesh.substitute_link(6, &BRepMeshEdge::new(1, 4, Freedom::Free))?);
        }
    }
    Ok(())
}
fn report_map(indices: &Map<i32>) {
    int(indices.extent() as i32);
    for index in indices.iter() {
        int(*index);
    }
}
fn snapshot(mesh: &Mesh) {
    int(mesh.nb_nodes());
    int(mesh.nb_links());
    int(mesh.nb_elements());
    report_map(&mesh.links_of_domain());
    report_map(&mesh.elements_of_domain());
    report_nodes(mesh);
    report_links(mesh);
    report_elements(mesh);
    let queue = mesh.vertex_tool().get_list_of_del_nodes();
    int(queue.read().extent());
    for index in queue.read().iter() {
        int(*index);
    }
}
fn report_nodes(mesh: &Mesh) {
    for index in 1..=mesh.nb_nodes() {
        let node = mesh.get_node(index).unwrap();
        report_vertex(&node);
        int(mesh.index_of_node(&node).unwrap());
        let links = mesh.links_connected_to(index).unwrap();
        int(links.extent());
        for link in links.iter() {
            int(*link);
        }
    }
}
fn report_links(mesh: &Mesh) {
    for index in 1..=mesh.nb_links() {
        let edge = mesh.get_link(index).unwrap();
        int(edge.first_node());
        int(edge.last_node());
        int(edge.movability() as i32);
        int(mesh.index_of_link(&edge));
        let pair = mesh.elements_connected_to(index).unwrap();
        int(pair.extent());
        int(pair.index(1).unwrap());
        int(pair.index(2).unwrap());
    }
}
fn report_elements(mesh: &Mesh) {
    for index in 1..=mesh.nb_elements() {
        let element = mesh.get_element(index).unwrap();
        for edge in element.edges {
            int(edge);
        }
        for orientation in element.orientations {
            flag(orientation);
        }
        int(element.movability as i32);
        report_element_nodes(mesh, &element);
    }
}
fn report_element_nodes(mesh: &Mesh, element: &BRepMeshTriangle) {
    match mesh.element_nodes(element) {
        Ok(nodes) => {
            print!("OK ");
            for node in nodes {
                int(node);
            }
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
}
pub fn invalid_case(v: &[i32]) {
    let mesh = Mesh::new(&Handle::new(Default::default()), 1);
    populate_invalid(&mesh, v[0]);
    let result = match v[0] {
        0 => mesh.get_node(v[1]).map(|_| ()),
        1 => mesh.get_element(v[1]).map(|_| ()),
        2 => mesh
            .substitute_element(1, &triangle(Freedom::Free))
            .map(|_| ()),
        _ => {
            mesh.remove_link(1, true).unwrap();
            mesh.clear_deleted()
        }
    };
    report_unit(result);
    println!();
}
fn populate_invalid(mesh: &Mesh, operation: i32) {
    mesh.add_node(&vertex(1, Freedom::Free), true).unwrap();
    if operation == 3 {
        for index in 2..=3 {
            mesh.add_node(&vertex(index, Freedom::Free), true).unwrap();
        }
        for (a, b) in [(1, 1), (1, 2), (2, 3), (3, 1)] {
            mesh.add_link(&BRepMeshEdge::new(a, b, Freedom::Free))
                .unwrap();
        }
    } else {
        for _ in 0..4 {
            mesh.add_link(&BRepMeshEdge::new(1, 1, Freedom::Deleted))
                .unwrap();
        }
    }
    mesh.add_element(&BRepMeshTriangle::new([2, 3, 4], [true; 3], Freedom::Free))
        .unwrap();
}
#[allow(dead_code)]
#[path = "../../../tkbrep/tests/bintools_behavior/fakes.rs"]
mod dump_filesystem;
pub fn dump_case(v: &[i32]) {
    let mesh = Mesh::new(&Handle::new(Default::default()), 1);
    for index in 1..=3 {
        mesh.add_node(&vertex(index, Freedom::Free), true).unwrap();
    }
    if v[0] > 0 {
        mesh.add_link(&BRepMeshEdge::new(
            1,
            if v[0] == 1 { 2 } else { 1 },
            Freedom::Free,
        ))
        .unwrap();
    }
    let fs = dump_filesystem::MemoryFileSystem::default();
    mesh.dump_on(&fs, "mesh.brep");
    for byte in fs.contents("mesh.brep").unwrap() {
        int(i32::from(byte));
    }
    println!();
}

fn compact_attached(mesh: &Mesh) -> Result<(), Failure> {
    mesh.remove_element(1)?;
    mesh.remove_link(1, true)?;
    mesh.add_element(&BRepMeshTriangle::new([2, 4, 6], [true; 3], Freedom::Free))?;
    mesh.add_element(&BRepMeshTriangle::new([2, 5, 6], [true; 3], Freedom::Free))?;
    mesh.remove_element(4)?;
    mesh.clear_deleted()
}

fn replace_deleted_element(mesh: &Mesh) -> Result<(), Failure> {
    mesh.remove_element(1)?;
    flag(mesh.substitute_element(1, &triangle(Freedom::Frontier))?);
    Ok(())
}
