//! 1:1 translations of src/ApplicationFramework/TKLCAF/GTests/TNaming_Builder_Test.cxx and
//! TNaming_Name_Test.cxx: TKLCAF GTests on TNaming, which the TKCAF crate provides.
#![allow(non_snake_case)]
mod support;

use rustycad_tkcaf::tnaming::{TNamingBuilder, TNamingName, TNamingNameType, TNamingNamedShape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tklcaf::tdf::{TdfData, TdfLabelMap};

// Test OCC361bug: TNaming_Builder preservation of shape orientation.
#[test]
fn TNaming_Builder_Test__OCC361_ShapeOrientationPreservation() {
    let document = support::new_document();
    let mut solid = support::make_box_at([0.0, 0.0, 0.0], 100.0, 100.0, 100.0);
    solid.set_orientation(Orientation::Forward);
    let label = document.main().unwrap();
    TNamingBuilder::new(&label)
        .unwrap()
        .generated(&solid)
        .unwrap();
    // A copy with the REVERSED orientation.
    let mut reversed = solid.clone();
    reversed.set_orientation(Orientation::Reversed);
    label.forget_all_attributes(true).unwrap();
    let builder = TNamingBuilder::new(&label).unwrap();
    builder.generated(&reversed).unwrap();
    let shape = TNamingNamedShape::get(&builder.named_shape()).unwrap();
    assert_eq!(
        shape.orientation(),
        Orientation::Reversed,
        "Shape orientation should be preserved as REVERSED after TNaming_Builder operations"
    );
}

// Test BUC60925: TNaming_Name solve with an empty NamedShape.
#[test]
fn TNaming_Name_Test__BUC60925_SolveWithEmptyNamedShape() {
    let data = TdfData::new();
    let label = data.root().find_child(2, true).unwrap();
    let mut labels = TdfLabelMap::default();
    labels.add(label.clone());
    let named: Handle<TNamingNamedShape> = Handle::new(TNamingNamedShape::new());
    let mut name = TNamingName::new();
    name.set_type(TNamingNameType::Identity);
    name.append(&named);
    assert!(
        !name.solve(&label, &labels),
        "Solve should return false for empty NamedShape"
    );
}
