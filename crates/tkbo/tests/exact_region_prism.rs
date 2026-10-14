// SPDX-License-Identifier: GPL-2.0-only
//! Diagnostic region/prism bridge; no production extension or strict validity claim (#10).
use rustycad_tkbo::bop_algo::{BOPAlgoBuilder, BOPAlgoPaveFiller, BuilderFace, BuilderHooks};
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp,
    topods::{Shape, edge, face},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkg3d::{
    geom::GeomBezierCurve,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::{brep_builder_api::BRepBuilderAPIMakeFace, brep_lib};
#[path = "exact_region_prism/classification.rs"]
mod classification;
#[path = "exact_region_prism/ownership.rs"]
mod ownership;
#[path = "exact_region_prism/solid.rs"]
mod solid;

struct NonemptyProbe(BOPAlgoBuilder);
impl BuilderHooks for NonemptyProbe {
    fn general_fuse(&mut self) -> &mut BOPAlgoBuilder {
        &mut self.0
    }
    fn check_data(&mut self, filler: &BOPAlgoPaveFiller) {
        // Match the existing issue #10 experiment; default mapped behavior stays unchanged.
        self.0.options().report().merge(filler.options().report());
    }
}
#[path = "support/closed_analytic_cubic.rs"]
mod cubic_fixture;
use cubic_fixture::closed_analytic_cubic as cubic_source;

fn arrange(arguments: &List<Shape>) -> NonemptyProbe {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.set_arguments(arguments);
    filler.set_non_destructive(true);
    filler.perform();
    let mut builder = NonemptyProbe(BOPAlgoBuilder::new());
    builder.0.set_arguments(arguments);
    builder.0.set_to_fill_history(true);
    builder.perform_internal1(&mut filler).unwrap();
    builder
}
fn arranged_cubic(source: &Shape) -> NonemptyProbe {
    let mut arguments = List::default();
    arguments.append(source.clone());
    let arrangement = arrange(&arguments);
    assert!(!arrangement.0.options().has_errors());
    assert!(arrangement.0.modified(source).unwrap().is_empty());
    assert!(arrangement.0.generated(source).unwrap().is_empty());
    assert!(!arrangement.0.is_deleted(source).unwrap());
    assert!(
        shapes(arrangement.0.shape(), ShapeEnum::Edge)
            .find_key(1)
            .unwrap()
            .is_same(source)
    );
    arrangement
}
fn shapes(shape: &Shape, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut found = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut found).unwrap();
    found
}
fn region_inputs(
    arrangement: &NonemptyProbe,
    plane: &rustycad_tkbrep::topods::Face,
) -> List<Shape> {
    let split = shapes(arrangement.0.shape(), ShapeEnum::Edge);
    assert_eq!(split.extent(), 1);
    let segment = edge(split.find_key(1).unwrap()).unwrap();
    brep_lib::build_pcurve_for_edge_on_plane(&segment, plane).unwrap();
    let mut inputs = List::default();
    inputs.append(segment.oriented(Orientation::Forward));
    inputs.append(segment.oriented(Orientation::Reversed));
    inputs
}
fn regions(arrangement: &NonemptyProbe) -> BuilderFace {
    let plane = BRepBuilderAPIMakeFace::from_plane(Plane3::default())
        .unwrap()
        .face()
        .unwrap();
    let mut builder = BuilderFace::new();
    builder.set_face(&plane);
    builder.set_shapes(&region_inputs(arrangement, &plane));
    builder.perform().unwrap();
    assert!(!builder.options().has_errors());
    assert_eq!(builder.areas().len(), 2);
    builder
}
#[test]
fn exact_region_prism_matches_every_native_cubic_face() {
    let source = cubic_source(1., false);
    let builder = regions(&arranged_cubic(&source));
    let region = classification::bounded_cubic(&builder, 1.);
    classification::check_seed_corpus(&region, 1.);
    solid::verify_region_prism(&region, 1.);
}
#[test]
fn exact_region_prism_generated_scales_and_source_reversals() {
    for exponent in -8..=8 {
        for reverse in [false, true] {
            let scale = 2_f64.powi(exponent);
            let source = cubic_source(scale, reverse);
            let builder = regions(&arranged_cubic(&source));
            let region = classification::bounded_cubic(&builder, scale);
            classification::check_seed_corpus(&region, scale);
            solid::verify_region_prism(&region, scale);
        }
    }
}
#[test]
fn exact_region_prism_empty_input_is_a_named_decline() {
    let builder = arrange(&List::default());
    assert!(builder.0.options().has_errors());
    assert!(builder.0.shape().is_null());
    let mut messages = Vec::new();
    builder.0.options().dump_errors(&mut messages).unwrap();
    assert!(
        String::from_utf8(messages)
            .unwrap()
            .contains("TooFewArguments")
    );
}
