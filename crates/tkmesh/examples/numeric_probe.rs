//! TKMesh foundation oracle twin; each request is independent.
use rustycad_tkernel::{ncollection::IncrementalAllocatorObject, standard::Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmesh::{brep_mesh_data::*, imesh_data::*};
use std::io::{self, BufRead};
mod mesh_probe;
fn real(value: f64) {
    print!("{} ", value.to_bits());
}
fn int(value: i32) {
    print!("{value} ");
}
fn flag(value: bool) {
    int(i32::from(value));
}
fn point3(value: Point3) {
    real(value.x());
    real(value.y());
    real(value.z());
}
fn point2(value: Point2) {
    real(value.x());
    real(value.y());
}
fn orientation(value: i32) -> Orientation {
    [
        Orientation::Forward,
        Orientation::Reversed,
        Orientation::Internal,
        Orientation::External,
    ][value as usize]
}
fn curve_report(curve: &dyn MeshCurve) {
    for index in 0..curve.parameters_nb() {
        point3(curve.get_point(index).unwrap());
        real(curve.get_parameter(index).unwrap());
    }
}
fn curve_case(count: i32, keep: bool) {
    let allocator = Handle::new(IncrementalAllocatorObject::default());
    let curve = Handle::new(DiscreteCurve::new(&allocator));
    let array = ParametersListArrayAdaptor::new(curve.cast());
    int(array.lower());
    int(array.upper());
    flag(rustycad_tkernel::standard::Transient::is_instance_name(
        &array,
        "Standard_Transient",
    ));
    let object = curve.get().unwrap();
    populate_curve(object, count);
    int(array.upper());
    real(array.value(0).unwrap());
    curve_report(object);
    object.remove_point(count / 2).unwrap();
    object.clear(keep);
    int(object.parameters_nb());
    curve_report(object);
    flag(curve.is_kind_name("IMeshData_ParametersList"));
    println!();
}
fn pcurve_report(curve: &dyn MeshPCurve) {
    for index in 0..curve.parameters_nb() {
        point2(curve.get_point(index).unwrap());
        real(curve.get_parameter(index).unwrap());
        int(curve.get_index(index).unwrap());
    }
}
fn pcurve_case(count: i32, keep: bool, orientation: Orientation) {
    let allocator = Handle::new(IncrementalAllocatorObject::default());
    let curve = Handle::new(DiscretePCurve::new(
        FacePtr::default(),
        orientation,
        &allocator,
    ));
    let object = curve.get().unwrap();
    populate_pcurve(object, count);
    pcurve_report(object);
    object.remove_point(count / 2).unwrap();
    object.clear(keep);
    int(object.parameters_nb());
    pcurve_report(object);
    pcurve_errors(object, count);
    flag(curve.is_kind_name("IMeshData_ParametersList"));
    println!();
}
fn dispatch(operation: &str, values: &[i32]) -> bool {
    match operation {
        "mesh_curve" => curve_case(values[0], values[1] != 0),
        "mesh_pcurve" => pcurve_case(values[0], values[1] != 0, orientation(values[2])),
        "mesh_pipeline" => mesh_probe::pipeline::pipeline_case(
            values[0],
            values[1..6].try_into().unwrap(),
            values[6] != 0,
            values[7] != 0,
        ),
        "mesh_shape_tool" => mesh_probe::shape_tool::shape_tool_case(values),
        "mesh_shape_polygons" => {
            mesh_probe::shape_tool::shape_polygons_case(values[0] != 0, values[1])
        }
        "mesh_wire_order" => mesh_probe::wire_foundation::wire_order_case(values),
        "mesh_outer_wire" => mesh_probe::wire_foundation::outer_wire_case(values),
        "mesh_model_builder" => mesh_probe::model_builder::model_builder_case(values),
        "mesh_deflection" => mesh_probe::deflection::deflection_case(values),
        "mesh_consistent" => mesh_probe::deflection::consistent_case(values),
        "mesh_post_processor" => mesh_probe::post_processor::post_processor_case(values),
        "mesh_edge_parameter" => mesh_probe::edge_parameter::edge_parameter_case(values),
        "mesh_tessellation_extractor" => {
            mesh_probe::tessellation_extractor::tessellation_extractor_case(values)
        }
        "mesh_curve_tessellator" => mesh_probe::curve_tessellator::curve_tessellator_case(values),
        "mesh_curve_refinement" => mesh_probe::curve_tessellator::curve_refinement_case(values),
        "mesh_edge_discret" => mesh_probe::edge_discret::edge_discret_case(values),
        "mesh_edge_tessellate" => mesh_probe::edge_discret::edge_tessellate_case(values),
        "mesh_pre" => mesh_probe::pre_processor::pre_case(values),
        "mesh_pre_cone" => mesh_probe::pre_processor::cone_case(values),
        "mesh_geom_intersection" => mesh_probe::geom_tool::intersection_case(values),
        "mesh_geom_surface" => mesh_probe::geom_tool::surface_case(values),
        "mesh_geom_sampling" => mesh_probe::geom_tool::sampling_case(values),
        "mesh_geom_cells" => mesh_probe::geom_tool::cells_case(values),
        "mesh_range_overflow" => mesh_probe::geom_tool::overflow_steps_case(),
        "mesh_editable_tree" => mesh_probe::editable_tree::editable_tree_case(values),
        "mesh_delaun_invalid" => mesh_probe::delaun_structure::invalid_case(values),
        "mesh_delaun_dump" => mesh_probe::delaun_structure::dump_case(values),
        "mesh_selector" => mesh_probe::delaun_selector::selector_case(values),
        "mesh_selector_invalid" => mesh_probe::delaun_selector::invalid_case(values),
        "mesh_circle_inspector" => mesh_probe::circle_tool::inspector_case(values),
        "mesh_circle_tool" => mesh_probe::circle_tool::tool_case(values),
        "mesh_circle_make" => mesh_probe::circle_tool::make_case(values),
        "mesh_delaun_null" => mesh_probe::delaun_core::null_case(values),
        "mesh_delaun_core" => mesh_probe::delaun_core::core_case(values),
        "mesh_delaun_cleanup" => mesh_probe::delaun_core::cleanup_case(values),
        "mesh_delaun_use_edge" => mesh_probe::delaun_core::use_edge_case(values[0]),
        "mesh_delaun_sort" => mesh_probe::delaun_core::sort_vertices_case(values),
        "mesh_delaun_polygon" => mesh_probe::delaun_core::polygon_case(values),
        "mesh_delaun_polygon_mesh" => mesh_probe::delaun_core::polygon_mesh_case(values),
        "mesh_delaun_super_mesh" => mesh_probe::delaun_core::super_mesh_case(values),
        "mesh_delaun_super_fan" => mesh_probe::delaun_core::super_fan_case(values),
        "mesh_circle_random" => mesh_probe::circle_tool::random_case(),
        "mesh_circle_invalid" => mesh_probe::circle_tool::invalid_case(values),
        "mesh_delaun" => mesh_probe::delaun_structure::delaun_case(values),
        "mesh_vertex_tool" => mesh_probe::vertex_tool::vertex_tool_case(values),
        "mesh_vertex_invalid" => mesh_probe::vertex_inspector::invalid_index_case(values),
        "mesh_vertex_inspector" => mesh_probe::vertex_inspector::inspector_case(values),
        "mesh_cell_failure" => mesh_probe::cell_failure::cell_failure_case(values),
        "mesh_element_vertex" => mesh_probe::elements::vertex_case(values),
        "mesh_element_edge" => mesh_probe::elements::edge_case(values),
        "mesh_element_triangle" => mesh_probe::elements::triangle_case(values),
        "mesh_element_pair" => mesh_probe::elements::pair_case(values),
        "mesh_healer" => mesh_probe::model_healer::healer_case(values),
        "mesh_face_checker" => mesh_probe::face_checker::face_checker_case(values),
        "mesh_face_segment" => mesh_probe::face_checker::face_segment_case(),
        "mesh_range" => mesh_probe::range_splitter::range_case(values),
        "mesh_edge_callback" => mesh_probe::edge_discret::edge_callback_case(),
        "mesh_tools" => mesh_probe::tools::tools_case(values[0]),
        "mesh_explorer" => mesh_probe::tools::explorer_case(values[0], values[1] != 0),
        "mesh_model" => mesh_probe::model_case(values[0], values[1], orientation(values[2])),
        _ => return false,
    }
    true
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut tokens = line.split_whitespace();
        let Some(operation) = tokens.next() else {
            continue;
        };
        let values: Vec<i32> = tokens.map(|value| value.parse().unwrap()).collect();
        if !dispatch(operation, &values) {
            eprintln!("Unknown probe operation: {operation}");
            std::process::exit(2);
        }
    }
}

fn populate_curve(object: &dyn MeshCurve, count: i32) {
    for i in 0..count {
        object.add_point(
            Point3::new(i.into(), (-i).into(), (i * 2).into()),
            f64::from(i) * 0.25,
        );
    }
    object
        .insert_point(count / 2, Point3::new(7.0, 8.0, 9.0), -1.25)
        .unwrap();
    let mut first = object.get_point(0).unwrap();
    first.set_x(4.5);
    object.set_point(0, first).unwrap();
    object.set_parameter(0, -0.0).unwrap();
}

fn populate_pcurve(object: &dyn MeshPCurve, count: i32) {
    flag(object.is_forward());
    flag(object.is_internal());
    int(object.get_orientation() as i32);
    flag(object.get_face().is_null());
    for i in 0..count {
        object.add_point(Point2::new(i.into(), (-i).into()), f64::from(i) * 0.25);
        object.set_index(i, i + 100).unwrap();
    }
    object
        .insert_point(count / 2, Point2::new(7.0, 8.0), -1.25)
        .unwrap();
    let mut first = object.get_point(0).unwrap();
    first.set_x(4.5);
    object.set_point(0, first).unwrap();
    object.set_parameter(0, -0.0).unwrap();
}

fn pcurve_errors(object: &dyn MeshPCurve, count: i32) {
    print!(
        "{} {} {} ",
        object.get_point(-1).unwrap_err().exception_type(),
        object.get_index(count + 1).unwrap_err().exception_type(),
        object.get_parameter(-1).unwrap_err().exception_type()
    );
}

#[path = "../../tkshhealing/tests/support/wire_foundation.rs"]
mod wire_fixture;
