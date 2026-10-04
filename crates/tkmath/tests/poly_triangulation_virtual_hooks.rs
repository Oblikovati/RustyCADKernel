//! Supplemental Rust analogue for native `Poly_Triangulation` virtual hooks.
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Axis1, Direction3, Point3, Transform3},
    poly::{Triangulation, TriangulationHooks, mesh_purpose},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct ExpandedBounds(Arc<AtomicUsize>);

impl TriangulationHooks for ExpandedBounds {
    fn compute_bounding_box(
        &self,
        base: &Triangulation,
        transform: Transform3,
    ) -> Option<BoundingBox3> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let mut bounds = base.compute_bounding_box_base(transform);
        bounds.enlarge(2.0);
        Some(bounds)
    }
}

fn degenerate_triangle_mesh(hooks: Box<dyn TriangulationHooks>) -> Triangulation {
    let mesh = Triangulation::with_hooks(hooks);
    mesh.resize_nodes(3, false).unwrap();
    for index in 1..=3 {
        mesh.set_node(index, Point3::new(1.0, 2.0, 3.0)).unwrap();
    }
    mesh.resize_triangles(1, false).unwrap();
    mesh.set_triangle(1, rustycad_tkmath::poly::Triangle::new(1, 2, 3))
        .unwrap();
    mesh
}

fn box_values(bounds: &BoundingBox3) -> [f64; 6] {
    bounds.get().unwrap()
}

#[test]
fn virtual_bounds_dispatch_and_base_qualified_delegation() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mesh = degenerate_triangle_mesh(Box::new(ExpandedBounds(calls.clone())));
    assert_eq!(
        box_values(&mesh.compute_bounding_box_base(Transform3::default())),
        [1.0, 2.0, 3.0, 1.0, 2.0, 3.0]
    );
    assert_eq!(
        box_values(&mesh.compute_bounding_box(Transform3::default())),
        [-1.0, 0.0, 1.0, 3.0, 4.0, 5.0]
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_dispatch_and_cached_path(&mesh, &calls);
    assert_copy_is_base_object(&mesh, &calls);
}

fn assert_dispatch_and_cached_path(mesh: &Triangulation, calls: &AtomicUsize) {
    let mut exact = BoundingBox3::default();
    assert!(mesh.min_max(&mut exact, Transform3::default(), true));
    assert_eq!(box_values(&exact), [-1.0, 0.0, 1.0, 3.0, 4.0, 5.0]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    mesh.set_cached_min_max(&BoundingBox3::new(
        Point3::new(-5.0, -5.0, -5.0),
        Point3::new(5.0, 5.0, 5.0),
    ));
    let mut cached = BoundingBox3::default();
    assert!(mesh.min_max(&mut cached, Transform3::default(), false));
    assert_eq!(box_values(&cached), [-5.0, -5.0, -5.0, 5.0, 5.0, 5.0]);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    let rotation = Transform3::rotation(Axis1::new(Point3::default(), Direction3::Z), 0.5).unwrap();
    assert!(mesh.min_max(&mut BoundingBox3::default(), rotation, true));
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

fn assert_copy_is_base_object(mesh: &Triangulation, calls: &AtomicUsize) {
    let copy = mesh.copy();
    let rotation = Transform3::rotation(Axis1::new(Point3::default(), Direction3::Z), 0.5).unwrap();
    assert_eq!(
        box_values(&copy.compute_bounding_box(rotation)),
        [
            -0.08126851531803325,
            2.2345906623849485,
            3.0,
            -0.08126851531803325,
            2.2345906623849485,
            3.0,
        ]
    );
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

struct EntityFactory;

impl TriangulationHooks for EntityFactory {
    fn nb_deferred_triangles(&self) -> i32 {
        1
    }
    fn create_new_entity(&self) -> Handle<Triangulation> {
        let calls = Arc::new(AtomicUsize::new(0));
        let mesh = Handle::new(Triangulation::with_hooks(Box::new(ExpandedBounds(calls))));
        mesh.resize_nodes(1, false).unwrap();
        mesh.set_node(1, Point3::new(4.0, 5.0, 6.0)).unwrap();
        mesh
    }
    fn load_deferred_data(&self, _destination: &Triangulation) -> bool {
        true
    }
}

#[test]
fn detached_factory_can_return_an_object_with_its_own_hooks() {
    let source = Triangulation::with_hooks(Box::new(EntityFactory));
    let detached = source.detached_load_deferred_data();
    assert!(!detached.is_null());
    assert_eq!(detached.mesh_purpose(), mesh_purpose::LOADED);
    let bounds = detached.compute_bounding_box(Transform3::default());
    assert_eq!(bounds.get().unwrap(), [2.0, 3.0, 4.0, 6.0, 7.0, 8.0]);
}
