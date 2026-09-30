//! Stored free-edge polygons and model-state clearing for the edge-stage oracle.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{gp::Point3, poly::Polygon3D};
use rustycad_tkmesh::{brep_mesh::ShapeTool, imesh_data::MeshModel};
pub fn seed_free_polygons(model: &Handle<dyn MeshModel>, kind: i32) {
    if model.is_null() || kind == 0 {
        return;
    }
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        if !edge.is_free() {
            continue;
        }
        let polygon = Handle::new(Polygon3D::new(3, kind != 2).unwrap());
        polygon.change_nodes(|nodes| {
            for index in 1..=3 {
                nodes[index] = Point3::new(f64::from(index), 0.0, 0.0);
            }
        });
        if kind != 2 {
            polygon
                .change_parameters(|values| {
                    for index in 1..=3 {
                        values[index] = f64::from(index) - 1.0;
                    }
                })
                .unwrap();
        }
        polygon.set_deflection(if kind == 3 { 100.0 } else { 0.01 });
        ShapeTool::update_edge_polygon3d(&edge.get_edge().unwrap(), &polygon).unwrap();
    }
}
pub fn clear_model_curves(model: &Handle<dyn MeshModel>) {
    for index in 0..model.edges_nb() {
        let edge = model.get_edge(index).unwrap();
        edge.clear(false).unwrap();
    }
}
