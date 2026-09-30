//! Behaviour of the HLRAlgo polyhedral classes: PolyData hiding, PolyShellData,
//! PolyAlgo (Update and the walks) and PolyInternalData (the links, the split and the
//! growth), with the native quirks the port keeps (docs/tkhlr-progress.md).
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkhlr::hlr_algo::{
    BiPoint, BiPointFace, BiPointIndices, BiPointPoints, EdgeStatus, NodeData, PolyAlgo, PolyData,
    PolyHidingData, PolyInternalData, PolyInternalNode, PolyMask, PolyShellData, PolyTriangle,
    TriangleData,
};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Xy, Xyz};

fn shared<T: Default + Clone + Send + Sync + 'static>(values: Vec<T>) -> Handle<HArray1<T>> {
    let array = HArray1::with_bounds(1, values.len() as i32).unwrap();
    array
        .change_array1()
        .iter_mut()
        .zip(values)
        .for_each(|(slot, v)| *slot = v);
    Handle::new(array)
}

/// A hiding triangle in the plane z = 1 over the unit corner of the view.
fn hiding_face() -> PolyData {
    let face = PolyData::new();
    face.set_h_nodes(shared(vec![
        Xyz::new(0.0, 0.0, 1.0),
        Xyz::new(4.0, 0.0, 1.0),
        Xyz::new(0.0, 4.0, 1.0),
    ]));
    let flags = PolyMask::FMskHiding as i32 | PolyMask::EMskGrALin1 as i32;
    face.set_h_t_data(shared(vec![TriangleData {
        node1: 1,
        node2: 2,
        node3: 3,
        flags,
    }]));
    let mut plane = PolyHidingData::default();
    plane.set(1, 0, 0, 0.0, 0.0, 1.0, 1.0);
    face.set_h_ph_dat(shared(vec![plane]));
    face.set_face_index(1);
    face
}

fn segment(z: f64) -> BiPointPoints {
    let (a, b) = (Xyz::new(-1.0, 1.0, z), Xyz::new(3.0, 1.0, z));
    BiPointPoints {
        pnt1: a,
        pnt2: b,
        pnt_p1: a,
        pnt_p2: b,
    }
}

fn triangle() -> PolyTriangle {
    PolyTriangle {
        tol_param: 1e-8,
        tol_ang: 1e-4,
        tolerance: 1e-7,
        ..Default::default()
    }
}

#[test]
fn a_segment_behind_a_triangle_is_hidden_where_it_is_covered() {
    let face = hiding_face();
    let mut status = EdgeStatus::with_bounds(0.0, 1e-8, 1.0, 1e-8);
    let mut working = triangle();
    face.hide_by_poly_data(
        &segment(0.0),
        &mut working,
        &BiPointIndices::default(),
        false,
        &mut status,
    )
    .unwrap();
    // The segment enters the triangle at a quarter and ends on its hypotenuse.
    assert_eq!(status.nb_visible_part(), 1);
    assert_eq!(status.visible_part(1).unwrap().2, 0.25);
    assert_eq!(working.v2, Xy::new(4.0, 0.0));
    let mut front = EdgeStatus::with_bounds(0.0, 1e-8, 1.0, 1e-8);
    face.hide_by_poly_data(
        &segment(2.0),
        &mut triangle(),
        &BiPointIndices::default(),
        false,
        &mut front,
    )
    .unwrap();
    assert!(
        front.all_visible(),
        "a segment in front of the plane is not hidden"
    );
    let mut bounds = BoundingBox3::default();
    face.update_global_min_max(&mut bounds).unwrap();
    assert_eq!(bounds.limits().x_max, 4.0);
    assert!(
        face.hiding() && face.nodes().is_ok() && face.t_data().is_ok() && face.ph_dat().is_ok()
    );
    assert_eq!(face.face_index(), 1);
}

#[test]
fn an_empty_hiding_array_raises_out_of_range() {
    let face = hiding_face();
    face.set_h_ph_dat(shared(Vec::new()));
    let mut status = EdgeStatus::with_bounds(0.0, 1e-8, 1.0, 1e-8);
    let error = face.hide_by_poly_data(
        &segment(0.0),
        &mut triangle(),
        &BiPointIndices::default(),
        false,
        &mut status,
    );
    assert_eq!(error.unwrap_err().exception_type(), "Standard_OutOfRange");
}

#[test]
fn poly_algo_updates_boxes_and_walks_every_segment() {
    let algo = PolyAlgo::new();
    algo.init(1).unwrap();
    let shell = PolyShellData::new(1).unwrap();
    shell
        .poly_data()
        .set_value(1, Handle::new(hiding_face()))
        .unwrap();
    let face = BiPointFace {
        conex: 2,
        pt1: 1,
        pt2: 2,
    };
    shell
        .edges()
        .append(BiPoint::with_faces_flags(segment(0.0), 7, [face, face], 4));
    shell
        .edges()
        .append(BiPoint::with_faces_flags(segment(2.0), 8, [face, face], 0));
    algo.change_poly_shell()
        .set_value(1, Handle::new(shell))
        .unwrap();
    algo.update().unwrap();
    let shells = algo.poly_shell();
    let shell = shells.value(1).unwrap().get().unwrap();
    assert!(shell.hiding());
    assert_eq!(shell.hiding_poly_data().length(), 1);
    assert_ne!(shell.edges().iter().next().unwrap().indices().max_seg, 0);
    drop(shells);
    let mut status = EdgeStatus::new();
    let mut visible_parts = Vec::new();
    algo.init_hide().unwrap();
    while algo.more_hide() {
        let segment = algo.hide(&mut status).unwrap();
        visible_parts.push((segment.index, segment.outl, status.nb_visible_part()));
        algo.next_hide().unwrap();
    }
    assert_eq!(visible_parts, vec![(7, true, 1), (8, false, 1)]);
    algo.init_show().unwrap();
    assert_eq!(algo.show().unwrap().index, 7);
    algo.clear();
    algo.init_show().unwrap();
    assert!(!algo.more_show());
}

#[test]
fn shell_hiding_faces_resize() {
    let shell = PolyShellData::new(2).unwrap();
    assert!(!shell.hiding());
    shell.update_hiding(3).unwrap();
    assert_eq!(shell.hiding_poly_data().upper(), 3);
    shell.update_hiding(0).unwrap();
    assert!(!shell.hiding());
    shell.indices().min = 4;
    assert_eq!(shell.indices().min, 4);
    let mut bounds = BoundingBox3::default();
    let error = shell.update_global_min_max(&mut bounds).unwrap_err();
    assert_eq!(
        error.exception_type(),
        "Standard_NullObject",
        "a null face crashes natively"
    );
}

/// The unit square split by its diagonal (1, 3): nodes 1..4 counter-clockwise.
fn square() -> PolyInternalData {
    let data = PolyInternalData::new(4, 2).unwrap();
    let triangles = data.t_data();
    let mut triangles = triangles.get().unwrap().change_array1();
    *triangles.change_value(1).unwrap() = TriangleData {
        node1: 1,
        node2: 2,
        node3: 3,
        flags: 5,
    };
    *triangles.change_value(2).unwrap() = TriangleData {
        node1: 1,
        node2: 3,
        node3: 4,
        flags: 6,
    };
    drop(triangles);
    let (t, s, n) = (data.t_data(), data.pi_seg(), data.pi_nod());
    let (t, s, n) = (t.get().unwrap(), s.get().unwrap(), n.get().unwrap());
    data.update_links(&mut t.change_array1(), &mut s.change_array1(), &n.array1())
        .unwrap();
    data
}

#[test]
fn update_links_builds_one_segment_per_edge() {
    let data = square();
    assert_eq!(data.nb_pi_seg(), 5);
    let segments = data.pi_seg();
    let diagonal = *segments.get().unwrap().array1().value(3).unwrap();
    assert_eq!(
        (
            diagonal.lst_sg1,
            diagonal.lst_sg2,
            diagonal.conex1,
            diagonal.conex2
        ),
        (3, 1, 1, 2)
    );
}

#[test]
fn a_node_splits_the_diagonal_and_both_triangles() {
    let data = square();
    let (mut n1, mut n2) = (data.pi_nod(), data.pi_nod());
    let a = NodeData {
        normal: Xyz::new(0.0, 0.0, 1.0),
        ..Default::default()
    };
    let ip3 = data
        .add_node(&a, &a, &mut n1, &mut n2, 0.5, 0.5, 0.5, 0.0)
        .unwrap();
    assert_eq!(ip3, 5);
    assert_eq!(
        data.pi_nod().get().unwrap().array1().upper(),
        8,
        "the node array doubled"
    );
    let (mut t1, mut t2, mut s1, mut s2) =
        (data.t_data(), data.t_data(), data.pi_seg(), data.pi_seg());
    data.update_links_split(
        1, 3, 5, &mut t1, &mut t2, &mut s1, &mut s2, &mut n1, &mut n2,
    )
    .unwrap();
    assert_eq!((data.nb_t_data(), data.nb_pi_seg()), (4, 8));
    let mut text = String::new();
    data.dump(&mut text).unwrap();
    assert!(text.starts_with("Node      1 :      1"));
    assert_eq!(text.lines().count(), 5 + 8 + 4);
    data.set_planar(true);
    data.set_int_out_l(true);
    assert!(data.planar() && data.int_out_l());
    data.dec_t_data();
    data.dec_pi_nod();
    data.dec_pi_seg();
    assert_eq!(
        (data.nb_t_data(), data.nb_pi_nod(), data.nb_pi_seg()),
        (3, 4, 7)
    );
}

#[test]
fn growing_redirects_the_second_pointer_only_when_shared() {
    let data = PolyInternalData::new(1, 1).unwrap();
    let (mut a, mut b) = (
        data.t_data(),
        Handle::new(HArray1::with_bounds(0, 0).unwrap()),
    );
    data.inc_t_data(&mut a, &mut b).unwrap();
    assert_eq!(a.object_address(), data.t_data().object_address());
    assert_ne!(b.object_address(), data.t_data().object_address());
    let (mut s1, mut s2) = (data.pi_seg(), data.pi_seg());
    for _ in 0..4 {
        data.inc_pi_seg(&mut s1, &mut s2).unwrap();
    }
    assert_eq!(s2.object_address(), data.pi_seg().object_address());
    let (mut p1, mut p2) = (data.pi_nod(), data.pi_nod());
    data.inc_pi_nod(&mut p1, &mut p2).unwrap();
    assert_eq!(data.nb_pi_nod(), 2);
    let empty = PolyInternalData::new(0, 1);
    assert_eq!(empty.err().unwrap().exception_type(), "Standard_OutOfRange");
    let node = PolyInternalNode::new();
    node.data().scal = 2.0;
    assert_eq!(node.indices().nd_sg, 0);
}
