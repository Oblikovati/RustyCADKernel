//! Native cache service and mesh migration regressions for compaction.
use super::*;
use rustycad_tkbrep::brep_graph::cache_mesh::{CacheMesh, CacheMeshDriver, DirtySet, SlotId};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkmath::poly::{Triangle, Triangulation};

struct StubMeshDriver {
    transient: StandardTransient,
    recipe: u64,
}
impl_transient!(StubMeshDriver, "StubMeshDriver", dyn CacheMeshDriver, transient,
    interfaces: [dyn CacheMeshDriver]);

impl CacheMeshDriver for StubMeshDriver {
    fn id(&self) -> Guid {
        Guid::from_components(
            0xaaaaaaaau32 as i32,
            0xbbbb,
            0xcccc,
            0xdddd,
            0xee,
            0xee,
            0xee,
            0xee,
            0xee,
            0xee,
        )
    }

    fn recipe_hash(&self) -> u64 {
        self.recipe
    }

    fn fill(
        &self,
        _: &mut BRepGraph,
        _: SlotId,
        _: &DirtySet,
        _: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        Ok(true)
    }
}

fn install_driver(graph: &BRepGraph, recipe: u64) -> Handle<CacheMesh> {
    let cache = graph.cache_registry().ensure::<CacheMesh>().unwrap();
    let driver = Handle::new(StubMeshDriver {
        transient: StandardTransient::default(),
        recipe,
    });
    cache.register_driver(0, &driver.cast::<dyn CacheMeshDriver>());
    assert!(!cache.is_null());
    cache
}

fn compact_triangle() -> Handle<Triangulation> {
    let mesh = Handle::new(Triangulation::with_sizes(3, 1, false, false).unwrap());
    mesh.set_node(1, Point3::new(0.0, 0.0, 0.0)).unwrap();
    mesh.set_node(2, Point3::new(1.0, 0.0, 0.0)).unwrap();
    mesh.set_node(3, Point3::new(0.0, 1.0, 0.0)).unwrap();
    mesh.set_triangle(1, Triangle::new(1, 2, 3)).unwrap();
    mesh
}

fn cache_face(graph: &mut BRepGraph, face: FaceId, mesh: &Handle<Triangulation>) {
    graph
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(face, mesh)
        .unwrap();
    let cache = graph.cache_registry().find::<CacheMesh>();
    assert!(cache.find_face_mesh(graph, face).is_some());
}

fn assert_driver(graph: &BRepGraph, recipe: u64) -> Handle<CacheMesh> {
    let cache = graph.cache_registry().find::<CacheMesh>();
    assert!(!cache.is_null());
    let driver = cache.driver_of(0);
    assert!(!driver.is_null());
    assert_eq!(driver.recipe_hash(), recipe);
    cache
}

#[test]
fn BRepGraph_CompactTest__CacheMesh_DriverSurvivesCompact() {
    let mut graph = compact_graph(&make_box(10.0, 20.0, 30.0));
    assert!(!graph.is_empty());
    let cache = install_driver(&graph, 42);
    let face = FaceId::start();
    assert!(face.index < graph.topo().faces().nb());
    cache_face(&mut graph, face, &compact_triangle());
    assert!(!cache.driver_of(0).is_null());
    assert!(cache.state(0).has_driver);
    assert_eq!(compact::perform(&mut graph).unwrap().nb_removed_vertices, 0);
    let after = assert_driver(&graph, 42);
    assert!(cache == after);
    assert!(after.state(0).has_driver);
    assert!(after.find_face_mesh(&graph, face).is_some());
}

fn merge_duplicates(graph: &mut BRepGraph) {
    let options = DeduplicateOptions {
        merge_entities_when_safe: true,
        ..DeduplicateOptions::default()
    };
    deduplicate::perform_with(graph, &options).unwrap();
}

fn cache_surviving_faces(graph: &mut BRepGraph, mesh: &Handle<Triangulation>) {
    assert!(graph.topo().faces().nb() > 0);
    for index in 0..graph.topo().faces().nb() {
        let face = FaceId::new(index);
        if !face.is_removed(graph) {
            cache_face(graph, face, mesh);
        }
    }
}

fn assert_compact_removals(result: compact::CompactResult) {
    assert!(result.nb_removed_vertices + result.nb_removed_edges + result.nb_removed_faces > 0);
}

#[test]
fn BRepGraph_CompactTest__CacheMesh_DriverSurvivesCompactWithDedup() {
    let mut graph = compact_graph(&two_copied_faces());
    assert!(!graph.is_empty());
    let cache = install_driver(&graph, 99);
    cache_surviving_faces(&mut graph, &compact_triangle());
    merge_duplicates(&mut graph);
    assert_compact_removals(compact::perform(&mut graph).unwrap());
    let after = assert_driver(&graph, 99);
    assert!(cache == after);
    for index in 0..graph.topo().faces().nb() {
        assert!(after.find_face_mesh(&graph, FaceId::new(index)).is_none());
    }
}

#[test]
fn BRepGraph_CompactTest__CacheMesh_CanCopyFreshEntriesThroughCompact() {
    let mut graph = compact_graph(&two_copied_faces());
    assert!(!graph.is_empty());
    let cache = install_driver(&graph, 77);
    let mesh = compact_triangle();
    merge_duplicates(&mut graph);
    cache_surviving_faces(&mut graph, &mesh);
    let options = compact::Options {
        cache_mode: compact::CachePolicy::CopyFresh,
        ..compact::Options::default()
    };
    assert_compact_removals(compact::perform_with(&mut graph, &options).unwrap());
    let after = assert_driver(&graph, 77);
    assert!(cache != after);
    assert_cached_triangles(&graph, &after, &mesh);
}

fn assert_cached_triangles(graph: &BRepGraph, cache: &CacheMesh, mesh: &Handle<Triangulation>) {
    for index in 0..graph.topo().faces().nb() {
        let entry = cache.find_face_mesh(graph, FaceId::new(index)).unwrap();
        assert!(entry.triangulation == *mesh);
    }
}
