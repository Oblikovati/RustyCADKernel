//! Behaviour of the derived-state cache service, the mesh cache and the mesh view on a
//! box graph: memoised answers and their invalidation, the Tool routing, slots,
//! freshness stamps, drivers, Ensure and Needs, and the fresh-entry copies.
mod support;
use rustycad_tkbrep::brep_graph::cache::BRepGraphCache;
use rustycad_tkbrep::brep_graph::cache_derived_state::{CacheDerivedState, compute_wire_is_closed};
use rustycad_tkbrep::brep_graph::cache_mesh::{
    CacheMesh, CacheMeshDriver, DEFAULT_DISPLAY_SLOT, DirtySet, SlotId,
};
use rustycad_tkbrep::brep_graph::cache_registry::CacheRegistry;
use rustycad_tkbrep::brep_graph::copy_remap::{MappingKind, Mode};
use rustycad_tkbrep::brep_graph::tool;
use rustycad_tkbrep::brep_graph::{
    BRepGraph, CoEdgeId, EdgeId, FaceId, NodeId, ShellId, SolidId, WireId,
};
use rustycad_tkbrep::brep_graph_inc::populate::{self, Options};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Guid, Handle, StandardTransient};
use rustycad_tkmath::poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangulation};
use std::sync::Mutex;
use support::make_box;

fn box_graph() -> BRepGraph {
    let mut graph = BRepGraph::new();
    populate::perform(&mut graph, &make_box(10.0, 20.0, 30.0), false, &Options).unwrap();
    graph
}

fn derived(graph: &BRepGraph) -> Handle<CacheDerivedState> {
    graph
        .cache_registry()
        .ensure::<CacheDerivedState>()
        .unwrap()
}

fn mesh(graph: &BRepGraph) -> Handle<CacheMesh> {
    graph.cache_registry().ensure::<CacheMesh>().unwrap()
}

fn triangulation() -> Handle<Triangulation> {
    Handle::new(Triangulation::new())
}

#[test]
fn derived_answers_are_memoised_until_cleared() {
    let graph = box_graph();
    let closed = compute_wire_is_closed(&graph, WireId::new(0));
    assert_eq!(graph.cache_registry().nb_caches(), 0);
    assert_eq!(tool::wire::is_closed(&graph, WireId::new(0)), closed);
    assert_eq!(graph.cache_registry().nb_caches(), 1);
    let cache = derived(&graph);
    cache.set_wire_is_closed(&graph, WireId::new(0), !closed);
    assert_eq!(
        cache.get_wire_is_closed(&graph, WireId::new(0)).unwrap(),
        Some(!closed)
    );
    assert_eq!(tool::wire::is_closed(&graph, WireId::new(0)), !closed);
    cache.clear();
    assert_eq!(tool::wire::is_closed(&graph, WireId::new(0)), closed);
    assert_eq!(
        cache.get_wire_is_closed(&graph, WireId::new(99)).unwrap(),
        None
    );
    assert!(cache.is_shell_closed(&graph, ShellId::new(0)).unwrap());
    assert!(!cache.is_degenerated(&graph, EdgeId::new(0)).unwrap());
    assert!(!cache.is_closed(&graph, EdgeId::new(0)).unwrap());
    assert!(cache.same_range(&graph, CoEdgeId::new(0)).unwrap());
    assert!(cache.same_parameter(&graph, CoEdgeId::new(0)).unwrap());
    assert!(!cache.same_range(&graph, CoEdgeId::new(999)).unwrap());
}

#[test]
fn a_mutation_invalidates_the_shell_closure() {
    let mut graph = box_graph();
    let cache = derived(&graph);
    assert!(cache.is_shell_closed(&graph, ShellId::new(0)).unwrap());
    let extra = make_box(1.0, 1.0, 1.0);
    let face = rustycad_tkbrep::topexp::Explorer::new(
        &extra.into(),
        rustycad_tkg3d::top_abs::ShapeEnum::Face,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    .current()
    .clone();
    graph
        .shapes_mut()
        .add_under(&face, ShellId::new(0).into())
        .unwrap();
    assert!(!cache.is_shell_closed(&graph, ShellId::new(0)).unwrap());
}

#[test]
fn a_detached_service_refuses_to_compute() {
    let graph = box_graph();
    let detached = CacheDerivedState::new();
    let error = detached.is_closed(&graph, EdgeId::new(0)).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn fresh_derived_entries_copy_to_the_target() {
    let source = box_graph();
    let cache = derived(&source);
    let (two, one) = (WireId::new(2), WireId::new(1));
    let closed = (
        compute_wire_is_closed(&source, two),
        compute_wire_is_closed(&source, one),
    );
    cache.set_wire_is_closed(&source, two, !closed.0);
    assert_eq!(
        cache.get_wire_is_closed(&source, one).unwrap(),
        Some(closed.1)
    );
    let mut target = box_graph();
    CacheRegistry::copy_fresh_caches_to_with_mapping(
        &source,
        &mut target,
        MappingKind::Identity,
        Mode::Copy,
    )
    .unwrap();
    let copied = target.cache_registry().find::<CacheDerivedState>();
    assert_eq!(
        copied.get_wire_is_closed(&target, two).unwrap(),
        Some(!closed.0)
    );
    assert_eq!(
        copied.get_wire_is_closed(&target, one).unwrap(),
        Some(closed.1)
    );
}

#[test]
fn cached_meshes_are_fresh_until_their_stamps_change() {
    let graph = box_graph();
    let (face, co_edge) = (FaceId::new(0), CoEdgeId::new(0));
    let editor = graph.mesh().editor();
    editor
        .faces()
        .set_cached_triangulation(face, &triangulation())
        .unwrap();
    let polygon = Handle::new(PolygonOnTriangulation::new(2, false).unwrap());
    editor
        .co_edges()
        .append_cached_polygon_on_tri(co_edge, &polygon)
        .unwrap();
    editor
        .co_edges()
        .set_cached_polygon2d(co_edge, &Handle::new(Polygon2D::new(2).unwrap()))
        .unwrap();
    let cache = mesh(&graph);
    let entry = cache.find_face_mesh(&graph, face).unwrap();
    assert_eq!(entry.mesh_generation, 1);
    let face_of_co_edge = graph.topo().co_edges().definition(co_edge).face_id;
    if face_of_co_edge == face {
        assert!(cache.find_co_edge_polygon_on_tri(&graph, co_edge).is_some());
        editor.faces().clear(face).unwrap();
        assert!(cache.find_co_edge_polygon_on_tri(&graph, co_edge).is_none());
    }
    assert!(cache.find_co_edge_polygon2d(&graph, co_edge).is_some());
    assert!(
        cache
            .find_face_entry_raw(DEFAULT_DISPLAY_SLOT, face)
            .is_none()
    );
    assert!(graph.mesh().cache().co_edges().has(co_edge));
    assert!(cache.change_face_mesh(FaceId::invalid(), |_| ()).is_err());
}

#[test]
fn the_effective_view_falls_back_to_nothing_without_meshes() {
    let graph = box_graph();
    let view = graph.mesh();
    assert!(!view.persistent().faces().has(FaceId::new(0)));
    assert!(!view.effective().faces().has(FaceId::new(0)));
    let tri = triangulation();
    view.editor()
        .faces()
        .set_cached_triangulation(FaceId::new(0), &tri)
        .unwrap();
    assert!(view.effective().faces().triangulation(FaceId::new(0)) == tri);
    let polygon = Handle::new(Polygon3D::new(2, false).unwrap());
    view.editor()
        .edges()
        .set_cached_polygon3d(EdgeId::new(1), &polygon)
        .unwrap();
    assert!(view.effective().edges().polygon3d(EdgeId::new(1)) == polygon);
    view.editor().edges().clear(EdgeId::new(1));
    assert!(!view.cache().edges().has(EdgeId::new(1)));
    assert_eq!(view.poly().nb_face_triangulations(), 0);
}

/// A driver that fills every dirty face with one triangulation and logs the calls.
struct FillingDriver {
    transient: StandardTransient,
    recipe: u64,
    calls: Mutex<Vec<(SlotId, usize)>>,
}
impl_transient!(FillingDriver, "FillingDriver", dyn CacheMeshDriver, transient,
    interfaces: [dyn CacheMeshDriver]);

impl CacheMeshDriver for FillingDriver {
    fn id(&self) -> Guid {
        Guid::from_components(7, 7, 7, 7, 7, 7, 7, 7, 7, 7)
    }
    fn recipe_hash(&self) -> u64 {
        self.recipe
    }
    fn fill(
        &self,
        graph: &mut BRepGraph,
        slot: SlotId,
        dirty: &DirtySet,
        _range: &ProgressRange<'_>,
    ) -> Result<bool, Failure> {
        self.calls.lock().unwrap().push((slot, dirty.faces.len()));
        for face in &dirty.faces {
            graph
                .mesh()
                .editor()
                .faces()
                .set_cached_triangulation(*face, &triangulation())?;
        }
        Ok(true)
    }
}

fn driver(recipe: u64) -> Handle<FillingDriver> {
    Handle::new(FillingDriver {
        transient: StandardTransient::default(),
        recipe,
        calls: Mutex::new(Vec::new()),
    })
}

#[test]
fn a_driver_refills_the_dirty_faces_once() {
    let mut graph = box_graph();
    let cache = mesh(&graph);
    let range = ProgressRange::default();
    assert!(!cache.ensure(&mut graph, 0, &range).unwrap());
    let first = driver(4);
    cache.register_driver(0, &first.cast::<dyn CacheMeshDriver>());
    let solid = NodeId::from(SolidId::new(0));
    assert!(cache.needs(&graph, solid, 0).unwrap());
    assert!(cache.ensure(&mut graph, 0, &range).unwrap());
    assert!(cache.ensure(&mut graph, 0, &range).unwrap());
    assert!(!cache.needs(&graph, solid, 0).unwrap());
    assert_eq!(*first.calls.lock().unwrap(), vec![(0, 6)]);
    let generation = cache.state(0).generation;
    cache.register_driver(0, &driver(5).cast::<dyn CacheMeshDriver>());
    assert_eq!(cache.state(0).generation, generation + 1);
    assert!(cache.needs(&graph, solid, 0).unwrap());
    cache.unregister_driver(0);
    assert!(!cache.state(0).has_driver);
}

#[test]
fn fresh_meshes_copy_with_their_slots() {
    let source = box_graph();
    let cache = mesh(&source);
    cache.register_driver(1, &driver(9).cast::<dyn CacheMeshDriver>());
    cache.set_active_display_slot(1);
    source
        .mesh()
        .editor()
        .faces()
        .set_cached_triangulation(FaceId::new(3), &triangulation())
        .unwrap();
    let mut target = box_graph();
    CacheRegistry::copy_fresh_caches_to_with_mapping(
        &source,
        &mut target,
        MappingKind::Identity,
        Mode::Copy,
    )
    .unwrap();
    let copied = target.cache_registry().find::<CacheMesh>();
    assert_eq!(copied.active_display_slot(), 1);
    assert_eq!(copied.state(1).recipe_hash, 9);
    let entry = copied.find_face_mesh(&target, FaceId::new(3)).unwrap();
    assert_eq!(entry.mesh_generation, 1);
    assert!(copied.find_face_mesh(&target, FaceId::new(2)).is_none());
}
