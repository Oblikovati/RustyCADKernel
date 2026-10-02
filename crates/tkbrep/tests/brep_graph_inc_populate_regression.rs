//! Rust translation of `BRepGraphInc_PopulateRegression_Test.cxx` at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;

use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_graph::{
        compact,
        layer_topo_supplement::{AttachmentKind, LayerTopoSupplement},
        *,
    },
    brep_graph_inc::{populate, reconstruct},
    topods::{self, Builder, Compound, Face},
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg3d::{
    geom::{GeomSphericalSurface, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::{
    gp::{Axis3, Point3},
    poly::mesh_purpose,
    toploc::Location,
};

#[path = "brep_graph_inc_populate_regression/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn BRepGraphInc_PopulateRegressionTest__LocatedFacePolygonOnTriangulation_IsCaptured() {
    let box_shape = box_shape(10.0, 20.0, 30.0, true);
    let moved = box_shape.moved(&translation(5.0, 6.0, 7.0), false).unwrap();
    let mut located_polygons = 0;
    let mut identity_polygons = 0;
    for face_shape in subshapes(&moved, ShapeEnum::Face) {
        let face = topods::Face::unchecked(face_shape);
        let (triangulation, face_location) =
            BRepTool::triangulation(&face, mesh_purpose::NONE).unwrap();
        assert!(!face_location.is_identity());
        for edge_shape in subshapes(&face.clone().into_shape(), ShapeEnum::Edge) {
            let edge = topods::Edge::unchecked(edge_shape);
            located_polygons += usize::from(
                !BRepTool::polygon_on_triangulation(&edge, &triangulation, &face_location)
                    .unwrap()
                    .is_null(),
            );
            identity_polygons += usize::from(
                !BRepTool::polygon_on_triangulation(&edge, &triangulation, &Location::default())
                    .unwrap()
                    .is_null(),
            );
        }
    }
    assert!(located_polygons > 0);
    assert_eq!(identity_polygons, 0);
    let graph = populate_shape(&moved);
    let mut found = false;
    for index in 0..graph.topo().faces().nb() {
        let triangulation = graph
            .mesh()
            .persistent()
            .faces()
            .triangulation(FaceId::new(index));
        found |= !triangulation.is_null();
    }
    assert!(found);
    let polygon_count = (0..graph.topo().co_edges().nb())
        .filter(|index| {
            graph
                .topo()
                .co_edges()
                .definition(CoEdgeId::new(*index))
                .polygon_on_tri_rep_id
                .is_valid()
        })
        .count();
    assert!(polygon_count > 0);
    assert!(!graph.is_empty());
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__NoWireNaturalSphere_DegenerateBoundaryEdgesRoundTrip() {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 10.0).unwrap()).into_cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, precision::confusion())
        .unwrap();
    BRepBuilder::default()
        .natural_restriction(&face, true)
        .unwrap();
    let graph = populate_shape(&face.clone().into_shape());
    let mut degenerated = 0;
    for index in 0..graph.topo().edges().nb() {
        let edge = EdgeId::new(index);
        if !tool::edge::degenerated(&graph, edge) {
            continue;
        }
        degenerated += 1;
        assert!(
            !graph
                .topo()
                .edges()
                .definition(edge)
                .curve3d_rep_id
                .is_valid()
        );
    }
    assert!(degenerated >= 2);
    let reconstructed = reconstruct::node(&graph, FaceId::start().into()).unwrap();
    let reconstructed_degenerated = subshapes(&reconstructed, ShapeEnum::Edge)
        .iter()
        .map(|edge| topods::Edge::unchecked(edge.clone()))
        .filter(|edge| BRepTool::degenerated(edge).unwrap())
        .count();
    assert!(reconstructed_degenerated >= 2);
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__CompoundReferencesKeepDirectSourceLocationsAtEveryLevel() {
    let (moved_root, _, _) = nested_moved_box(true);
    let mut graph = BRepGraph::new();
    let result = graph.shapes_mut().add(&moved_root).unwrap();
    assert_eq!(result.topology_root.kind, NodeKind::COMPOUND);
    assert!(result.product.is_valid());
    assert_eq!(graph.topo().compounds().nb(), 2);
    let occurrence = graph
        .topo()
        .products()
        .relations(result.product)
        .occurrence_ref_ids[0];
    assert_translation(
        &graph.refs().occurrences().entry(occurrence).local_location,
        [0.0, 0.0, 11.0],
    );
    let (inner, solid) = nested_locations(&graph, result.topology_root);
    assert_translation(&inner, [0.0, 7.0, 0.0]);
    assert_translation(&solid, [5.0, 0.0, 0.0]);
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__ShapeViewFindNode_AcceptsOriginalPlacedSubshapes() {
    let (moved, original_solid, original_face) = nested_moved_box(true);
    let mut graph = BRepGraph::new();
    let result = graph.shapes_mut().add(&moved).unwrap();
    assert!(result.is_ok());
    assert_eq!(graph.shapes().find_node(&moved), result.topology_root);
    assert_eq!(
        graph.shapes().find_node(&original_solid).kind,
        NodeKind::SOLID
    );
    assert_eq!(
        graph.shapes().find_node(&original_face).kind,
        NodeKind::FACE
    );
    assert!(result.product.is_valid());
    let product_shape = graph.shapes().shape(result.product.into()).unwrap();
    assert!(product_shape.location().is_equal(moved.location()));
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__Compact_PreservesAssemblyShapeLookupAndPlacement() {
    let (moved, original_solid, original_face) = nested_moved_box(true);
    let mut graph = BRepGraph::new();
    let result = graph.shapes_mut().add(&moved).unwrap();
    let junk = box_shape(1.0, 1.0, 1.0, false);
    let options = shapes_view::Options {
        create_auto_product: false,
        ..Default::default()
    };
    let junk_result = graph
        .shapes_mut()
        .add_with_options(&junk, &options)
        .unwrap();
    graph
        .editor()
        .generic()
        .remove_subgraph(junk_result.topology_root)
        .unwrap();
    let compacted = compact::perform(&mut graph).unwrap();
    assert!(compacted.nb_nodes_before > compacted.nb_nodes_after);
    assert_eq!(graph.shapes().find_node(&moved), result.topology_root);
    assert_eq!(
        graph.shapes().find_node(&original_solid).kind,
        NodeKind::SOLID
    );
    assert_eq!(
        graph.shapes().find_node(&original_face).kind,
        NodeKind::FACE
    );
    let (inner, solid) = nested_locations(&graph, result.topology_root);
    assert_translation(&inner, [0.0, 7.0, 0.0]);
    assert_translation(&solid, [5.0, 0.0, 0.0]);
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__Append_ReusesSameLocatedDefinitionAcrossCalls() {
    let base = box_shape(2.0, 3.0, 4.0, false);
    let first = base.moved(&translation(5.0, 0.0, 0.0), false).unwrap();
    let mut graph = BRepGraph::new();
    populate::append(&mut graph, &first, false, &populate::Options).unwrap();
    let count = graph.topo().solids().nb();
    let first_node = graph.shapes().find_node(&first);
    populate::append(&mut graph, &first, false, &populate::Options).unwrap();
    assert_eq!(graph.topo().solids().nb(), count);
    assert_eq!(graph.shapes().find_node(&first), first_node);
    let second = base.moved(&translation(9.0, 0.0, 0.0), false).unwrap();
    populate::append(&mut graph, &second, false, &populate::Options).unwrap();
    assert_eq!(graph.topo().solids().nb(), count + 1);
    assert_ne!(graph.shapes().find_node(&second), first_node);
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__PersistentTriangulationBounds_AgreeWithMovedNodes() {
    let base = box_shape(2.0, 3.0, 4.0, true);
    let moved = base.moved(&translation(25.0, -3.0, 8.0), false).unwrap();
    let graph = populate_shape(&moved);
    let mut found = false;
    for index in 0..graph.topo().faces().nb() {
        let triangulation = graph
            .mesh()
            .persistent()
            .faces()
            .triangulation(FaceId::new(index));
        if triangulation.is_null() {
            continue;
        }
        found = true;
        assert!(!triangulation.has_cached_min_max());
    }
    assert!(found);
    assert_graph_box_bounds(
        &graph,
        Point3::new(25.0, -3.0, 8.0),
        Point3::new(27.0, 0.0, 12.0),
    );
    let reconstructed = reconstruct::node(&graph, SolidId::start().into()).unwrap();
    let reconstructed_faces = subshapes(&reconstructed, ShapeEnum::Face);
    assert_eq!(reconstructed_faces.len(), 6);
    for face in reconstructed_faces {
        let face = topods::Face::unchecked(face);
        let (triangulation, _) = BRepTool::triangulation(&face, mesh_purpose::NONE).unwrap();
        assert!(!triangulation.is_null());
        assert!(!triangulation.has_cached_min_max());
    }
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__Perform_ReplacesGraphThroughClearSemantics() {
    let first = box_shape(1.0, 1.0, 1.0, false);
    let second = box_shape(2.0, 2.0, 2.0, false);
    let mut graph = populate_shape(&first);
    let layer = graph
        .layer_registry()
        .ensure::<LayerTopoSupplement>()
        .unwrap();
    let vertex = subshapes(&first, ShapeEnum::Vertex).remove(0);
    let uid = layer.get().unwrap().add_attachment(
        &graph,
        SolidId::start().into(),
        AttachmentKind::SolidAuxShape,
        &vertex,
    );
    assert_ne!(uid, 0);
    populate::perform(&mut graph, &second, false, &populate::Options).unwrap();
    assert_eq!(graph.topo().solids().nb(), 1);
    assert!(
        layer
            .get()
            .unwrap()
            .attached_to(SolidId::start().into())
            .is_empty()
    );
}

#[test]
fn BRepGraphInc_PopulateRegressionTest__FlattenedTraversal_SkipsInternalExternalChildren() {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    for (size, orientation) in [
        (1.0, Orientation::Forward),
        (2.0, Orientation::Internal),
        (3.0, Orientation::External),
    ] {
        let face = subshapes(&box_shape(size, size, size, false), ShapeEnum::Face).remove(0);
        let face = face.oriented(orientation);
        Builder.add(&mut compound, &face).unwrap();
    }
    let options = shapes_view::Options {
        create_auto_product: false,
        flatten: true,
        ..Default::default()
    };
    let mut graph = BRepGraph::new();
    let result = graph
        .shapes_mut()
        .add_with_options(&compound.into_shape(), &options)
        .unwrap();
    assert_eq!(graph.topo().faces().nb(), 1);
    assert_eq!(result.topology_root.kind, NodeKind::FACE);
}
