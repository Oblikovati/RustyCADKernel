//! Supplemental Poly_Triangulation regressions: sharing, storage, ranges, arrays and late loading.
use rustycad_tkernel::{
    ncollection::{Array1, Vec3},
    standard::{Handle, dump::DumpStream},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Point2, Point3, Transform3},
    poly::{DeferredTriangulation, Triangle, Triangulation, TriangulationParameters, mesh_purpose},
};
fn square() -> Handle<Triangulation> {
    let mesh = Triangulation::with_sizes(4, 2, false, false).unwrap();
    let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    for (index, (x, y)) in corners.into_iter().enumerate() {
        mesh.set_node(index as i32 + 1, Point3::new(x, y, 0.0))
            .unwrap();
    }
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh.set_triangle(2, Triangle::new(1, 3, 4)).unwrap();
    Handle::new(mesh)
}
#[test]
fn changes_through_one_handle_are_seen_through_every_handle() {
    let first = square();
    let second = first.clone();
    first.set_deflection(0.25);
    second.set_node(2, Point3::new(2.0, 0.0, 0.0)).unwrap();
    assert_eq!(second.deflection(), 0.25);
    assert_eq!(first.node(2).unwrap(), Point3::new(2.0, 0.0, 0.0));
    assert!(first.has_geometry());
}
#[test]
fn zero_nodes_and_bad_indices_raise_native_exceptions() {
    let error = Triangulation::with_sizes(0, 1, false, false).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_RangeError");
    let mesh = square();
    assert_eq!(
        mesh.node(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        mesh.triangle(3).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        Triangulation::new()
            .add_uv_nodes()
            .unwrap_err()
            .exception_type(),
        "Standard_RangeError"
    );
    assert_eq!(
        mesh.set_double_precision(false)
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
}
#[test]
fn normals_average_unnormalized_triangle_normals() {
    let mesh = square();
    mesh.compute_normals().unwrap();
    for index in 1..=4 {
        assert_eq!(mesh.normal_vec3(index).unwrap(), Vec3::new(0.0, 0.0, 1.0));
    }
    let isolated = Triangulation::with_sizes(3, 0, false, true).unwrap();
    isolated.compute_normals().unwrap();
    assert_eq!(isolated.normal_vec3(3).unwrap(), Vec3::new(0.0, 0.0, 1.0));
    assert_eq!(mesh.map_normal_array().unwrap().len(), 12);
}
#[test]
fn cached_range_serves_axis_aligned_transforms_only() {
    let mesh = square();
    let mut exact = BoundingBox3::default();
    assert!(mesh.min_max(&mut exact, Transform3::default(), true));
    mesh.set_cached_min_max(&BoundingBox3::new(
        Point3::new(-5.0, -5.0, -5.0),
        Point3::new(5.0, 5.0, 5.0),
    ));
    let mut cached = BoundingBox3::default();
    mesh.min_max(&mut cached, Transform3::translation([1.0, 0.0, 0.0]), true);
    assert_eq!(cached.get().unwrap()[0], -4.0);
    // UpdateCachedMinMax goes through MinMax with an identity, which reuses the cache.
    mesh.update_cached_min_max();
    assert_eq!(mesh.cached_min_max().get().unwrap()[0], -5.0);
    mesh.set_cached_min_max(&BoundingBox3::default());
    mesh.update_cached_min_max();
    assert_eq!(mesh.cached_min_max().get().unwrap(), exact.get().unwrap());
    mesh.set_cached_min_max(&BoundingBox3::default());
    assert!(!mesh.has_cached_min_max());
    assert!(!Triangulation::new().min_max(
        &mut BoundingBox3::default(),
        Transform3::default(),
        false
    ));
}
#[test]
fn triangle_array_keeps_the_input_lower_bound() {
    let points = [
        Point3::default(),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let nodes = Array1::from_shared(&points, 1);
    let corners = [Triangle::new(1, 2, 3)];
    let triangles = Array1::from_shared(&corners, 0);
    let mesh = Triangulation::from_nodes(&nodes, &triangles).unwrap();
    assert_eq!(mesh.triangle(0).unwrap(), Triangle::new(1, 2, 3));
    assert!(mesh.triangle(1).is_err());
    let parameters = [Point2::default(); 2];
    let uv = Array1::from_shared(&parameters, 1);
    let error = Triangulation::from_nodes_and_uv(&nodes, &uv, &triangles)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
}
#[test]
fn copies_parameters_range_and_clears_with_precision() {
    let mesh = square();
    mesh.set_parameters(Handle::new(TriangulationParameters::new(0.1, 0.2, -1.0)));
    mesh.update_cached_min_max();
    let copy = mesh.copy();
    assert!(copy.parameters() != mesh.parameters());
    assert_eq!(copy.parameters().deflection(), 0.1);
    assert!(!copy.parameters().has_min_size());
    assert!(copy.has_cached_min_max());
    let single = Triangulation::new();
    single.set_double_precision(false).unwrap();
    single.resize_nodes(2, false).unwrap();
    single.clear();
    assert!(!single.is_double_precision());
    assert_eq!(single.nb_nodes(), 0);
}
#[test]
fn array_views_and_deprecated_members() {
    let mesh = square();
    assert_eq!(mesh.map_node_array().unwrap().upper(), 4);
    assert!(mesh.map_uv_node_array().is_none());
    assert_eq!(
        mesh.map_triangle_array().unwrap().value(2).unwrap(),
        &Triangle::new(1, 3, 4)
    );
    let wrong = Array1::from_shared(&[0.0_f32; 5], 1);
    assert_eq!(
        mesh.set_normals(Some(&wrong)).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    mesh.change_triangle(1, |t| t.set(4, 3, 2)).unwrap();
    assert_eq!(mesh.triangles().value(1).unwrap(), &Triangle::new(4, 3, 2));
    let mut json = DumpStream::default();
    mesh.dump_json(&mut json).unwrap();
    assert_eq!(
        json.text(),
        "\"className\": \"Poly_Triangulation\", \"Deflection\": 0, \"Nodes.Size\": 4, \"Triangles.Size\": 2, \"Purpose\": 0"
    );
}
/// A named fake deferred storage that loads one triangle.
struct FakeTriangleStorage;
impl DeferredTriangulation for FakeTriangleStorage {
    fn nb_deferred_nodes(&self) -> i32 {
        3
    }
    fn nb_deferred_triangles(&self) -> i32 {
        1
    }
    fn load_deferred_data(&self, destination: &Triangulation) -> bool {
        destination.resize_nodes(3, false).unwrap();
        destination.resize_triangles(1, false).unwrap();
        destination.set_triangle(1, Triangle::new(1, 2, 3)).is_ok()
    }
}
#[test]
fn deferred_storage_loads_unloads_and_detaches() {
    let mesh = Triangulation::with_deferred(Box::new(FakeTriangleStorage));
    assert!(mesh.has_deferred_data());
    assert!(mesh.load_deferred_data());
    assert_eq!(mesh.mesh_purpose(), mesh_purpose::LOADED);
    let detached = mesh.detached_load_deferred_data();
    assert_eq!(detached.nb_triangles(), 1);
    assert!(mesh.unload_deferred_data());
    assert_eq!(mesh.nb_nodes(), 0);
    assert_eq!(mesh.mesh_purpose(), mesh_purpose::NONE);
    assert!(!Triangulation::new().load_deferred_data());
}
