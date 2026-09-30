//! Behavior of the IntPolyh port beyond the numeric probe, which compares it with native
//! OCCT bit for bit: the storage model of the arrays, the records and their dumps.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::int_polyh::*;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3};

fn square() -> Adaptor3dSurfaceHandle {
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-1.0, 1.0, -1.0, 1.0)).unwrap()).cast()
}

#[test]
fn array_init_allocates_one_more_item_and_grows_by_the_increment() {
    let mut a: IntPolyhArray<i32> = IntPolyhArray::with_size(2, 3);
    assert_eq!((a.get_n(), a.nb_items()), (2, 0));
    // The storage holds items 0..=2: the item past the allocated count is readable.
    assert_eq!(*a.value(2).unwrap(), 0);
    assert_eq!(
        a.value(3).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
    a.increment_nb_items();
    a.increment_nb_items();
    assert_eq!((a.get_n(), a.nb_items()), (5, 2));
    a[4] = 9;
    a.init(4);
    // Init(N) resets the item N.
    assert_eq!(a[4], 0);
}

#[test]
fn array_copy_keeps_its_own_increment() {
    let source: IntPolyhArray<i32> = IntPolyhArray::with_size(1, 10);
    let mut target: IntPolyhArray<i32> = IntPolyhArray::with_size(0, 2);
    target.copy_from(&source);
    target.increment_nb_items();
    // Grown by the target's increment (2), not the source's (10).
    assert_eq!(target.get_n(), 3);
    let mut clone = source.clone();
    clone.increment_nb_items();
    assert_eq!(clone.get_n(), 11);
}

#[test]
fn section_line_keeps_a_spare_point() {
    let mut line = IntPolyhSectionLine::new();
    assert_eq!(line.nb_start_points(), -1);
    assert!(line.change_value(-1).is_err());
    line.init(5);
    let mut sp = IntPolyhStartPoint::default();
    sp.set_xyz(1.0, 2.0, 3.0);
    *line.change_value(0).unwrap() = sp;
    line.increment_nb_start_points();
    line.prepend(&IntPolyhStartPoint::default());
    assert_eq!((line.nb_start_points(), line.get_n()), (2, 3));
    assert_eq!(line.value(1).unwrap().x(), 1.0);
    assert_eq!(
        line.value(3).err().unwrap().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn point_dump_writes_the_printf_text() {
    let mut text = String::new();
    IntPolyhPoint::new(1234.5, -0.5, 0.0, 1.0, 2.0)
        .dump_index(3, &mut text)
        .unwrap();
    assert_eq!(
        text,
        "\nPoint(  3) : x=+1.234e+03g y=-5.000e-01g z=+0.000e+00g u=+1.000e+00g v=+2.000e+00g poc=  1\n"
    );
}

#[test]
fn couple_is_equal_in_either_order() {
    let (a, b) = (
        IntPolyhCouple::new(1, 2, 0.5),
        IntPolyhCouple::new(2, 1, -2.0),
    );
    assert!(a.is_equal(&b) && a == b);
    let mut text = String::new();
    a.dump(4, &mut text).unwrap();
    assert_eq!(text, "\nCouple(  4) :     1     2   0 0.500000\n");
}

#[test]
fn start_point_same_edge_parameter_or_same_vertex() {
    let a = IntPolyhStartPoint::new([0.0; 3], [0.1, 0.2, 0.3, 0.4], (1, 5, 0.25), (2, 7, 0.5), 0);
    let b = IntPolyhStartPoint::new([0.0; 3], [0.9, 0.9, 0.3, 0.4], (1, 5, 0.25), (2, 8, 0.1), 0);
    assert_eq!(a.check_same_sp(&b), 1);
    let vertex = IntPolyhStartPoint::new(
        [0.0; 3],
        [0.1, 0.2, 0.0, 0.0],
        (1, -1, -1.0),
        (2, 3, 0.5),
        0,
    );
    assert_eq!(vertex.check_same_sp(&a), 1);
    assert_eq!(
        a.get_edge_points(&IntPolyhTriangle::new(4, 5, 6)),
        (1, None)
    );
}

#[test]
fn triangle_edges_and_refinement_on_a_grid() {
    let s = square();
    let mut m = IntPolyhMaillageAffinage::with_samples(&s, (3, 3), &s, (3, 3), 0);
    m.fill_array_of_pnt(1).unwrap();
    m.fill_array_of_triangles(1);
    m.fill_array_of_edges(1);
    let (mut p, mut t, mut e) = (
        m.get_array_of_points(1).clone(),
        m.get_array_of_triangles(1).clone(),
        m.get_array_of_edges(1).clone(),
    );
    assert_eq!((p.nb_items(), t.nb_items(), e.nb_items()), (9, 8, 16));
    // The first triangle has three edges; its first side borders nothing.
    assert!((1..=3).all(|k| t[0].get_edge_number(k) >= 0));
    assert_eq!(t[0].get_next_triangle(0, 1, &e), -1);
    let mut first = t[0].clone();
    first
        .middle_refinement(0, &*s, &mut p, &mut t, &mut e)
        .unwrap();
    assert_eq!((p.nb_items(), t.nb_items()), (10, 12));
    assert!(!first.is_intersection_possible() && first.deflection() == -1.0);
}

#[test]
fn intersection_of_crossing_squares_has_a_section_line() {
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(0.0, 1.0, 0.0).unwrap(),
    )
    .unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    let other: Adaptor3dSurfaceHandle =
        Handle::new(GeomAdaptorSurface::with_bounds(&plane, (-1.0, 1.0, -1.0, 1.0)).unwrap())
            .cast();
    let inter = IntPolyhIntersection::new(&square(), &other).unwrap();
    assert!(inter.is_done() && !inter.is_parallel());
    assert!(inter.nb_section_lines() > 0);
    let n = inter.nb_points_in_line(1).unwrap();
    for k in 1..=n {
        let [x, y, z, ..] = inter.get_line_point(1, k).unwrap();
        // The line y = 0, z = 0 on both planes.
        assert!(y.abs() < 1.0e-9 && z.abs() < 1.0e-9 && x.abs() <= 1.0 + 1.0e-9);
    }
    assert_eq!(inter.nb_points_in_tangent_zone(9), 1);
}
