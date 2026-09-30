use super::super::{flag, int, real};
use super::{
    elements::report_vertex,
    vertex_inspector::{query_point, report_index, report_unit, sample_vertex},
};
use rustycad_tkernel::{
    ncollection::{DynamicArray, Shared},
    standard::{Handle, SharedState},
};
use rustycad_tkmesh::brep_mesh::{BRepMeshVertexTool, DegreeOfFreedom};
pub fn vertex_tool_case(v: &[i32]) {
    let tool = Handle::new(BRepMeshVertexTool::new(&Handle::new(Default::default())));
    flag(tool.is_instance_name("BRepMesh_VertexTool"));
    flag(tool.is_kind_name("Standard_Transient"));
    report_tool(&tool);
    setup_tool(&tool, v[0], v[1]);
    for index in 0..6 {
        report_index(tool.add(&sample_vertex(index), true));
    }
    report_index(tool.add(&sample_vertex(0), v[2] != 0));
    alter_tool(&tool, v[3]);
    query_tool(&tool);
    report_tool(&tool);
    report_index(tool.add(&sample_vertex(1), true));
    report_index(tool.add(&sample_vertex(2), true));
    report_unit(tool.remove_last());
    report_tool(&tool);
    reset_tool(&tool);
    println!();
}
fn setup_tool(tool: &BRepMeshVertexTool, cells: i32, tolerance: i32) {
    if cells == 0 {
        tool.set_cell_size(0.1).unwrap();
    } else {
        tool.set_cell_sizes(if cells == 1 { 0.1 } else { -0.1 }, 0.2)
            .unwrap();
    }
    match tolerance {
        0 => tool.set_tolerance(1e-7),
        1 => tool.set_tolerance(0.04),
        2 => tool.set_tolerance(0.1),
        3 => tool.set_tolerances(0.05, 0.2),
        _ => tool.set_tolerances(-0.05, -0.2),
    }
}
fn alter_tool(tool: &BRepMeshVertexTool, mode: i32) {
    match mode {
        1 => {
            tool.delete_vertex(2).unwrap();
            tool.delete_vertex(2).unwrap();
        }
        2 => tool
            .vertices()
            .write()
            .change_value(1)
            .unwrap()
            .set_movability(DegreeOfFreedom::Deleted),
        3 => tool.substitute(2, &sample_vertex(4)).unwrap(),
        4 => replace_tool_vertices(tool),
        _ => (),
    }
}
fn replace_tool_vertices(tool: &BRepMeshVertexTool) {
    let old = tool.vertices();
    let mut replacement = DynamicArray::default();
    for index in 0..tool.extent() {
        replacement.append(sample_vertex((5 - index % 6) % 6));
    }
    *tool.change_vertices().write() = Handle::new(Shared::new(SharedState::new(replacement)));
    int(old.read().length());
    flag(old != tool.vertices());
}
fn query_tool(tool: &BRepMeshVertexTool) {
    for index in 0..6 {
        let mut vertex = sample_vertex(0);
        *vertex.change_coord() = query_point(index);
        report_index(tool.find_index(&vertex));
    }
}
fn report_tool(tool: &BRepMeshVertexTool) {
    int(tool.extent());
    flag(tool.is_empty());
    let (first, second) = tool.get_tolerance();
    real(first);
    real(second);
    for index in 1..=tool.extent() {
        report_vertex(&tool.find_key(index).unwrap());
    }
    let deleted = tool.get_list_of_del_nodes();
    int(deleted.read().extent());
    for index in deleted.read().iter() {
        int(*index);
    }
}
fn reset_tool(tool: &BRepMeshVertexTool) {
    let old = tool.vertices();
    report_unit(tool.set_cell_sizes(0.2, 0.3));
    report_tool(tool);
    int(old.read().length());
    report_index(tool.add(&sample_vertex(0), false));
    report_unit(tool.set_cell_size(0.4));
    report_tool(tool);
}
