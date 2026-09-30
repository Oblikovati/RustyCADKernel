use super::super::{flag, int, real};
use super::elements::report_vertex;
use rustycad_tkernel::{
    ncollection::{CellAction, Shared},
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkmath::gp::Xy;
use rustycad_tkmesh::brep_mesh::{BRepMeshVertex, BRepMeshVertexInspector, DegreeOfFreedom};
pub(super) fn sample_vertex(index: i32) -> BRepMeshVertex {
    let xy = [
        [0., 0.],
        [0.05, 0.],
        [-0.05, 0.],
        [0., 0.05],
        [0., -0.05],
        [0.05, 0.05],
    ][index as usize];
    BRepMeshVertex::new(Xy::new(xy[0], xy[1]), index + 1, DegreeOfFreedom::Free)
}
pub(super) fn query_point(index: i32) -> Xy {
    let xy = [
        [0., 0.],
        [0.025, 0.],
        [0.1, 0.1],
        [0.0002, 0.],
        [0., 0.0002],
        [-0.05, 0.],
    ][index as usize];
    Xy::new(xy[0], xy[1])
}
fn inspector_tolerances(inspector: &mut BRepMeshVertexInspector, first: i32, mode: i32) {
    let tolerances = [1e-7, 0.1, 0.001, 0.0001, 0., -0.2, f64::NAN];
    if mode == 0 {
        inspector.set_tolerance(tolerances[first as usize]);
    } else {
        inspector.set_tolerances(
            tolerances[first as usize],
            tolerances[(first as usize + 1) % 7],
        );
    }
}
pub fn inspector_case(v: &[i32]) {
    let mut inspector = BRepMeshVertexInspector::new(&Handle::new(Default::default()));
    int(inspector.get_coincident_point());
    for index in 0..6 {
        int(inspector.add(&sample_vertex(index)).unwrap());
    }
    inspector_tolerances(&mut inspector, v[0], v[1]);
    prepare_deletions(&mut inspector, v[4]);
    inspector.set_point(query_point(v[2]));
    inspect_candidates(&mut inspector, v[3]);
    report_inspector(&inspector);
    int(inspector.add(&sample_vertex(1)).unwrap());
    int(inspector.add(&sample_vertex(2)).unwrap());
    report_inspector(&inspector);
    clear_and_replace(&mut inspector);
    println!();
}
fn inspect_candidates(inspector: &mut BRepMeshVertexInspector, reverse: i32) {
    for index in 1..=6 {
        let index = if reverse == 0 { index } else { 7 - index };
        int(if inspector.inspect(index).unwrap() == CellAction::Keep {
            0
        } else {
            1
        });
        int(inspector.get_coincident_point());
    }
}
fn prepare_deletions(inspector: &mut BRepMeshVertexInspector, mode: i32) {
    if mode == 1 {
        inspector.delete(3).unwrap();
        inspector.delete(3).unwrap();
    }
    if mode == 2 {
        let mut vertex = inspector.get_vertex(3).unwrap();
        vertex.set_movability(DegreeOfFreedom::Deleted);
        inspector.set_vertex(3, vertex).unwrap();
    }
}
fn report_inspector(inspector: &BRepMeshVertexInspector) {
    int(inspector.nb_vertices());
    int(inspector.get_coincident_point());
    let deleted = inspector.get_list_of_del_points();
    int(deleted.read().extent());
    for index in deleted.read().iter() {
        int(*index);
    }
    for index in 1..=inspector.nb_vertices() {
        report_vertex(&inspector.get_vertex(index).unwrap());
    }
}
fn clear_and_replace(inspector: &mut BRepMeshVertexInspector) {
    let old = inspector.vertices();
    inspector.clear();
    int(old.read().length());
    int(inspector.get_coincident_point());
    let replacement = Handle::new(Shared::new(SharedState::default()));
    *inspector.change_vertices().write() = replacement.clone();
    int(inspector.add(&sample_vertex(0)).unwrap());
    flag(inspector.vertices() == replacement);
    int(old.read().length());
    report_static_geometry();
}
fn report_static_geometry() {
    for index in [-1, 0, 1, 2] {
        match BRepMeshVertexInspector::coord(index, &Xy::new(2., 3.)) {
            Ok(value) => real(value),
            Err(error) => print!("{} ", error.exception_type()),
        }
    }
    let shifted = BRepMeshVertexInspector::shift(&Xy::new(2., 3.), 0.5);
    real(shifted.x());
    real(shifted.y());
    flag(BRepMeshVertexInspector::is_equal(1, 1));
    flag(BRepMeshVertexInspector::is_equal(1, 2));
}
pub(super) fn report_index(result: Result<i32, Failure>) {
    match result {
        Ok(value) => {
            print!("OK ");
            int(value);
        }
        Err(error) => print!("{} ", error.exception_type()),
    }
}
pub(super) fn report_unit(result: Result<(), Failure>) {
    match result {
        Ok(()) => print!("OK "),
        Err(error) => print!("{} ", error.exception_type()),
    }
}

pub fn invalid_index_case(v: &[i32]) {
    let mut inspector = BRepMeshVertexInspector::new(&Handle::default());
    inspector.add(&sample_vertex(0)).unwrap();
    let result = match v[1] {
        0 => inspector
            .get_vertex(v[0])
            .map(|vertex| vertex.location_3d()),
        1 => inspector.delete(v[0]).map(|()| 0),
        _ => inspector
            .inspect(v[0])
            .map(|action| if action == CellAction::Keep { 0 } else { 1 }),
    };
    report_index(result);
    println!();
}
