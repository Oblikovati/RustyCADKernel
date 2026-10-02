// Port of BOPAlgo_Alerts.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Typed Boolean failure and warning alerts.
use rustycad_tkbrep::define_alert_with_shape;
use rustycad_tkernel::define_simple_alert;

define_simple_alert!(AlertUserBreak, "BOPAlgo_AlertUserBreak");
define_simple_alert!(AlertBOPNotAllowed, "BOPAlgo_AlertBOPNotAllowed");
define_simple_alert!(AlertBOPNotSet, "BOPAlgo_AlertBOPNotSet");
define_simple_alert!(AlertBuilderFailed, "BOPAlgo_AlertBuilderFailed");
define_simple_alert!(AlertIntersectionFailed, "BOPAlgo_AlertIntersectionFailed");
define_simple_alert!(AlertMultipleArguments, "BOPAlgo_AlertMultipleArguments");
define_simple_alert!(AlertNoFiller, "BOPAlgo_AlertNoFiller");
define_simple_alert!(AlertNullInputShapes, "BOPAlgo_AlertNullInputShapes");
define_simple_alert!(AlertPostTreatFF, "BOPAlgo_AlertPostTreatFF");
define_simple_alert!(AlertSolidBuilderFailed, "BOPAlgo_AlertSolidBuilderFailed");
define_simple_alert!(AlertTooFewArguments, "BOPAlgo_AlertTooFewArguments");
define_alert_with_shape!(AlertBadPositioning, "BOPAlgo_AlertBadPositioning");
define_alert_with_shape!(AlertEmptyShape, "BOPAlgo_AlertEmptyShape");
define_alert_with_shape!(AlertNotSplittableEdge, "BOPAlgo_AlertNotSplittableEdge");
define_alert_with_shape!(
    AlertRemovalOfIBForEdgesFailed,
    "BOPAlgo_AlertRemovalOfIBForEdgesFailed"
);
define_alert_with_shape!(
    AlertRemovalOfIBForFacesFailed,
    "BOPAlgo_AlertRemovalOfIBForFacesFailed"
);
define_alert_with_shape!(
    AlertRemovalOfIBForMDimShapes,
    "BOPAlgo_AlertRemovalOfIBForMDimShapes"
);
define_alert_with_shape!(
    AlertRemovalOfIBForSolidsFailed,
    "BOPAlgo_AlertRemovalOfIBForSolidsFailed"
);
define_alert_with_shape!(
    AlertSelfInterferingShape,
    "BOPAlgo_AlertSelfInterferingShape"
);
define_alert_with_shape!(AlertShellSplitterFailed, "BOPAlgo_AlertShellSplitterFailed");
define_alert_with_shape!(AlertTooSmallEdge, "BOPAlgo_AlertTooSmallEdge");
define_alert_with_shape!(
    AlertIntersectionOfPairOfShapesFailed,
    "BOPAlgo_AlertIntersectionOfPairOfShapesFailed"
);
define_alert_with_shape!(
    AlertBuildingPCurveFailed,
    "BOPAlgo_AlertBuildingPCurveFailed"
);
define_alert_with_shape!(
    AlertAcquiredSelfIntersection,
    "BOPAlgo_AlertAcquiredSelfIntersection"
);
define_alert_with_shape!(AlertUnsupportedType, "BOPAlgo_AlertUnsupportedType");
define_simple_alert!(AlertNoFacesToRemove, "BOPAlgo_AlertNoFacesToRemove");
define_alert_with_shape!(
    AlertUnableToRemoveTheFeature,
    "BOPAlgo_AlertUnableToRemoveTheFeature"
);
define_simple_alert!(
    AlertRemoveFeaturesFailed,
    "BOPAlgo_AlertRemoveFeaturesFailed"
);
define_alert_with_shape!(
    AlertSolidBuilderUnusedFaces,
    "BOPAlgo_AlertSolidBuilderUnusedFaces"
);
define_alert_with_shape!(
    AlertFaceBuilderUnusedEdges,
    "BOPAlgo_AlertFaceBuilderUnusedEdges"
);
define_alert_with_shape!(
    AlertUnableToOrientTheShape,
    "BOPAlgo_AlertUnableToOrientTheShape"
);
define_alert_with_shape!(AlertUnknownShape, "BOPAlgo_AlertUnknownShape");
define_simple_alert!(
    AlertNoPeriodicityRequired,
    "BOPAlgo_AlertNoPeriodicityRequired"
);
define_alert_with_shape!(AlertUnableToTrim, "BOPAlgo_AlertUnableToTrim");
define_alert_with_shape!(
    AlertUnableToMakeIdentical,
    "BOPAlgo_AlertUnableToMakeIdentical"
);
define_alert_with_shape!(AlertUnableToRepeat, "BOPAlgo_AlertUnableToRepeat");
define_simple_alert!(
    AlertMultiDimensionalArguments,
    "BOPAlgo_AlertMultiDimensionalArguments"
);
define_alert_with_shape!(
    AlertUnableToMakePeriodic,
    "BOPAlgo_AlertUnableToMakePeriodic"
);
define_alert_with_shape!(AlertUnableToGlue, "BOPAlgo_AlertUnableToGlue");
define_alert_with_shape!(AlertShapeIsNotPeriodic, "BOPAlgo_AlertShapeIsNotPeriodic");
define_alert_with_shape!(
    AlertUnableToMakeClosedEdgeOnFace,
    "BOPAlgo_AlertUnableToMakeClosedEdgeOnFace"
);
