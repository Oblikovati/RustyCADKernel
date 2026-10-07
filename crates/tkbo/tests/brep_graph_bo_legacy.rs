//! Direct translation of the BRepGraph_Copy_Test.cxx and BRepGraph_Compact_Test.cxx cases that
//! build their input with the TKBO Boolean operations (src/ModelingData/TKBRep/GTests at 3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_graph::compact;
use rustycad_tkbrep::brep_graph::copy::{self, CachePolicy, GeomPolicy, MeshPolicy};
use rustycad_tkbrep::brep_graph::deduplicate::{self, Options as DeduplicateOptions};
use rustycad_tkbrep::brep_graph::validate;
use rustycad_tkbrep::brep_graph::*;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Axis2, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_gprop::BRepGProp;

fn surface_area(shape: &Shape) -> f64 {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

fn make_box(origin: Point3, size: f64) -> Shape {
    BRepPrimAPIMakeBox::at_point(origin, size, size, size)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

// Regularity (BRep_CurveOn2Surfaces) preserved via EncodeRegularity.
#[test]
fn BRepGraph_CopyTest__FusedBoxes_Regularity_AreaPreserved() {
    // Fuse two adjacent boxes to produce a shape with regularity edges.
    // Verify that the copy preserves area (geometry integrity).
    let first = make_box(Point3::default(), 10.0);
    let second = make_box(Point3::new(10.0, 0.0, 0.0), 10.0);
    let fuser = BRepAlgoAPIFuse::from_shapes(&first, &second).unwrap();
    assert!(fuser.is_done());
    let fused = fuser.shape().clone();
    let original_area = surface_area(&fused);

    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&fused).unwrap();
    assert!(!graph.is_empty());
    let mut copy_graph = BRepGraph::new();
    copy::perform(
        &graph,
        &mut copy_graph,
        GeomPolicy::Copy,
        MeshPolicy::Copy,
        CachePolicy::Drop,
    )
    .unwrap();
    assert!(!copy_graph.is_empty());

    // Verify node counts match.
    assert_eq!(copy_graph.topo().faces().nb(), graph.topo().faces().nb());
    assert_eq!(copy_graph.topo().edges().nb(), graph.topo().edges().nb());

    // Verify area is preserved by summing individual face areas.
    let mut copy_area = 0.0;
    for index in 0..copy_graph.topo().faces().nb() {
        let face = copy_graph
            .shapes()
            .reconstruct(FaceId::new(index).into())
            .unwrap();
        copy_area += surface_area(&face).abs();
    }
    assert!((copy_area - original_area).abs() <= original_area * 0.01);
}

#[test]
fn BRepGraph_CompactTest__Compact_FaceWithInnerWires_WireCountPreserved() {
    // Build a box with a cylindrical hole so that some faces have inner wires.
    let box_shape = make_box(Point3::default(), 20.0);
    let axes = Axis2::new(Point3::new(10.0, 10.0, 0.0), gp::dz()).unwrap();
    let cylinder = BRepPrimAPIMakeCylinder::with_axes(axes, 4.0, 25.0)
        .shape()
        .unwrap()
        .clone();
    let cut = BRepAlgoAPICut::from_shapes(&box_shape, &cylinder).unwrap();
    assert!(cut.is_done());

    let copy = || {
        BRepBuilderAPICopy::with_shape(cut.shape(), true, false)
            .unwrap()
            .shape()
            .unwrap()
            .clone()
    };
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut compound = compound.into_shape();
    builder.add(&mut compound, &copy()).unwrap();
    builder.add(&mut compound, &copy()).unwrap();

    let mut graph = BRepGraph::new();
    graph.shapes_mut().add(&compound).unwrap();
    assert!(!graph.is_empty());

    // Verify before dedup that the shape produces faces with inner wires (WireRefIds > 1).
    let has_inner_wires = (0..graph.topo().faces().nb()).any(|index| {
        graph
            .topo()
            .faces()
            .relations(FaceId::new(index))
            .wire_ref_ids
            .len()
            > 1
    });
    assert!(has_inner_wires);

    deduplicate::perform_with(
        &mut graph,
        &DeduplicateOptions {
            merge_entities_when_safe: true,
            ..DeduplicateOptions::default()
        },
    )
    .unwrap();
    let result = compact::perform(&mut graph).unwrap();
    assert_ne!(result.nb_nodes_after, 0);
    assert!(validate::perform(&graph).is_valid());
}
