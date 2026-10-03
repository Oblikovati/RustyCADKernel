//! Native-backed wire orientation and MapWires behavior for ShapeFix_Face.
#[path = "support/shape_fix_face.rs"]
mod support;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topods::{Builder, Face, ShapeIterator, Wire};
use rustycad_tkernel::standard::{Handle, RuntimeTyped, Transient};
use rustycad_tkg3d::geom::{GeomCylindricalSurface, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Direction3, Line3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisSurface;
use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
use rustycad_tkshhealing::shape_extend::{BasicMsgRegistrator, ShapeExtendMsgRegistrator};
use rustycad_tkshhealing::shape_fix::{FixRoot, ShapeFixFace, ShapeFixFaceWireMap, ShapeFixRoot};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use support::*;

#[test]
fn single_clockwise_wire_is_reversed_and_retained_as_face_result() {
    let face = face_with_wires(&[WireFixtureSpec {
        reversed: true,
        bounds: (0.0, 0.0, 4.0, 4.0),
    }]);
    let fixer = ShapeFixFace::new(face.into_shape());
    assert!(fixer.fix_orientation().unwrap());
    assert_eq!(
        wire_orientations(&fixer.face().unwrap()),
        [Orientation::Forward]
    );
}

#[test]
fn single_counterclockwise_wire_is_a_noop_and_map_remains_empty() {
    let face = face_with_wires(&[WireFixtureSpec {
        reversed: false,
        bounds: (0.0, 0.0, 4.0, 4.0),
    }]);
    let fixer = ShapeFixFace::new(face.into_shape());
    let mut map = ShapeFixFaceWireMap::default();
    assert!(!fixer.fix_orientation_with_map(&mut map).unwrap());
    assert!(map.is_empty());
    assert_eq!(
        wire_orientations(&fixer.face().unwrap()),
        [Orientation::Forward]
    );
}

#[test]
fn face_mode_defaults_and_shape_fix_root_parent_are_native() {
    let fixer = ShapeFixFace::default();
    assert_eq!(fixer.fix_orientation_mode(), -1);
    assert_eq!(fixer.fix_add_natural_bound_mode(), -1);
    assert!(fixer.is_kind(ShapeFixRoot::static_type()));
    fixer.set_fix_orientation_mode(0);
    fixer.set_fix_add_natural_bound_mode(1);
    assert_eq!(fixer.fix_orientation_mode(), 0);
    assert_eq!(fixer.fix_add_natural_bound_mode(), 1);
    let parent: &ShapeFixRoot = &fixer;
    parent.set_precision(0.25);
    assert_eq!(fixer.precision(), 0.25);
    fixer.set_min_tolerance(0.125);
    assert_eq!(parent.min_tolerance(), 0.125);
}

#[test]
fn surface_initialization_keeps_root_face_and_result_as_separate_states() {
    let surface = plane_surface();
    let fixer = ShapeFixFace::default();
    fixer.init_surface(&surface, 1e-6, false).unwrap();
    let parent: &ShapeFixRoot = &fixer;
    let base = parent.fix_root_data().with(|fields| fields.shape.clone());

    assert_eq!(base.orientation(), Orientation::Forward);
    assert_eq!(fixer.face().unwrap().orientation(), Orientation::Reversed);
    assert!(fixer.result().is_null());
}

#[test]
fn surface_initializers_add_wire_and_natural_bound_mode_follow_face_state() {
    let surface = plane_surface();
    let fixer = ShapeFixFace::default();
    fixer.init_surface(&surface, 1e-6, true).unwrap();
    assert_eq!(count_wires(&fixer.face().unwrap()), 0);

    let analysis = Handle::new(ShapeAnalysisSurface::new(&surface).unwrap());
    fixer.init_analysis_surface(&analysis, 1e-5, false).unwrap();
    assert_eq!(fixer.face().unwrap().orientation(), Orientation::Reversed);

    let wire = square(&WireFixtureSpec {
        reversed: false,
        bounds: (0.0, 0.0, 4.0, 4.0),
    });
    fixer.add_wire(&wire).unwrap();
    assert_eq!(count_wires(&fixer.face().unwrap()), 1);
    assert!(
        ShapeFixFace::default()
            .init_analysis_surface(&Handle::default(), 1e-7, true)
            .is_err()
    );
}

#[test]
fn sphere_natural_bound_mode_suppresses_single_wire_reversal() {
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap())
        .cast::<dyn GeomSurface>();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &sphere, 1e-7)
        .unwrap();
    let mut wire_shape = patch_wire(&sphere, (0.2, 1.0, 0.1, 0.5)).into_shape();
    wire_shape.set_orientation(Orientation::Reversed);
    Builder.add(&mut face, &wire_shape).unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());
    fixer.set_fix_add_natural_bound_mode(1);

    assert!(!fixer.fix_orientation().unwrap());
    assert_eq!(
        wire_orientations(&fixer.face().unwrap()),
        [Orientation::Reversed]
    );
}

#[test]
fn nested_wire_map_and_rebuilt_topology_match_native_probe() {
    let face = face_with_wires(&[
        WireFixtureSpec {
            reversed: false,
            bounds: (0.0, 0.0, 10.0, 10.0),
        },
        WireFixtureSpec {
            reversed: false,
            bounds: (2.0, 2.0, 8.0, 8.0),
        },
        WireFixtureSpec {
            reversed: false,
            bounds: (3.0, 3.0, 4.0, 4.0),
        },
    ]);
    let fixer = ShapeFixFace::new(face.into_shape());
    let mut map = ShapeFixFaceWireMap::default();
    assert!(fixer.fix_orientation_with_map(&mut map).unwrap());
    assert_eq!(map.extent(), 1);
    assert_eq!(
        map.items()
            .map(|(_, children)| children.len())
            .sum::<usize>(),
        2
    );
    assert_eq!(count_wires(&fixer.face().unwrap()), 3);
}

#[test]
fn disjoint_outer_wires_are_each_returned_in_native_map() {
    let face = face_with_wires(&[
        WireFixtureSpec {
            reversed: false,
            bounds: (0.0, 0.0, 2.0, 2.0),
        },
        WireFixtureSpec {
            reversed: false,
            bounds: (5.0, 0.0, 7.0, 2.0),
        },
    ]);
    let fixer = ShapeFixFace::new(face.into_shape());
    let mut map = ShapeFixFaceWireMap::default();
    assert!(!fixer.fix_orientation_with_map(&mut map).unwrap());
    assert_eq!(map.extent(), 2);
    assert_eq!(
        map.items()
            .map(|(_, children)| children.len())
            .sum::<usize>(),
        0
    );
    assert_eq!(count_wires(&fixer.face().unwrap()), 2);
}

#[test]
fn orientation_applies_and_replaces_the_context_face() {
    let original = face_with_wires(&[WireFixtureSpec {
        reversed: false,
        bounds: (20.0, 20.0, 30.0, 30.0),
    }]);
    let replacement = face_with_wires(&[WireFixtureSpec {
        reversed: true,
        bounds: (0.0, 0.0, 4.0, 4.0),
    }]);
    let context = Handle::new(ShapeBuildReShape::new());
    context
        .replace(
            &original.clone().into_shape(),
            &replacement.clone().into_shape(),
        )
        .unwrap();
    let fixer = ShapeFixFace::new(original.clone().into_shape());
    fixer.set_context(&context);

    assert!(fixer.fix_orientation().unwrap());
    let applied = context
        .apply(
            &original.clone().into_shape(),
            rustycad_tkg3d::top_abs::ShapeEnum::Shape,
        )
        .unwrap();
    assert!(applied.is_same(&fixer.face().unwrap().into_shape()));
    assert!(fixer.result().is_null());
    assert_eq!(
        wire_orientations(&fixer.face().unwrap()),
        [Orientation::Forward]
    );
}

#[test]
fn sub_confusion_one_edge_wire_is_removed_while_outer_wire_rebuilds() {
    let mut face = face_with_wires(&[WireFixtureSpec {
        reversed: true,
        bounds: (0.0, 0.0, 4.0, 4.0),
    }]);
    let line = Line3::new(
        Point3::new(6.0, 6.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    let edge = BRepBuilderAPIMakeEdge::from_lin_params(line, 0.0, 1.0)
        .unwrap()
        .edge()
        .unwrap();
    BRepBuilder::default()
        .range(&edge, 0.0, 1e-8, true)
        .unwrap();
    let (_, first, last) = BRepTool::curve(&edge).unwrap();
    assert_eq!((first, last), (0.0, 1e-8));
    let mut tiny = Wire::default();
    Builder.make_wire(&mut tiny);
    Builder.add(&mut tiny, &edge.into_shape()).unwrap();
    Builder.add(&mut face, &tiny.into_shape()).unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());

    assert!(fixer.fix_orientation().unwrap());
    assert_eq!(
        wire_orientations(&fixer.face().unwrap()),
        [Orientation::Forward]
    );
}

#[test]
fn rebuilding_preserves_non_orientable_internal_wire_children() {
    let mut face = face_with_wires(&[WireFixtureSpec {
        reversed: true,
        bounds: (0.0, 0.0, 4.0, 4.0),
    }]);
    let mut internal = square(&WireFixtureSpec {
        reversed: false,
        bounds: (6.0, 6.0, 8.0, 8.0),
    })
    .into_shape();
    internal.set_orientation(Orientation::Internal);
    Builder.add(&mut face, &internal).unwrap();
    let fixer = ShapeFixFace::new(face.into_shape());

    assert!(fixer.fix_orientation().unwrap());
    let orientations = wire_orientations(&fixer.face().unwrap());
    assert_eq!(orientations.len(), 2);
    assert!(orientations.contains(&Orientation::Forward));
    assert!(orientations.contains(&Orientation::Internal));
    assert_eq!(count_wires(&fixer.face().unwrap()), 2);
}

#[test]
fn periodic_seam_classification_shifts_the_candidate_pcurve() {
    let surface = Handle::new(
        GeomCylindricalSurface::new(
            Axis3::new(
                Point3::default(),
                Direction3::new(0.0, 0.0, 1.0).unwrap(),
                Direction3::new(1.0, 0.0, 0.0).unwrap(),
            )
            .unwrap(),
            1.0,
        )
        .unwrap(),
    )
    .cast::<dyn GeomSurface>();
    let left = patch_wire(&surface, (5.8, 6.4, 1.0, 4.0));
    let right = patch_wire(&surface, (0.1, 0.5, 1.0, 4.0));
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    Builder.add(&mut face, &left.clone().into_shape()).unwrap();
    Builder.add(&mut face, &right.clone().into_shape()).unwrap();
    let before = first_pcurve_midpoint(&face, &right);
    let shifted = patch_wire(
        &surface,
        (
            0.1 + std::f64::consts::TAU,
            0.5 + std::f64::consts::TAU,
            1.0,
            4.0,
        ),
    );
    let mut shifted_face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut shifted_face, &surface, 1e-7)
        .unwrap();
    Builder
        .add(&mut shifted_face, &shifted.into_shape())
        .unwrap();
    let shifted_classifier =
        BRepTopAdaptorFClass2d::new(&shifted_face, rustycad_tkernel::precision::p_confusion())
            .unwrap();
    let left_edges = ShapeIterator::new(&left.clone().into_shape(), true, true);
    let shifted_states: Vec<_> = left_edges
        .map(|edge| {
            let pcurve =
                BRepTool::curve_on_face(&rustycad_tkbrep::topods::edge(&edge).unwrap(), &face)
                    .unwrap();
            let point = pcurve
                .pcurve
                .value((pcurve.first + pcurve.last) * 0.5)
                .unwrap();
            shifted_classifier.perform(point, false).unwrap()
        })
        .collect();
    assert_eq!(shifted_states, [rustycad_tkg3d::top_abs::State::Out; 4]);
    let fixer = ShapeFixFace::new(face.into_shape());
    let registrator = Handle::new(ShapeExtendMsgRegistrator::new());
    fixer.set_msg_registrator(&registrator.cast::<dyn BasicMsgRegistrator>());

    let changed = fixer.fix_orientation().unwrap();
    let after = first_pcurve_midpoint(&fixer.face().unwrap(), &right);
    assert!(changed, "native MSG5 reversal sets FixOrientation done");
    assert!((before.x() - 0.1).abs() < 1e-9);
    assert!((after.x() - (0.1 + std::f64::consts::TAU)).abs() < 1e-7);
    let warnings = registrator.map_shape(|messages| {
        messages
            .values()
            .flat_map(|list| list.iter().map(|message| message.original().to_utf8()))
            .collect::<Vec<_>>()
    });
    assert_eq!(warnings.len(), 2);
    assert!(warnings.iter().any(|message| message.contains("MSG5")));
    assert!(warnings.iter().any(|message| message.contains("MSG11")));
}

#[test]
fn immediate_wire_reversal_preserves_shared_periodic_seam_state() {
    let surface = cylinder_surface();
    let wire = patch_wire(&surface, (0.0, std::f64::consts::TAU, 1.0, 4.0));
    let reversed = rustycad_tkbrep::topods::wire(&wire.clone().into_shape().reversed()).unwrap();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    Builder.add(&mut face, &wire.clone().into_shape()).unwrap();
    Builder.add(&mut face, &reversed.into_shape()).unwrap();

    let fixer = ShapeFixFace::new(face.into_shape());
    assert!(fixer.fix_orientation().unwrap());
    let fixed_face = fixer.face().unwrap();
    let fixed_wires = wire_shapes(&fixed_face);
    assert_eq!(fixed_wires.len(), 2);
    assert!(
        (first_pcurve_midpoint(&fixed_face, &fixed_wires[0]).x() - std::f64::consts::TAU).abs()
            < 1e-7
    );
    assert!(
        (first_pcurve_midpoint(&fixed_face, &fixed_wires[1]).x() - std::f64::consts::TAU).abs()
            < 1e-7
    );
}
