//! Direct translation of BRepGraph_RepId_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkbrep::brep_graph::{
    CoEdgeCurve2DRepId, CoEdgePolygon2DRepId, CoEdgePolygonOnTriRepId, EdgeCurve3DRepId,
    EdgePolygon3DRepId, FaceSurfaceRepId, FaceTriangulationRepId, RepId, RepKind,
};

#[test]
fn BRepGraph_RepIdTest__DefaultRepId_IsInvalid() {
    let anId = RepId::default();
    assert!(!anId.is_valid());
}

#[test]
fn BRepGraph_RepIdTest__TypedDefaultRepId_IsInvalid() {
    let aCurve = EdgeCurve3DRepId::default();
    let aSurface = FaceSurfaceRepId::default();
    let aPCurve = CoEdgeCurve2DRepId::default();
    let aTri = FaceTriangulationRepId::default();
    assert!(!aCurve.is_valid());
    assert!(!aSurface.is_valid());
    assert!(!aPCurve.is_valid());
    assert!(!aTri.is_valid());
}

#[test]
fn BRepGraph_RepIdTest__TypedRepId_ConvertsToUntyped() {
    let aTyped = EdgeCurve3DRepId::new(42);
    let anUntyped: RepId = aTyped.into();
    assert_eq!(anUntyped.kind, RepKind::EDGE_CURVE_3D);
    assert_eq!(anUntyped.index, 42u32);
}

#[test]
fn BRepGraph_RepIdTest__KindClassification() {
    assert!(RepKind::is_valid_kind(RepKind::EDGE_CURVE_3D));
    assert!(RepKind::is_valid_kind(RepKind::FACE_SURFACE));
    assert!(RepKind::is_valid_kind(RepKind::CO_EDGE_CURVE_2D));
    assert!(RepKind::is_valid_kind(RepKind::FACE_TRIANGULATION));
}

#[test]
fn BRepGraph_RepIdTest__ValidRepId_PassesBoundsCheck() {
    let anId = FaceSurfaceRepId::new(5);
    assert!(anId.is_valid());
    assert!(anId.is_valid_for(10));
    assert!(!anId.is_valid_for(3));
}

#[test]
fn BRepGraph_RepIdTest__StartId_IsZero() {
    let aStart = EdgeCurve3DRepId::start();
    assert!(aStart.is_valid());
    assert_eq!(aStart.index, 0u32);
}

#[test]
fn BRepGraph_RepIdTest__EqualityAndComparison() {
    let a1 = EdgeCurve3DRepId::new(1);
    let a2 = EdgeCurve3DRepId::new(2);
    let a1Copy = EdgeCurve3DRepId::new(1);
    assert_eq!(a1, a1Copy);
    assert_ne!(a1, a2);
    assert!(a1 < a2);
    assert!(a1 <= a1Copy);
    assert!(a2 > a1);
    assert!(a1Copy >= a1);
}

#[test]
fn BRepGraph_RepIdTest__DifferentKinds_AreNotEqual() {
    let aCurve = RepId::new(RepKind::EDGE_CURVE_3D, 0);
    let aSurface = RepId::new(RepKind::FACE_SURFACE, 0);
    assert_ne!(aCurve, aSurface);
}

#[test]
fn BRepGraph_RepIdTest__Increment() {
    let mut anId = EdgeCurve3DRepId::start();
    assert_eq!(anId.index, 0u32);
    anId.pre_increment();
    assert_eq!(anId.index, 1u32);
    anId.post_increment();
    assert_eq!(anId.index, 2u32);
}

#[test]
fn BRepGraph_RepIdTest__AllTypedAliases_DefaultInvalid() {
    assert!(!EdgeCurve3DRepId::default().is_valid());
    assert!(!EdgePolygon3DRepId::default().is_valid());
    assert!(!CoEdgeCurve2DRepId::default().is_valid());
    assert!(!CoEdgePolygon2DRepId::default().is_valid());
    assert!(!CoEdgePolygonOnTriRepId::default().is_valid());
    assert!(!FaceSurfaceRepId::default().is_valid());
    assert!(!FaceTriangulationRepId::default().is_valid());
}
