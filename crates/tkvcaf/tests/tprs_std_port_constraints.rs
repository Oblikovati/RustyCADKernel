//! Port tests of TPrsStd_ConstraintTools and TPrsStd_ConstraintDriver: the presentation of
//! each constraint type, the value text and the state colors.
mod support;

use rustycad_tkcaf::tdata_xtd::{TDataXtdConstraintEnum as Kind, TDataXtdPosition};
use rustycad_tkernel::quantity::NamedColor;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdName, TDataStdReal};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfReference};
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::ais::AisInteractiveObject;
use rustycad_tkv3d::prs_dim::{
    PrsDimConcentricRelation, PrsDimDiameterDimension, PrsDimFixRelation, PrsDimIdenticRelation,
    PrsDimLengthDimension, PrsDimMaxRadiusDimension, PrsDimMinRadiusDimension,
    PrsDimParallelRelation, PrsDimPerpendicularRelation, PrsDimRadiusDimension,
    PrsDimRelationMembers, PrsDimRelationSource, PrsDimTangentRelation,
};
use rustycad_tkvcaf::tprs_std::{TPrsStdConstraintDriver, TPrsStdConstraintTools, TPrsStdDriver};

fn p(x: f64, y: f64, z: f64) -> Point3 {
    Point3::new(x, y, z)
}

#[test]
fn text_and_value_format_the_value_and_the_captured_name() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let edge = support::named_shape(&root, &support::circle_edge(p(0.0, 0.0, 0.0), 2.5));
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Radius, &[edge], 2.5);
    let (mut value, mut text) = (0.0, ExtendedString::default());
    TPrsStdConstraintTools::compute_text_and_value(&constraint, &mut value, &mut text, false)
        .unwrap();
    assert_eq!(value, 2.5);
    assert_eq!(text.to_utf8(), "2.5");

    let parameters = root.new_child().unwrap();
    TDataStdName::set(&parameters, &ExtendedString::from_utf8("Params")).unwrap();
    let origin = parameters.new_child().unwrap();
    TDataStdName::set(&origin, &ExtendedString::from_utf8("R")).unwrap();
    TDataStdReal::set(&origin, 2.5).unwrap();
    let real = constraint.get_value();
    TdfReference::set(&real.label(), &origin).unwrap();
    TPrsStdConstraintTools::compute_text_and_value(&constraint, &mut value, &mut text, false)
        .unwrap();
    assert_eq!(text.to_utf8(), "Params.R=2.5");
}

#[test]
fn radius_diameter_and_round_of_a_circle() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let edge = support::named_shape(&root, &support::circle_edge(p(0.0, 0.0, 0.0), 3.0));
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Radius, &[edge], 3.0);
    let radius = TPrsStdConstraintTools::compute_radius(&constraint).unwrap();
    assert!(!radius.cast::<PrsDimRadiusDimension>().is_null());
    let diameter = TPrsStdConstraintTools::compute_diameter(&constraint).unwrap();
    assert!(!diameter.cast::<PrsDimDiameterDimension>().is_null());
    let round = TPrsStdConstraintTools::compute_round(&constraint).unwrap();
    assert!(!round.cast::<PrsDimRadiusDimension>().is_null());
    let minor = TPrsStdConstraintTools::compute_min_radius(&constraint).unwrap();
    assert!(!minor.cast::<PrsDimMinRadiusDimension>().is_null());
    let major = TPrsStdConstraintTools::compute_max_radius(&constraint).unwrap();
    let major = major.cast::<PrsDimMaxRadiusDimension>();
    assert_eq!(major.value(), 3.0);
    assert_eq!(major.text().to_utf8(), "3");
}

#[test]
fn radius_of_a_solid_is_not_shown() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let solid = support::named_shape(&root, &support::make_box(1.0, 1.0, 1.0));
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Radius, &[solid], 1.0);
    assert!(
        TPrsStdConstraintTools::compute_radius(&constraint)
            .unwrap()
            .is_null()
    );
}

#[test]
fn planar_relations_on_the_constraint_plane() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let first = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)),
    );
    let second = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0)),
    );
    let plane = support::xy_plane(&root);
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Parallel, &[first, second], 0.0);
    assert!(
        TPrsStdConstraintTools::compute_parallel(&constraint)
            .unwrap()
            .is_null()
    );
    constraint.set_plane(&plane).unwrap();
    let parallel = TPrsStdConstraintTools::compute_parallel(&constraint).unwrap();
    assert!(!parallel.cast::<PrsDimParallelRelation>().is_null());
    let perpendicular = TPrsStdConstraintTools::compute_perpendicular(&constraint).unwrap();
    assert!(
        !perpendicular
            .cast::<PrsDimPerpendicularRelation>()
            .plane()
            .is_null()
    );
    let tangent = TPrsStdConstraintTools::compute_tangent(&constraint).unwrap();
    assert_eq!(
        tangent.cast::<PrsDimTangentRelation>().arrow_size(),
        10_000_000.0
    );
    let coincident = TPrsStdConstraintTools::compute_coincident(&constraint).unwrap();
    assert!(!coincident.cast::<PrsDimIdenticRelation>().is_null());
    let concentric = TPrsStdConstraintTools::compute_concentric(&constraint).unwrap();
    assert!(!concentric.cast::<PrsDimConcentricRelation>().is_null());
    let fix = TPrsStdConstraintTools::compute_fix(&constraint).unwrap();
    assert!(!fix.cast::<PrsDimFixRelation>().is_null());
    assert!(
        TPrsStdConstraintTools::compute_others(&constraint)
            .unwrap()
            .is_null()
    );
}

#[test]
fn too_few_geometries_raise_or_give_nothing() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let edge = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)),
    );
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Concentric, &[edge], 0.0);
    let concentric = TPrsStdConstraintTools::compute_concentric(&constraint);
    assert!(concentric.is_err_and(|failure| failure.is_kind("Standard_ProgramError")));
    let placement = TPrsStdConstraintTools::compute_placement(&constraint);
    assert!(placement.is_err_and(|failure| failure.is_kind("Standard_ProgramError")));
    assert!(
        TPrsStdConstraintTools::compute_parallel(&constraint)
            .unwrap()
            .is_null()
    );
    assert!(
        TPrsStdConstraintTools::compute_symmetry(&constraint)
            .unwrap()
            .is_null()
    );
    assert!(
        TPrsStdConstraintTools::compute_equal_distance(&constraint)
            .unwrap()
            .is_null()
    );
}

#[test]
fn distance_between_two_vertices_builds_its_plane() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let first = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 0.0, 0.0), p(4.0, 0.0, 0.0)),
    );
    let second = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 2.0, 0.0), p(4.0, 2.0, 0.0)),
    );
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Distance, &[first, second], 2.0);
    let distance = TPrsStdConstraintTools::compute_distance(&constraint).unwrap();
    assert!(!distance.cast::<PrsDimLengthDimension>().is_null());
}

#[test]
fn update_only_value_rewrites_the_relation_text() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let edge = support::named_shape(&root, &support::circle_edge(p(0.0, 0.0, 0.0), 1.5));
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::MinorRadius, &[edge], 1.5);
    let minor = TPrsStdConstraintTools::compute_min_radius(&constraint).unwrap();
    let relation = minor.cast::<dyn PrsDimRelationSource>();
    relation.set_text(&ExtendedString::from_utf8("old"));
    let real = constraint.get_value();
    real.set_value(4.0).unwrap();
    TPrsStdConstraintTools::update_only_value(&constraint, &minor).unwrap();
    assert_eq!(relation.text().to_utf8(), "4");
}

#[test]
fn constraint_driver_colors_and_places_the_relation() {
    let document = support::new_document();
    let root = document.main().unwrap();
    let first = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 0.0, 0.0), p(1.0, 0.0, 0.0)),
    );
    let second = support::named_shape(
        &root,
        &support::segment_edge(p(0.0, 1.0, 0.0), p(1.0, 1.0, 0.0)),
    );
    let plane = support::xy_plane(&root);
    let label = root.new_child().unwrap();
    let constraint = support::constraint(&label, Kind::Parallel, &[first, second], 0.0);
    constraint.set_plane(&plane).unwrap();
    TDataXtdPosition::set_position_of(&label, p(5.0, 6.0, 7.0)).unwrap();
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdConstraintDriver::new()
            .update(&label, &mut object)
            .unwrap()
    );
    let relation = object.cast::<dyn PrsDimRelationSource>();
    assert!(relation.position().is_equal(&p(5.0, 6.0, 7.0), 0.0));
    assert_ne!(object.color().name(), NamedColor::RED);
    constraint.set_verified(false).unwrap();
    let mut object: Handle<dyn AisInteractiveObject> = Handle::default();
    assert!(
        TPrsStdConstraintDriver::new()
            .update(&label, &mut object)
            .unwrap()
    );
    assert_eq!(object.color().name(), NamedColor::RED);
}
