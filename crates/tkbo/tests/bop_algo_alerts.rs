//! Checks every source-declared Boolean alert runtime identity and merge policy.
use rustycad_tkbo::bop_algo::*;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Shape, Vertex};
use rustycad_tkernel::message::{Alert, AlertHandle};
use rustycad_tkmath::gp::Point3;
use std::sync::Arc;

fn alert_vertex() -> Shape {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(1.0, 2.0, 3.0), 1e-7)
        .unwrap();
    vertex.into()
}

fn assert_alert_identity(alert: &dyn Alert, name: &str, parent: &str, merge: bool) {
    assert_eq!(alert.get_message_key().as_bytes(), name.as_bytes());
    assert_eq!(alert.dynamic_type().name(), name);
    assert_eq!(alert.dynamic_type().parent().unwrap().name(), parent);
    assert_eq!(alert.supports_merge(), merge);
}

macro_rules! simple_alert_case {
    ($test:ident, $alert:ident, $name:literal) => {
        #[test]
        fn $test() {
            let alert = $alert;
            assert_alert_identity(&alert, $name, "Message_Alert", true);
            let target: AlertHandle = Arc::new($alert);
            assert!(alert.merge(&target));
        }
    };
}

macro_rules! shape_alert_case {
    ($test:ident, $alert:ident, $name:literal) => {
        #[test]
        fn $test() {
            let shape = alert_vertex();
            let alert = $alert::new(&shape);
            assert_alert_identity(&alert, $name, "TopoDS_AlertWithShape", false);
            assert!(alert.get_shape().is_same(&shape));
            let target: AlertHandle = Arc::new($alert::new(&shape));
            assert!(!alert.merge(&target));
            alert.set_shape(&Shape::default());
            assert!(alert.get_shape().is_null());
        }
    };
}

simple_alert_case!(alert_user_break, AlertUserBreak, "BOPAlgo_AlertUserBreak");
simple_alert_case!(
    alert_b_o_p_not_allowed,
    AlertBOPNotAllowed,
    "BOPAlgo_AlertBOPNotAllowed"
);
simple_alert_case!(
    alert_b_o_p_not_set,
    AlertBOPNotSet,
    "BOPAlgo_AlertBOPNotSet"
);
simple_alert_case!(
    alert_builder_failed,
    AlertBuilderFailed,
    "BOPAlgo_AlertBuilderFailed"
);
simple_alert_case!(
    alert_intersection_failed,
    AlertIntersectionFailed,
    "BOPAlgo_AlertIntersectionFailed"
);
simple_alert_case!(
    alert_multiple_arguments,
    AlertMultipleArguments,
    "BOPAlgo_AlertMultipleArguments"
);
simple_alert_case!(alert_no_filler, AlertNoFiller, "BOPAlgo_AlertNoFiller");
simple_alert_case!(
    alert_null_input_shapes,
    AlertNullInputShapes,
    "BOPAlgo_AlertNullInputShapes"
);
simple_alert_case!(
    alert_post_treat_f_f,
    AlertPostTreatFF,
    "BOPAlgo_AlertPostTreatFF"
);
simple_alert_case!(
    alert_solid_builder_failed,
    AlertSolidBuilderFailed,
    "BOPAlgo_AlertSolidBuilderFailed"
);
simple_alert_case!(
    alert_too_few_arguments,
    AlertTooFewArguments,
    "BOPAlgo_AlertTooFewArguments"
);
shape_alert_case!(
    alert_bad_positioning,
    AlertBadPositioning,
    "BOPAlgo_AlertBadPositioning"
);
shape_alert_case!(
    alert_empty_shape,
    AlertEmptyShape,
    "BOPAlgo_AlertEmptyShape"
);
shape_alert_case!(
    alert_not_splittable_edge,
    AlertNotSplittableEdge,
    "BOPAlgo_AlertNotSplittableEdge"
);
shape_alert_case!(
    alert_removal_of_i_b_for_edges_failed,
    AlertRemovalOfIBForEdgesFailed,
    "BOPAlgo_AlertRemovalOfIBForEdgesFailed"
);
shape_alert_case!(
    alert_removal_of_i_b_for_faces_failed,
    AlertRemovalOfIBForFacesFailed,
    "BOPAlgo_AlertRemovalOfIBForFacesFailed"
);
shape_alert_case!(
    alert_removal_of_i_b_for_m_dim_shapes,
    AlertRemovalOfIBForMDimShapes,
    "BOPAlgo_AlertRemovalOfIBForMDimShapes"
);
shape_alert_case!(
    alert_removal_of_i_b_for_solids_failed,
    AlertRemovalOfIBForSolidsFailed,
    "BOPAlgo_AlertRemovalOfIBForSolidsFailed"
);
shape_alert_case!(
    alert_self_interfering_shape,
    AlertSelfInterferingShape,
    "BOPAlgo_AlertSelfInterferingShape"
);
shape_alert_case!(
    alert_shell_splitter_failed,
    AlertShellSplitterFailed,
    "BOPAlgo_AlertShellSplitterFailed"
);
shape_alert_case!(
    alert_too_small_edge,
    AlertTooSmallEdge,
    "BOPAlgo_AlertTooSmallEdge"
);
shape_alert_case!(
    alert_intersection_of_pair_of_shapes_failed,
    AlertIntersectionOfPairOfShapesFailed,
    "BOPAlgo_AlertIntersectionOfPairOfShapesFailed"
);
shape_alert_case!(
    alert_building_p_curve_failed,
    AlertBuildingPCurveFailed,
    "BOPAlgo_AlertBuildingPCurveFailed"
);
shape_alert_case!(
    alert_acquired_self_intersection,
    AlertAcquiredSelfIntersection,
    "BOPAlgo_AlertAcquiredSelfIntersection"
);
shape_alert_case!(
    alert_unsupported_type,
    AlertUnsupportedType,
    "BOPAlgo_AlertUnsupportedType"
);
simple_alert_case!(
    alert_no_faces_to_remove,
    AlertNoFacesToRemove,
    "BOPAlgo_AlertNoFacesToRemove"
);
shape_alert_case!(
    alert_unable_to_remove_the_feature,
    AlertUnableToRemoveTheFeature,
    "BOPAlgo_AlertUnableToRemoveTheFeature"
);
simple_alert_case!(
    alert_remove_features_failed,
    AlertRemoveFeaturesFailed,
    "BOPAlgo_AlertRemoveFeaturesFailed"
);
shape_alert_case!(
    alert_solid_builder_unused_faces,
    AlertSolidBuilderUnusedFaces,
    "BOPAlgo_AlertSolidBuilderUnusedFaces"
);
shape_alert_case!(
    alert_face_builder_unused_edges,
    AlertFaceBuilderUnusedEdges,
    "BOPAlgo_AlertFaceBuilderUnusedEdges"
);
shape_alert_case!(
    alert_unable_to_orient_the_shape,
    AlertUnableToOrientTheShape,
    "BOPAlgo_AlertUnableToOrientTheShape"
);
shape_alert_case!(
    alert_unknown_shape,
    AlertUnknownShape,
    "BOPAlgo_AlertUnknownShape"
);
simple_alert_case!(
    alert_no_periodicity_required,
    AlertNoPeriodicityRequired,
    "BOPAlgo_AlertNoPeriodicityRequired"
);
shape_alert_case!(
    alert_unable_to_trim,
    AlertUnableToTrim,
    "BOPAlgo_AlertUnableToTrim"
);
shape_alert_case!(
    alert_unable_to_make_identical,
    AlertUnableToMakeIdentical,
    "BOPAlgo_AlertUnableToMakeIdentical"
);
shape_alert_case!(
    alert_unable_to_repeat,
    AlertUnableToRepeat,
    "BOPAlgo_AlertUnableToRepeat"
);
simple_alert_case!(
    alert_multi_dimensional_arguments,
    AlertMultiDimensionalArguments,
    "BOPAlgo_AlertMultiDimensionalArguments"
);
shape_alert_case!(
    alert_unable_to_make_periodic,
    AlertUnableToMakePeriodic,
    "BOPAlgo_AlertUnableToMakePeriodic"
);
shape_alert_case!(
    alert_unable_to_glue,
    AlertUnableToGlue,
    "BOPAlgo_AlertUnableToGlue"
);
shape_alert_case!(
    alert_shape_is_not_periodic,
    AlertShapeIsNotPeriodic,
    "BOPAlgo_AlertShapeIsNotPeriodic"
);
shape_alert_case!(
    alert_unable_to_make_closed_edge_on_face,
    AlertUnableToMakeClosedEdgeOnFace,
    "BOPAlgo_AlertUnableToMakeClosedEdgeOnFace"
);
