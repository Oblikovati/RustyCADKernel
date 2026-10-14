// SPDX-License-Identifier: GPL-2.0-only
use super::{tests::*, *};
use crate::sketch_region::{ArrangementRelation, equal_occurrences};
use crate::sketch_region_candidates::tests::{candidates, rectangle};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkg3d::top_abs::Orientation;
#[test]
fn complete_context_history_keeps_native_return_and_actual_orientation_distinct() {
    let result = retain_selected_prism(
        candidates(&rectangle(0., 0., 4., 3.)),
        request([2., 1.]),
        &mut progress(None, None),
    );
    let diagnostic = result.diagnostic.unwrap();
    assert!(diagnostic.transport_gaps.is_empty());
    assert_returned_context_difference(&diagnostic);
    assert_binding_integrity(&diagnostic);
    assert_all_roles(&diagnostic);
}
#[test]
fn opposite_seam_occurrences_have_independent_full_keys_and_context_paths() {
    let result = cubic_receipt([1., 1.], &mut progress(None, None));
    let bindings = &result.diagnostic.as_ref().unwrap().bindings;
    assert!(bindings.iter().any(|first| bindings.iter().any(|last| {
        first
            .native
            .returned
            .output
            .is_equal(&last.native.returned.output)
            && first.context.occurrence.is_same(&last.context.occurrence)
            && !first.context.occurrence.is_equal(&last.context.occurrence)
            && first.context.child_slots != last.context.child_slots
    })));
    assert_context_parent_coverage(result.diagnostic.as_ref().unwrap());
}
fn assert_context_parent_coverage(diagnostic: &PrismDiagnostic) {
    for context in &diagnostic.bindings {
        let row = diagnostic
            .relations
            .iter()
            .find(|row| row.output.is_equal(&context.context.occurrence))
            .unwrap();
        for parent in &context.native.returned.generated {
            assert!(row.generated.contains(parent));
        }
        for parent in &context.native.returned.modified {
            assert!(row.modified.contains(parent));
        }
    }
}
#[test]
fn native_full_forward_reversed_occurrence_matrix_replays_without_occt() {
    let actual = orientation_case(false, 0.)
        + &orientation_case(true, 0.)
        + &orientation_case(false, 10.)
        + &orientation_case(true, 10.);
    assert_eq!(
        actual,
        include_str!("../../tests/fixtures/prism-orientation-native.txt")
    );
    assert!(actual.lines().any(|row| row.contains("first 4 0 1 0")));
}
fn orientation_case(reverse: bool, offset: f64) -> String {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    let (base, mut maker) = orientation_maker(reverse, offset);
    let inputs = equal_occurrences(&base).unwrap();
    let outputs = equal_occurrences(maker.shape().unwrap()).unwrap();
    orientation_rows(reverse, offset, &inputs, &outputs, &mut maker)
}
fn orientation_member(
    maker: &mut rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism,
    source: &Shape,
    index: usize,
    outputs: &[Shape],
) -> String {
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
    let mut row = orientation_member_header(index, source);
    let generated = maker.generated(source).unwrap().clone();
    for image in &generated {
        row.push_str(&orientation_image("generated", image, outputs));
    }
    append_native_caps(&mut row, maker, source, outputs);
    row.push('\n');
    row
}
fn orientation_image(role: &str, image: &Shape, outputs: &[Shape]) -> String {
    let same = outputs
        .iter()
        .filter(|output| output.is_same(image))
        .count();
    let equal = outputs
        .iter()
        .filter(|output| output.is_equal(image))
        .count();
    format!(
        " {} {} {} {} {}",
        role,
        image.shape_type().unwrap() as i32,
        image.orientation() as i32,
        same,
        equal
    )
}
#[test]
fn unknown_image_is_a_typed_gap_not_a_fabricated_output_occurrence() {
    let body = super::native_tests::rectangle_face();
    let other = crate::closed_analytic_cubic::closed_analytic_cubic(1., false);
    let returned = empty_observed_row(&other);
    let observed = vec![PrismNativeImage {
        query: other,
        role: PrismImageRole::Generated,
        returned,
    }];
    let contexts = super::contexts::body_contexts(&body)
        .unwrap_or_else(|_| panic!("expected actual body contexts"));
    let (bindings, gaps) = super::contexts::bind_observed_images(observed, &contexts);
    assert!(bindings.is_empty());
    assert!(matches!(&gaps[0], PrismGap::MissingImageContext(_)));
    assert_eq!(body.orientation(), Orientation::Forward);
}

#[test]
fn conflicting_actual_full_keys_at_one_declared_path_are_typed_ambiguous() {
    let body = super::native_tests::rectangle_face();
    let original =
        super::contexts::body_contexts(&body).unwrap_or_else(|_| panic!("expected contexts"));
    let mut conflicting = original[0].clone();
    conflicting.occurrence = body.reversed();
    let contexts = vec![original[0].clone(), conflicting];
    let row = empty_observed_row(&body);
    let image = PrismNativeImage {
        query: body,
        role: PrismImageRole::First,
        returned: row,
    };
    let (bindings, gaps) = super::contexts::bind_observed_images(vec![image], &contexts);
    assert!(bindings.is_empty());
    assert!(
        gaps.iter()
            .any(|gap| matches!(gap, PrismGap::AmbiguousContextPath { .. }))
    );
}
#[test]
fn changing_body_children_preserves_actual_iterator_failure() {
    let shape =
        crate::changing_child_count::changing_shape(rustycad_tkg3d::top_abs::ShapeEnum::Face);
    let Err(PrismCause::Mapped { phase, failure }) = super::contexts::body_contexts(&shape) else {
        panic!("expected original Value failure");
    };
    assert_eq!(phase, PrismPhase::History);
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert!(failure.message().contains("TopoDS_Iterator::Value"));
}

fn assert_returned_context_difference(diagnostic: &PrismDiagnostic) {
    assert!(diagnostic.bindings.iter().any(|binding| {
        !binding
            .native
            .returned
            .output
            .is_equal(&binding.context.occurrence)
    }));
    assert!(diagnostic.bindings.iter().all(|binding| {
        binding
            .native
            .returned
            .output
            .is_same(&binding.context.occurrence)
    }));
    assert!(!diagnostic.bindings.is_empty());
}
fn assert_binding_integrity(diagnostic: &PrismDiagnostic) {
    for binding in &diagnostic.bindings {
        assert!(binding.transported.is_equal(&binding.context.occurrence));
        assert!(!binding.native.query.is_null());
        assert_eq!(
            binding.context.child_slots.len(),
            binding.context.ancestors.len()
        );
        if let Some(root) = binding.context.ancestors.first() {
            assert!(root.is_equal(&diagnostic.body));
        }
    }
}
fn assert_all_roles(diagnostic: &PrismDiagnostic) {
    for role in [
        PrismImageRole::Generated,
        PrismImageRole::First,
        PrismImageRole::Last,
    ] {
        assert!(
            diagnostic
                .bindings
                .iter()
                .any(|binding| binding.native.role == role)
        );
    }
}

fn orientation_maker(
    reverse: bool,
    offset: f64,
) -> (Shape, rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism) {
    use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeShape};
    let face = placed_rectangle(reverse, offset);
    let mut copier = BRepBuilderAPICopy::with_shape(&face, true, true).unwrap();
    let base = copier.shape().unwrap().clone();
    let maker = rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism::new(
        &base,
        request([2., 1.]).extrusion,
        false,
        true,
    )
    .unwrap();
    (base, maker)
}
fn orientation_rows(
    reverse: bool,
    offset: f64,
    inputs: &[Shape],
    outputs: &[Shape],
    maker: &mut rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism,
) -> String {
    let mut result = format!(
        "case {} {} {} {}\n",
        u8::from(reverse),
        offset,
        inputs.len(),
        outputs.len()
    );
    for (index, shape) in inputs.iter().enumerate() {
        result.push_str(&orientation_member(maker, shape, index + 1, outputs));
    }
    result
}

fn orientation_member_header(index: usize, source: &Shape) -> String {
    format!(
        "member {} {} {}",
        index,
        source.shape_type().unwrap() as i32,
        source.orientation() as i32
    )
}
fn append_native_caps(
    row: &mut String,
    maker: &mut rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism,
    source: &Shape,
    outputs: &[Shape],
) {
    row.push_str(&orientation_image(
        "first",
        &maker.first_shape_of(source).unwrap(),
        outputs,
    ));
    row.push_str(&orientation_image(
        "last",
        &maker.last_shape_of(source).unwrap(),
        outputs,
    ));
}

fn placed_rectangle(reverse: bool, offset: f64) -> Shape {
    let mut face = super::native_tests::rectangle_face();
    if reverse {
        face.reverse();
    }
    let mut placement = rustycad_tkmath::gp::Transform3::default();
    placement.set_translation(rustycad_tkmath::gp::Vector3::new(offset, 0., 0.));
    face.located(&rustycad_tkmath::toploc::Location::new(placement), true)
        .unwrap()
}

fn empty_observed_row(output: &Shape) -> ArrangementRelation {
    ArrangementRelation {
        output: output.clone(),
        preserved: Vec::new(),
        modified: Vec::new(),
        generated: Vec::new(),
    }
}
