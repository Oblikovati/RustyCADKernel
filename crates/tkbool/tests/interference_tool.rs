//! Factory, copy, and shared-mutation contracts of TopOpeBRepDS_InterferenceTool.
use rustycad_tkbool::top_ope_brep_ds::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Point2};

type Tool = TopOpeBRepDSInterferenceTool;
type Kind = TopOpeBRepDSKind;
type Config = TopOpeBRepDSConfig;

#[test]
fn curve_and_edge_factories_keep_arbitrary_kinds_indices_and_parameter_bits() {
    let transition = TopOpeBRepDSTransition::from_orientation(Orientation::Forward);
    let edge = Tool::make_edge_interference(&transition, Kind::Edge, -2, Kind::Vertex, 7, -0.0);
    let curve = Tool::make_curve_interference(&transition, Kind::Curve, 3, Kind::Point, 4, 0.25);
    assert_eq!(
        edge.get().unwrap().gkgsks(),
        (Kind::Vertex, 7, Kind::Edge, -2)
    );
    assert_eq!(
        curve.get().unwrap().gkgsks(),
        (Kind::Point, 4, Kind::Curve, 3)
    );
    assert_eq!(
        Tool::parameter(&edge).unwrap().to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(Tool::parameter(&curve).unwrap(), 0.25);
    let alias = curve.clone();
    Tool::set_parameter(&curve, -1.5).unwrap();
    assert_eq!(Tool::parameter(&alias).unwrap(), -1.5);
}

#[test]
fn duplicate_complements_transition_and_allocates_independent_parameter_storage() {
    let transition = TopOpeBRepDSTransition::from_orientation(Orientation::Forward);
    let original = Tool::make_curve_interference(&transition, Kind::Curve, 3, Kind::Point, 4, 0.25);
    let duplicate = Tool::duplicate_curve_point_interference(&original).unwrap();
    assert_ne!(original.object_address(), duplicate.object_address());
    assert_eq!(
        duplicate.get().unwrap().transition(),
        transition.complement()
    );
    assert_eq!(
        duplicate.get().unwrap().gkgsks(),
        original.get().unwrap().gkgsks()
    );
    Tool::set_parameter(&original, 2.0).unwrap();
    assert_eq!(Tool::parameter(&duplicate).unwrap(), 0.25);
}

#[test]
fn face_curve_factory_preserves_pcurve_identity_and_surface_factory_sets_kinds() {
    let transition = TopOpeBRepDSTransition::default();
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(Point2::default(), Direction2::X)).cast();
    let face = Tool::make_face_curve_interference(&transition, 2, 3, &pcurve);
    let surface: Handle<TopOpeBRepDSSurfaceCurveInterference> = face.clone().cast();
    assert_eq!(
        surface.get().unwrap().pcurve().object_address(),
        pcurve.object_address()
    );
    assert_eq!(
        face.get().unwrap().gkgsks(),
        (Kind::Curve, 3, Kind::Face, 2)
    );
    let solid = Tool::make_solid_surface_interference(&transition, 4, 5);
    assert_eq!(
        solid.get().unwrap().gkgsks(),
        (Kind::Surface, 5, Kind::Solid, 4)
    );
    let typed: Handle<TopOpeBRepDSSolidSurfaceInterference> = solid.cast();
    assert!(!typed.is_null());
}

#[test]
fn shape_factories_preserve_bound_flags_configuration_and_edge_parameter() {
    let transition = TopOpeBRepDSTransition::default();
    let edge =
        Tool::make_edge_vertex_interference(&transition, 2, 3, true, Config::SameOriented, 1.25);
    let typed: Handle<TopOpeBRepDSEdgeVertexInterference> = edge.clone().cast();
    assert_eq!(
        edge.get().unwrap().gkgsks(),
        (Kind::Vertex, 3, Kind::Edge, 2)
    );
    assert_eq!(typed.get().unwrap().config(), Config::SameOriented);
    assert!(typed.get().unwrap().gbound());
    assert_eq!(typed.get().unwrap().parameter(), 1.25);
    let face = Tool::make_face_edge_interference(&transition, 4, 5, false, Config::DiffOriented);
    let typed: Handle<TopOpeBRepDSFaceEdgeInterference> = face.clone().cast();
    assert_eq!(face.get().unwrap().gkgsks(), (Kind::Edge, 5, Kind::Face, 4));
    assert_eq!(typed.get().unwrap().config(), Config::DiffOriented);
    assert!(!typed.get().unwrap().gbound());
}

#[test]
fn invalid_parameter_targets_return_an_explicit_failure() {
    let plain = Handle::new(TopOpeBRepDSInterference::default());
    for interference in [Handle::default(), plain] {
        let failure = Tool::parameter(&interference).unwrap_err();
        assert_eq!(failure.exception_type(), "Standard_NullObject");
        assert!(
            failure
                .message()
                .contains("expected a non-null TopOpeBRepDS_CurvePointInterference")
        );
        assert!(Tool::set_parameter(&interference, 1.0).is_err());
        assert!(Tool::duplicate_curve_point_interference(&interference).is_err());
    }
}
