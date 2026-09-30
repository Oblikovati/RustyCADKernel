//! Behaviour of the HLRAlgo value classes: the box functions, BiPoint, the edge and
//! wire blocks, EdgeStatus with EdgeIterator, the interference records and the
//! Projector, with the native quirks the port keeps (docs/tkhlr-progress.md).
//! tools/verify_tkhlr_numeric.py compares the same members with OCCT bit for bit.
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkhlr::hlr_algo::{
    self as ha, BiPoint, BiPointFace, BiPointPoints, BiPointRegularity, Coincidence, EdgeIterator,
    EdgeStatus, EdgesBlock, Interference, Intersection, MinMaxIndices, Projector, WiresBlock,
};
use rustycad_tkmath::gp::{Axis2, Direction3, Point2, Point3, Transform3, Vector2, Vector3};

#[test]
fn update_min_max_projects_on_seven_plane_directions_and_z() {
    let (mut min, mut max) = ([0.0; 16], [0.0; 16]);
    ha::init_min_max(1e100, &mut min, &mut max);
    assert_eq!((min[3], max[3]), (1e100, -1e100));
    ha::update_min_max(1.0, 0.0, 5.0, &mut min, &mut max);
    assert_eq!((min[0], min[1], min[14], max[15]), (1.0, 0.0, 5.0, 5.0));
    ha::enlarge_min_max(0.5, &mut min, &mut max);
    assert_eq!((min[0], max[0]), (0.5, 1.5));
}

#[test]
fn encode_and_decode_pack_fifteen_bit_fields() {
    let min = MinMaxIndices {
        min: [1, 2, 3, 4, 5, 6, 7, 8],
        max: [9, 10, 11, 12, 13, 14, 15, 16],
    };
    let max = MinMaxIndices {
        min: [0x7fff, 0x8001, 3, 4, 5, 6, 7, 8],
        max: [1; 8],
    };
    let mut code = MinMaxIndices::default();
    ha::encode_min_max(&min, &max, &mut code);
    assert_eq!(code.min[0], (1 << 16) + 2);
    let (mut a, mut b) = (MinMaxIndices::default(), MinMaxIndices::default());
    ha::decode_min_max(&code, &mut a, &mut b);
    assert_eq!(a, min);
    assert_eq!(b.min[1], 1, "only the low 15 bits of 0x8001 survive");
    assert_eq!(ha::size_box(&min, &min), 0.0);
}

#[test]
fn minimize_and_maximize_move_both_arrays() {
    let mut a = MinMaxIndices {
        min: [5; 8],
        max: [5; 8],
    };
    let other = MinMaxIndices {
        min: [1; 8],
        max: [9; 8],
    };
    a.minimize(&other);
    assert_eq!((a.min[0], a.max[0]), (1, 5));
    let (mut o1, mut o2) = (a, a);
    ha::add_min_max(&other, &other, &mut o1, &mut o2);
    assert_eq!((o1.min[0], o2.max[0]), (1, 9));
    ha::copy_min_max(&other, &a, &mut o1, &mut o2);
    assert_eq!((o1, o2), (other, a));
}

#[test]
fn bi_point_constructors_set_faces_and_flags() {
    let points = BiPointPoints::from_coordinates([
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ]);
    let face = BiPointFace {
        conex: 2,
        pt1: 3,
        pt2: 4,
    };
    let regular = BiPointRegularity {
        reg1: true,
        intl: true,
        ..Default::default()
    };
    let a = BiPoint::with_faces_regularity(points, 7, [face, face], regular);
    assert!(a.rg1_line() && a.int_line() && !a.rgn_line() && !a.out_line());
    assert_eq!((a.indices().face_conex2, a.indices().seg_flags), (2, 9));
    let mut b = BiPoint::with_flags(points, 7, 0x1f);
    assert!(!b.hidden(), "every constructor clears the hidden flag");
    b.set_hidden(true);
    b.indices_mut().min_seg = 3;
    b.points_mut().pnt1 = points.pnt2;
    assert!(b.hidden() && b.points().pnt1 == points.pnt2);
    assert_eq!(b.points().pnt_p2_2d().x(), 10.0);
    assert_eq!(BiPoint::default().indices().shape_index, -1);
    let c = BiPoint::with_face_flags(points, 1, face, 4);
    let d = BiPoint::with_face_regularity(points, 1, face, regular);
    let e = BiPoint::with_faces_flags(points, 1, [face, face], 2);
    let f = BiPoint::with_regularity(points, 1, regular);
    assert!(c.out_line() && d.rg1_line() && e.rgn_line() && f.int_line());
}

#[test]
fn edges_block_flags_and_ranks() {
    let block = EdgesBlock::new(1).unwrap();
    block.set_edge(1, 4).unwrap();
    block.set_orientation(1, Orientation::External).unwrap();
    for set in [
        EdgesBlock::set_out_line,
        EdgesBlock::set_internal,
        EdgesBlock::set_double,
        EdgesBlock::set_iso_line,
    ] {
        set(&block, 1, true).unwrap();
    }
    block.set_double(1, false).unwrap();
    assert_eq!(block.orientation(1).unwrap(), Orientation::External);
    assert!(block.out_line(1).unwrap() && block.internal(1).unwrap() && block.iso_line(1).unwrap());
    assert!(!block.double(1).unwrap());
    let error = block.edge(2).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    let packed = MinMaxIndices {
        min: [3; 8],
        max: [4; 8],
    };
    block.update_min_max(&packed);
    assert_eq!(block.min_max(), packed);
    assert_eq!(EdgesBlock::TYPE_NAME, "HLRAlgo_EdgesBlock");
}

#[test]
fn wires_block_holds_shared_edge_blocks() {
    let wires = WiresBlock::new(1).unwrap();
    assert_eq!(wires.nb_wires(), 1);
    assert!(wires.wire(1).unwrap().is_null());
    wires
        .set(1, Handle::new(EdgesBlock::new(2).unwrap()))
        .unwrap();
    assert_eq!(wires.wire(1).unwrap().get().unwrap().nb_edges(), 2);
    wires.update_min_max(&MinMaxIndices::default());
    assert_eq!(wires.min_max(), MinMaxIndices::default());
    assert_eq!(
        wires
            .set(2, Handle::default())
            .unwrap_err()
            .exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn edge_status_hides_and_walks() {
    let mut status = EdgeStatus::with_bounds(0.0, 0.0, 1.0, 0.0);
    status.hide(0.2, 0.0, 0.4, 0.0, false, false);
    status.hide(0.9, 0.0, 0.95, 0.0, true, false);
    assert_eq!(status.nb_visible_part(), 2);
    let mut it = EdgeIterator::new();
    it.init_hidden(&status).unwrap();
    let mut hidden = Vec::new();
    while it.more_hidden() {
        hidden.push((it.hidden().0, it.hidden().2));
        it.next_hidden().unwrap();
    }
    assert_eq!(hidden, vec![(0.2, 0.4)]);
    it.init_visible(&status);
    let mut visible = Vec::new();
    while it.more_visible() {
        visible.push(it.visible().unwrap().0);
        it.next_visible();
    }
    assert_eq!(visible, vec![0.0, 0.4]);
    status.hide(-1.0, 0.0, 2.0, 0.0, false, false);
    assert!(status.all_hidden() && !status.all_visible());
    status.show_all();
    status.set_all_visible(false);
    status.set_all_hidden(true);
    assert_eq!(status.nb_visible_part(), 0);
    status.hide_all();
    status.initialize(0.0, 0.0, 2.0, 0.0);
    assert_eq!(
        status.visible_part(5).unwrap().2,
        2.0,
        "all visible answers the range for any rank"
    );
}

#[test]
fn a_default_status_has_no_hidden_walk() {
    let status = EdgeStatus::new();
    let mut it = EdgeIterator::new();
    assert_eq!(
        it.init_hidden(&status).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn interference_records_keep_their_values() {
    let point = Intersection::with_values(Orientation::Reversed, 1, 2, 3, 0.5, 0.25, State::On);
    let mut boundary = Coincidence::new();
    assert_eq!(boundary.state_3d(), (State::In, State::In));
    boundary.set_2d(4, 0.75);
    boundary.set_state_3d(State::Out, State::On);
    let mut interference = Interference::with_values(
        point,
        boundary,
        Orientation::Forward,
        Orientation::Internal,
        Orientation::External,
    );
    interference.change_boundary().set_2d(5, 0.1);
    interference.change_intersection().set_state(State::Unknown);
    interference.set_boundary_transition(Orientation::Reversed);
    assert_eq!(interference.boundary().value_2d(), (5, 0.1));
    assert_eq!(interference.intersection().state(), State::Unknown);
    assert_eq!(interference.boundary_transition(), Orientation::Reversed);
    assert_eq!(
        (
            point.level(),
            point.seg_index(),
            point.index(),
            point.tolerance()
        ),
        (1, 2, 3, 0.25)
    );
}

#[test]
fn projector_standard_views_and_type_zero_quirk() {
    let r2 = std::f64::consts::FRAC_1_SQRT_2;
    let frame = |n: (f64, f64, f64), x: (f64, f64, f64)| {
        let n = Direction3::new(n.0, n.1, n.2).unwrap();
        let x = Direction3::new(x.0, x.1, x.2).unwrap();
        Axis2::with_x_direction(Point3::new(0.0, 0.0, 0.0), n, x).unwrap()
    };
    let front = Projector::with_frame(frame((0.0, -1.0, 0.0), (1.0, 0.0, 0.0))).unwrap();
    assert_eq!(
        front.project_xyz(Point3::new(1.0, 2.0, 3.0)),
        (1.0, 3.0, -2.0)
    );
    let type0 = Projector::with_frame(frame((0.0, 1.0, 1.0), (r2, -0.5, 0.5))).unwrap();
    let mut untouched = Point2::new(7.0, 8.0);
    type0.project(Point3::new(1.0, 2.0, 3.0), &mut untouched);
    assert_eq!(
        (untouched.x(), untouched.y()),
        (7.0, 8.0),
        "type 0 never stores the projection"
    );
    assert_eq!(
        front.focus().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn projector_perspective_projects_and_shoots() {
    let top =
        Axis2::with_x_direction(Point3::new(0.0, 0.0, 0.0), Direction3::Z, Direction3::X).unwrap();
    let persp = Projector::with_frame_focus(top, 10.0).unwrap();
    assert!(persp.perspective());
    let (p, d) = persp.project_d1(Point3::new(1.0, 1.0, 5.0), Vector3::new(1.0, 0.0, 0.0));
    assert_eq!((p.x(), d.x()), (2.0, 2.0));
    let line = persp.shoot(1.0, 2.0).unwrap();
    assert_eq!(line.location().z(), 10.0);
    let degenerate = Projector::with_frame_focus(top, 0.0).unwrap();
    assert_eq!(
        degenerate.shoot(0.0, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    let mut moved = Projector::new().unwrap();
    let mut shifted = Transform3::default();
    shifted.set_translation_part(Vector3::new(1.0, 0.0, 0.0));
    moved.set(shifted, false, 0.0).unwrap();
    assert_eq!(moved.full_transformation(), shifted);
    assert_eq!(
        moved.transformation(),
        Transform3::default(),
        "the parallel view drops the translation"
    );
    moved.scaled(true).unwrap();
    let mut point = Point3::new(0.0, 0.0, 0.0);
    moved.transform_point(&mut point);
    assert_eq!(point.x(), 1.0);
    let mut vector = Vector3::new(0.0, 1.0, 0.0);
    moved.transform_vector(&mut vector);
    assert_eq!(moved.inverted_transformation().translation_part().x(), -1.0);
    let given = [
        Vector2::new(1.0, 0.0),
        Vector2::new(0.0, 1.0),
        Vector2::new(1.0, 1.0),
    ];
    let custom = Projector::with_directions(shifted, true, 5.0, given).unwrap();
    assert_eq!(custom.directions().2, given[2]);
    assert_eq!(
        Projector::with_transformation(shifted, false, 0.0)
            .unwrap()
            .directions()
            .0,
        Vector2::new(-0.0, 1.0)
    );
}
