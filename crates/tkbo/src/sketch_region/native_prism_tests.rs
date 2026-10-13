// SPDX-License-Identifier: GPL-2.0-only
//! Only the established cubic's native face corpus is accepted; full region validation is pending.
use super::tests::{arranged, closed_cubic};
use crate::bop_algo::BuilderFace;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Shape, edge, face},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::{brep_builder_api::BRepBuilderAPIMakeFace, brep_lib};

#[path = "../../tests/exact_region_prism/classification.rs"]
mod classification;
#[path = "../../tests/exact_region_prism/solid.rs"]
mod solid;
fn shapes(shape: &Shape, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut mapped = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut mapped).unwrap();
    mapped
}
fn cubic_region_inputs(arrangement: &Shape, plane: &rustycad_tkbrep::topods::Face) -> List<Shape> {
    let mapped = shapes(arrangement, ShapeEnum::Edge);
    assert_eq!(mapped.extent(), 1);
    let segment = edge(mapped.find_key(1).unwrap()).unwrap();
    brep_lib::build_pcurve_for_edge_on_plane(&segment, plane).unwrap();
    let mut inputs = List::default();
    inputs.append(segment.oriented(Orientation::Forward));
    inputs.append(segment.oriented(Orientation::Reversed));
    inputs
}
fn constructed_regions(arrangement: &Shape) -> BuilderFace {
    let plane = BRepBuilderAPIMakeFace::from_plane(Plane3::default())
        .unwrap()
        .face()
        .unwrap();
    let mut builder = BuilderFace::new();
    builder.set_face(&plane);
    builder.set_shapes(&cubic_region_inputs(arrangement, &plane));
    builder.perform().unwrap();
    assert!(!builder.options().has_errors());
    builder
}
#[test]
fn owned_stage_cubic_matches_existing_per_face_prism_native_receipt() {
    let original = closed_cubic();
    let owned = arranged(&original);
    let regions = constructed_regions(&owned.arrangement);
    assert_eq!(regions.areas().len(), 2);
    let bounded = classification::bounded_cubic(&regions, 1.);
    classification::check_seed_corpus(&bounded, 1.);
    solid::verify_region_prism(&bounded, 1.);
    assert_eq!(
        rustycad_tkbrep::brep::access::edge_tshape(&original, "source isolation")
            .unwrap()
            .curves()
            .len(),
        1
    );
}
